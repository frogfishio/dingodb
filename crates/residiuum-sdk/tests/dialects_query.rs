//! Query dialects: compile-to-SDA comfort frontends (doc/SDA/DIALECTS.md).
//!
//! Doctrine: dialects are not a hybrid peer language. Pure SDA remains the only
//! lossless path for distinctions foreign surfaces cannot express — especially
//! Null vs absence (SDA_SPEC §4.0.1).

use residiuum_sdk::{
    compile_dialect, json, list_builtin_dialects, BuiltinDialect, Residiuum, SdaShape,
};
use tempfile::tempdir;

#[test]
fn find_dialect_sql_and_json_on_collection() {
    let dir = tempdir().unwrap();
    let mut db = Residiuum::open(dir.path().join("d.residiuum")).unwrap();
    {
        let mut users = db.collection("users").unwrap();
        users
            .put("a", &json!({"name": "Ada", "status": "active", "age": 30}))
            .unwrap();
        users
            .put("b", &json!({"name": "Bob", "status": "idle", "age": 40}))
            .unwrap();
        users
            .put("c", &json!({"name": "Cy", "status": "active", "age": 10}))
            .unwrap();

        let sql_rows = users
            .find_dialect("sql", "SELECT * WHERE status = 'active' AND age >= 18")
            .unwrap();
        assert_eq!(sql_rows.len(), 1);
        assert_eq!(sql_rows[0].0, "a");

        let json_rows = users
            .find_dialect("json", r#"{"status":"active","age":{"$gte":18}}"#)
            .unwrap();
        assert_eq!(json_rows.len(), 1);
        assert_eq!(json_rows[0].0, "a");

        let mongo = users
            .find_dialect("mongo", r#"{"status":"active"}"#)
            .unwrap();
        assert_eq!(mongo.len(), 2);
    }
}

#[test]
fn compile_dialect_profile_and_builtins() {
    assert!(list_builtin_dialects().iter().any(|d| d.id == "sql"));
    assert!(list_builtin_dialects().iter().any(|d| d.id == "rql"));
    let c = BuiltinDialect::Sql
        .compile("WHERE country IN ('TH', 'SG')")
        .unwrap();
    assert_eq!(c.shape, SdaShape::DocumentPredicate);
    assert_eq!(c.dialect, "sql");
    let same = compile_dialect("sql", "WHERE country IN ('TH', 'SG')").unwrap();
    assert_eq!(c.sda, same.sda);
}

/// RQL dialect id is retired on the dialect→SDA surface (RQL-R1).
#[test]
fn rql_dialect_refuses_parallel_sda_path() {
    let rql = r#"
        from orders
        enrich customer using customers
          matching customer_id = id
          expect exactly_one
    "#;
    let err = compile_dialect("rql", rql).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("no longer compiles to SDA") || msg.contains("RQL-R1"),
        "got {msg}"
    );
    assert!(list_builtin_dialects().iter().any(|d| d.id == "rql"));
    assert!(
        !list_builtin_dialects()
            .iter()
            .find(|d| d.id == "rql")
            .unwrap()
            .implemented
    );
}

/// Pure SDA distinguishes stored null from missing key; SQL `IS NULL` cannot.
///
/// That gap is why dialects are comfort frontends, not a hybrid peer language:
/// when exact null-vs-absence meaning is required, callers must use pure SDA.
#[test]
fn pure_sda_null_vs_absence_sql_is_null_collapses() {
    let stored_null = json!({"nickname": null});
    let missing = json!({});
    let present = json!({"nickname": "ada"});

    let only_null = sda_core::Program::parse(
        r#"getPath(input, Seq["nickname"]) = Some(null)"#,
    )
    .unwrap();
    let only_absent = sda_core::Program::parse(
        r#"getPath(input, Seq["nickname"]) = None"#,
    )
    .unwrap();

    assert_eq!(only_null.run_json("input", stored_null.clone()).unwrap(), json!(true));
    assert_eq!(only_null.run_json("input", missing.clone()).unwrap(), json!(false));
    assert_eq!(only_null.run_json("input", present.clone()).unwrap(), json!(false));

    assert_eq!(only_absent.run_json("input", stored_null.clone()).unwrap(), json!(false));
    assert_eq!(only_absent.run_json("input", missing.clone()).unwrap(), json!(true));
    assert_eq!(only_absent.run_json("input", present.clone()).unwrap(), json!(false));

    // SQL dialect: IS NULL matches both missing and stored null (mimicry).
    let sql = compile_dialect("sql", "nickname IS NULL").unwrap();
    assert!(
        sql.notes.iter().any(|n| n.contains("absence") || n.contains("Null")),
        "IS NULL must document Null≠absence collapse: {:?}",
        sql.notes
    );
    let sql_prog = sda_core::Program::parse(&sql.sda).unwrap();
    assert_eq!(sql_prog.run_json("input", stored_null).unwrap(), json!(true));
    assert_eq!(sql_prog.run_json("input", missing).unwrap(), json!(true));
    assert_eq!(sql_prog.run_json("input", present).unwrap(), json!(false));

    // Mongo/JSON can split with $eq / $exists, but still is not full SDA carriers.
    let eq_null = compile_dialect("json", r#"{"nickname": null}"#).unwrap();
    let exists_false =
        compile_dialect("json", r#"{"nickname": {"$exists": false}}"#).unwrap();
    let eq_prog = sda_core::Program::parse(&eq_null.sda).unwrap();
    let ex_prog = sda_core::Program::parse(&exists_false.sda).unwrap();
    assert_eq!(
        eq_prog
            .run_json("input", json!({"nickname": null}))
            .unwrap(),
        json!(true)
    );
    assert_eq!(eq_prog.run_json("input", json!({})).unwrap(), json!(false));
    assert_eq!(ex_prog.run_json("input", json!({})).unwrap(), json!(true));
    assert_eq!(
        ex_prog
            .run_json("input", json!({"nickname": null}))
            .unwrap(),
        json!(false)
    );
}

#[test]
fn find_dialect_sql_is_null_on_collection() {
    let dir = tempdir().unwrap();
    let mut db = Residiuum::open(dir.path().join("d.residiuum")).unwrap();
    {
        let mut users = db.collection("users").unwrap();
        users
            .put("null_nick", &json!({"name": "N", "nickname": null}))
            .unwrap();
        users
            .put("missing_nick", &json!({"name": "M"}))
            .unwrap();
        users
            .put("has_nick", &json!({"name": "H", "nickname": "hi"}))
            .unwrap();

        // SQL comfort: both null and missing match IS NULL.
        let via_sql = users
            .find_dialect("sql", "SELECT * WHERE nickname IS NULL")
            .unwrap();
        let keys: Vec<_> = via_sql.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"null_nick"));
        assert!(keys.contains(&"missing_nick"));
        assert!(!keys.contains(&"has_nick"));

        // Pure SDA: only stored null.
        let only_null = users
            .filter_sda(r#"getPath(input, Seq["nickname"]) = Some(null)"#)
            .unwrap();
        assert_eq!(only_null.len(), 1);
        assert_eq!(only_null[0].0, "null_nick");

        // Pure SDA: only absence.
        let only_missing = users
            .filter_sda(r#"getPath(input, Seq["nickname"]) = None"#)
            .unwrap();
        assert_eq!(only_missing.len(), 1);
        assert_eq!(only_missing[0].0, "missing_nick");
    }
}
