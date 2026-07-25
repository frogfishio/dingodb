//! Stage 7 CLI integration tests: put/get/list, doctor, salvage, serve+connect.

use std::fs;
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

fn dingo_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dingo"))
}

fn run_ok(args: &[&str]) -> String {
    let output = dingo_bin().args(args).output().expect("run dingo");
    assert!(
        output.status.success(),
        "cmd {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn version_and_help() {
    let out = run_ok(&["--version"]);
    assert!(out.contains("-build"), "version={out}");
    let help = run_ok(&["--help"]);
    assert!(help.contains("doctor"));
    assert!(help.contains("salvage"));
    assert!(help.contains("serve"));
}

#[test]
fn put_get_list_delete_roundtrip() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();

    run_ok(&[
        "put",
        store_s,
        "users/user-42",
        "--json",
        r#"{"name":"Alice","status":"active"}"#,
    ]);
    let got = run_ok(&["get", store_s, "users/user-42"]);
    assert!(got.contains("Alice"));

    let list_cols = run_ok(&["list", store_s]);
    assert!(list_cols.contains("users"));

    let list_keys = run_ok(&["list", store_s, "users"]);
    assert!(list_keys.contains("user-42"));

    let json_get = run_ok(&["--json-out", "get", store_s, "users/user-42"]);
    assert!(json_get.contains(r#""found":true"#) || json_get.contains(r#""found": true"#));

    run_ok(&["delete", store_s, "users/user-42"]);
    let missing = dingo_bin()
        .args(["get", store_s, "users/user-42"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
}

#[test]
fn put_bytes_roundtrip() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();
    let bin = dir.path().join("blob.bin");
    fs::write(&bin, b"\x00\xffhello").unwrap();

    run_ok(&[
        "put-bytes",
        store_s,
        "artifacts/build-19",
        bin.to_str().unwrap(),
    ]);
    // list keys
    let keys = run_ok(&["list", store_s, "artifacts"]);
    assert!(keys.contains("build-19"));
}

#[test]
fn doctor_is_read_only_on_healthy_store() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();
    run_ok(&["put", store_s, "users/u1", "--json", r#"{"n":1}"#]);

    // Snapshot mtimes of authoritative paths.
    let active = store.join("active").join("active.dingo");
    let before = fs::metadata(&active).unwrap().modified().unwrap();

    let out = run_ok(&["doctor", store_s]);
    assert!(out.contains("read_only: true"));
    assert!(out.contains("healthy: true"));

    let after = fs::metadata(&active).unwrap().modified().unwrap();
    assert_eq!(before, after, "doctor must not rewrite active segment");
}

#[test]
fn salvage_to_new_path_preserves_live_data() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("damaged.dingo");
    let dst = dir.path().join("recovered.dingo");
    let src_s = src.to_str().unwrap();
    let dst_s = dst.to_str().unwrap();

    run_ok(&["put", src_s, "users/alice", "--json", r#"{"name":"Alice"}"#]);
    run_ok(&["put", src_s, "users/bob", "--json", r#"{"name":"Bob"}"#]);

    // Wipe derived state on source (catalogs/indexes) — salvage must still work.
    for name in ["catalogs", "indexes", "snapshots"] {
        let p = src.join(name);
        if p.exists() {
            fs::remove_dir_all(&p).unwrap();
        }
    }

    let report = run_ok(&["salvage", src_s, "--output", dst_s]);
    assert!(report.contains("source immutable: true"));
    assert!(report.contains("subjects_copied: 2"));

    // Source still readable via open (segments intact).
    let src_get = run_ok(&["get", src_s, "users/alice"]);
    assert!(src_get.contains("Alice"));

    // Destination has recovered values.
    let dst_get = run_ok(&["get", dst_s, "users/bob"]);
    assert!(dst_get.contains("Bob"));
}

#[test]
fn serve_and_sdk_connect_parity() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();

    // Seed via CLI.
    run_ok(&["put", store_s, "users/seed", "--json", r#"{"from":"cli"}"#]);

    // Pick a free port.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let bind = format!("127.0.0.1:{port}");

    let mut child = dingo_bin()
        .args(["serve", store_s, "--bind", &bind])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn dingo serve");

    // Wait for accept.
    thread::sleep(Duration::from_millis(200));
    for _ in 0..20 {
        if std::net::TcpStream::connect(&bind).is_ok() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    let url = format!("dingo://{bind}/app");
    let mut db = dingo_sdk::Dingo::connect(&url).expect("connect");
    assert!(db.is_remote());

    {
        let mut users = db.collection("users").unwrap();
        let seed = users.get("seed").unwrap().unwrap();
        assert_eq!(seed["from"], "cli");
        users
            .put("remote", &serde_json::json!({"from": "sdk"}))
            .unwrap();
        assert_eq!(users.get("remote").unwrap().unwrap()["from"], "sdk");
    }

    // Embedded reopen sees remote write.
    let mut local = dingo_sdk::Dingo::open(&store).unwrap();
    let v = local
        .collection("users")
        .unwrap()
        .get("remote")
        .unwrap()
        .unwrap();
    assert_eq!(v["from"], "sdk");

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn history_command() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();
    run_ok(&["put", store_s, "docs/k", "--json", r#"{"v":1}"#]);
    run_ok(&["put", store_s, "docs/k", "--json", r#"{"v":2}"#]);
    let hist = run_ok(&["history", store_s, "docs/k"]);
    assert!(hist.contains("history"));
    assert!(hist.contains("versions="));
}

#[test]
fn serve_auth_token_required() {
    use dingo_sdk::{ConnectOptions, ErrorCode};

    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();
    run_ok(&["put", store_s, "users/seed", "--json", r#"{"from":"cli"}"#]);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let bind = format!("127.0.0.1:{port}");

    let mut child = dingo_bin()
        .args(["serve", store_s, "--bind", &bind, "--token", "s3cret"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn dingo serve");

    thread::sleep(Duration::from_millis(200));
    for _ in 0..20 {
        if std::net::TcpStream::connect(&bind).is_ok() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    let url = format!("dingo://{bind}/app");

    // No token → authentication_failed.
    let err = match dingo_sdk::Dingo::connect(&url) {
        Ok(_) => panic!("must reject missing token"),
        Err(e) => e,
    };
    assert_eq!(err.code(), ErrorCode::AuthenticationFailed);

    // Wrong token → authentication_failed.
    let err = match dingo_sdk::Dingo::connect_with(&url, ConnectOptions::new().auth_token("wrong"))
    {
        Ok(_) => panic!("must reject wrong token"),
        Err(e) => e,
    };
    assert_eq!(err.code(), ErrorCode::AuthenticationFailed);

    // Correct token → put/get works; receipts carry non-zero event ids.
    let mut db = dingo_sdk::Dingo::connect_with(
        &url,
        ConnectOptions::new()
            .auth_token("s3cret")
            .max_connect_attempts(5)
            .request_timeout(Duration::from_secs(5)),
    )
    .expect("connect with token");
    assert_ne!(db.store_id(), [0u8; 16]);
    {
        let mut users = db.collection("users").unwrap();
        let receipt = users
            .put("authed", &serde_json::json!({"ok": true}))
            .unwrap();
        assert_ne!(receipt.event_id, [0u8; 16]);
        assert_ne!(receipt.store_id, [0u8; 16]);
        assert_eq!(users.get("authed").unwrap().unwrap()["ok"], true);
    }

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn connect_retry_deadline_to_closed_port() {
    use dingo_sdk::{ConnectOptions, ErrorCode};
    use std::time::Instant;

    // Nothing listening: connect should fail after retries within a deadline.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let url = format!("dingo://127.0.0.1:{port}/app");
    let t0 = Instant::now();
    let err = match dingo_sdk::Dingo::connect_with(
        &url,
        ConnectOptions::new()
            .connect_timeout(Duration::from_millis(50))
            .max_connect_attempts(3)
            .retry_backoff(Duration::from_millis(10)),
    ) {
        Ok(_) => panic!("closed port must fail"),
        Err(e) => e,
    };
    let elapsed = t0.elapsed();
    // Three 50ms attempts + small backoffs should finish well under 5s.
    assert!(elapsed < Duration::from_secs(5), "elapsed={elapsed:?}");
    // IO or deadline depending on platform timeout reporting.
    assert!(
        matches!(
            err.code(),
            ErrorCode::Io | ErrorCode::DeadlineExceeded | ErrorCode::Internal
        ),
        "code={:?}",
        err.code()
    );
}
