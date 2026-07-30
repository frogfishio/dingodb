// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::TryStreamExt;
use k2db::{AggregateOp, CountOp, CreateOp, DatabaseConfig, EncryptionConfig, EnsureIndexesOptions, FindOneOp, FindOp, FindOptions, HostConfig, K2Db, K2DbError, OwnershipMode, ProjectionMode, SchemaMode, Scope, UpdateOneOp};
use mongodb::bson::{doc, Document};
use mongodb::options::IndexOptions;
use serde_json::json;
use tempfile::TempDir;
use tokio::process::{Child, Command};
use tokio::time::sleep;

struct TestMongo {
    _tmp: TempDir,
    child: Child,
    db: K2Db,
}

impl TestMongo {
    async fn start(name: &str, strict: bool) -> Self {
        let config = DatabaseConfig {
            name: name.to_owned(),
            hosts: vec![HostConfig {
                host: "127.0.0.1".to_owned(),
                port: None,
            }],
            user: None,
            password: None,
            auth_source: None,
            replica_set: None,
            slow_query_ms: Some(1),
            ownership_mode: if strict { OwnershipMode::Strict } else { OwnershipMode::Lax },
            aggregation_mode: Default::default(),
            secure_field_prefixes: vec!["#".to_owned()],
            secure_field_encryption: None,
            hooks: k2db::QueryHooks::default(),
        };
        Self::start_with_config(config).await
    }

    async fn start_with_config(config: DatabaseConfig) -> Self {
        Self::start_internal(config, false).await
    }

    async fn start_replica_set(mut config: DatabaseConfig) -> Self {
        config.replica_set = Some("rs0".to_owned());
        Self::start_internal(config, true).await
    }

    async fn start_internal(mut config: DatabaseConfig, replica_set: bool) -> Self {
        let tmp = tempfile::tempdir().expect("tempdir");
        let port = next_port();
        let log_path = tmp.path().join("mongod.log");
        let mongod = mongod_bin();

        let mut command = Command::new(mongod);
        command
            .arg("--dbpath")
            .arg(tmp.path())
            .arg("--port")
            .arg(port.to_string())
            .arg("--bind_ip")
            .arg("127.0.0.1")
            .arg("--quiet")
            .arg("--logpath")
            .arg(&log_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if replica_set {
            command.arg("--replSet").arg("rs0");
        }
        let child = command.spawn().expect("spawn mongod");

        config.hosts[0].port = Some(port);

        if replica_set {
            initiate_replica_set(port).await;
        }

        let db = K2Db::new(config).expect("db config");
        wait_until_healthy(&db).await;

        Self { _tmp: tmp, child, db }
    }
}

impl Drop for TestMongo {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

fn mongod_bin() -> PathBuf {
    std::env::var_os("MONGOD_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("mongod"))
}

fn next_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

async fn wait_until_healthy(db: &K2Db) {
    for _ in 0..50 {
        if db.init().await.is_ok() && db.is_healthy().await {
            return;
        }
        sleep(Duration::from_millis(100)).await;
    }
    panic!("mongod did not become healthy in time");
}

async fn initiate_replica_set(port: u16) {
    let client = mongodb::Client::with_uri_str(format!("mongodb://127.0.0.1:{port}/?directConnection=true"))
        .await
        .expect("direct replica-set client");
    let admin = client.database("admin");

    for _ in 0..50 {
        if admin.run_command(doc! { "ping": 1 }).await.is_ok() {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    let _ = admin
        .run_command(doc! {
            "replSetInitiate": {
                "_id": "rs0",
                "members": [
                    { "_id": 0, "host": format!("127.0.0.1:{port}") }
                ]
            }
        })
        .await;

    for _ in 0..80 {
        if let Ok(status) = admin.run_command(doc! { "hello": 1 }).await {
            if status.get_bool("isWritablePrimary").unwrap_or(false) {
                return;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    panic!("replica set did not become writable primary in time");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_get_find_count_update_delete_restore_round_trip() {
    let mongo = TestMongo::start("k2db_rust_crud", false).await;
    let db = &mongo.db;
    let collection = "items";

    let created = db
        .create(collection, "owner1", doc! { "name": "alpha", "seq": 1, "#secret": "hidden" })
        .await
        .expect("create");

    let fetched = db.get(collection, &created.id, None).await.expect("get");
    assert_eq!(fetched.get_str("_owner").unwrap(), "owner1");
    assert_eq!(fetched.get_str("name").unwrap(), "alpha");
    assert_eq!(fetched.get_str("#secret").unwrap(), "hidden");

    let found = db
        .find_one(collection, doc! { "_uuid": &created.id }, Some(vec!["name".to_owned()]), None)
        .await
        .expect("find_one")
        .expect("document");
    assert_eq!(found, doc! { "name": "alpha" });

    let count = db.count(collection, doc! { "name": "alpha" }, None).await.expect("count");
    assert_eq!(count.count, 1);

    let page = db
        .find(
            collection,
            doc! {},
            FindOptions {
                projection: ProjectionMode::Include(vec!["name".to_owned(), "seq".to_owned()]),
                sort: Some(doc! { "seq": 1 }),
                skip: 0,
                limit: 10,
                include_deleted: false,
                deleted_only: false,
            },
            None,
        )
        .await
        .expect("find");
    assert_eq!(page.len(), 1);
    assert_eq!(page[0], doc! { "name": "alpha", "seq": 1 });

    let updated = db
        .update(collection, &created.id, doc! { "name": "beta", "_owner": "bad" }, false, None)
        .await
        .expect("update");
    assert_eq!(updated.updated, 1);

    let after_update = db.get(collection, &created.id, None).await.expect("get updated");
    assert_eq!(after_update.get_str("name").unwrap(), "beta");
    assert_eq!(after_update.get_str("_owner").unwrap(), "owner1");

    let deleted = db.delete(collection, &created.id, None).await.expect("delete");
    assert_eq!(deleted.deleted, 1);

    let hidden = db.find_one(collection, doc! { "_uuid": &created.id }, None, None).await.expect("hidden find");
    assert!(hidden.is_none());

    let deleted_view = db
        .find_one(collection, doc! { "_uuid": &created.id, "_deleted": true }, None, None)
        .await
        .expect("deleted view")
        .expect("deleted doc");
    assert_eq!(deleted_view.get_bool("_deleted").unwrap(), true);

    let restored = db.restore(collection, doc! { "_uuid": &created.id }, None).await.expect("restore");
    assert_eq!(restored.status, "restored");
    assert_eq!(restored.modified, 1);

    let visible_again = db.get(collection, &created.id, None).await.expect("visible again");
    assert_eq!(visible_again.get_str("name").unwrap(), "beta");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn secure_fields_encrypt_at_rest_and_decrypt_on_single_read() {
    let mongo = TestMongo::start_with_config(DatabaseConfig {
        name: "k2db_rust_secure_fields".to_owned(),
        hosts: vec![HostConfig {
            host: "127.0.0.1".to_owned(),
            port: None,
        }],
        user: None,
        password: None,
        auth_source: None,
        replica_set: None,
        slow_query_ms: Some(1),
        ownership_mode: OwnershipMode::Lax,
        aggregation_mode: Default::default(),
        secure_field_prefixes: vec!["#".to_owned()],
        secure_field_encryption: Some(EncryptionConfig {
            key_id: "kid-1".to_owned(),
            key: [9_u8; 32],
        }),
        hooks: k2db::QueryHooks::default(),
    })
    .await;
    let db = &mongo.db;
    let collection = "secure_items";

    let created = db
        .create(collection, "owner1", doc! { "name": "alpha", "#secret": "hidden", "nested": { "#pin": 1234 } })
        .await
        .expect("create encrypted");

    let direct = mongodb::Client::with_uri_str(format!("mongodb://127.0.0.1:{}/", db.config().hosts[0].port.unwrap()))
        .await
        .expect("direct client");
    let coll = direct.database(&db.config().name).collection::<Document>(collection);

    let raw = coll
        .find_one(doc! { "_uuid": &created.id })
        .await
        .expect("raw find")
        .expect("raw doc");
    let raw_secret = raw.get_str("#secret").expect("encrypted secret");
    assert!(raw_secret.starts_with("kid-1:"));
    let raw_nested = raw.get_document("nested").unwrap().get_str("#pin").unwrap();
    assert!(raw_nested.starts_with("kid-1:"));

    let fetched = db.get(collection, &created.id, None).await.expect("decrypted get");
    assert_eq!(fetched.get_str("#secret").unwrap(), "hidden");
    assert_eq!(fetched.get_document("nested").unwrap().get_i32("#pin").unwrap(), 1234);

    let projected = db
        .find_one(collection, doc! { "_uuid": &created.id }, Some(vec!["name".to_owned()]), None)
        .await
        .expect("projected read")
        .expect("projected doc");
    assert_eq!(projected, doc! { "name": "alpha" });

    let listed = db.find(collection, doc! {}, FindOptions::default(), None).await.expect("find list");
    assert_eq!(listed.len(), 1);
    assert!(listed[0].get("#secret").is_none());
    assert!(listed[0].get_document("nested").unwrap().get("#pin").is_none());

    coll.insert_one(doc! {
        "_uuid": "01999ZZZ-AAAA-BBBB-CCCC-DDDDEEEEFFFF",
        "_owner": "owner1",
        "_created": 1_i64,
        "_updated": 1_i64,
        "name": "foreign-kid",
        "#secret": "other-kid:abc.def.ghi"
    })
    .await
    .expect("insert foreign kid");

    let foreign = db.get(collection, "01999ZZZ-AAAA-BBBB-CCCC-DDDDEEEEFFFF", None).await.expect("foreign kid get");
    assert_eq!(foreign.get_str("#secret").unwrap(), "other-kid:abc.def.ghi");

    coll.insert_one(doc! {
        "_uuid": "01999ZZZ-BBBB-CCCC-DDDD-EEEEFFFF0000",
        "_owner": "owner1",
        "_created": 1_i64,
        "_updated": 1_i64,
        "name": "broken",
        "#secret": "kid-1:broken.def.ghi"
    })
    .await
    .expect("insert broken secret");

    let broken = db.get(collection, "01999ZZZ-BBBB-CCCC-DDDD-EEEEFFFF0000", None).await.expect_err("broken decrypt should fail");
    assert_eq!(broken.key.as_deref(), Some("sys_mdb_secure_decrypt_failed"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn strict_scope_isolation_is_enforced() {
    let mongo = TestMongo::start("k2db_rust_scope", true).await;
    let db = &mongo.db;
    let collection = "scoped_items";

    let user1 = db.create(collection, "user1", doc! { "name": "u1" }).await.expect("create user1");
    db.create(collection, "user2", doc! { "name": "u2" }).await.expect("create user2");

    let missing_scope = db.get(collection, &user1.id, None).await.expect_err("missing scope should fail");
    assert_eq!(missing_scope.key.as_deref(), Some("sys_mdb_scope_required"));

    let user1_scope = Scope::owner("user1");
    let user2_scope = Scope::owner("user2");
    let admin_scope = Scope::all();

    let user1_view = db.get(collection, &user1.id, Some(&user1_scope)).await.expect("user1 get");
    assert_eq!(user1_view.get_str("_owner").unwrap(), "user1");

    let cross_owner = db.get(collection, &user1.id, Some(&user2_scope)).await.expect_err("cross owner should fail");
    assert_eq!(cross_owner.key.as_deref(), Some("sys_mdb_get_not_found"));

    let admin_results = db
        .find(collection, doc! {}, FindOptions::default(), Some(&admin_scope))
        .await
        .expect("admin find");
    assert_eq!(admin_results.len(), 2);

    let user1_results = db
        .find(collection, doc! {}, FindOptions::default(), Some(&user1_scope))
        .await
        .expect("user1 find");
    assert_eq!(user1_results.len(), 1);
    assert_eq!(user1_results[0].get_str("_owner").unwrap(), "user1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn find_deleted_policy_matches_contract() {
    let mongo = TestMongo::start("k2db_rust_deleted", false).await;
    let db = &mongo.db;
    let collection = "deleted_items";

    let a = db.create(collection, "owner", doc! { "name": "a", "seq": 1 }).await.expect("create a");
    let _b = db.create(collection, "owner", doc! { "name": "b", "seq": 2 }).await.expect("create b");
    db.delete(collection, &a.id, None).await.expect("delete a");

    let active_only = db
        .find(collection, doc! {}, FindOptions::default(), None)
        .await
        .expect("find active only");
    assert_eq!(active_only.len(), 1);
    assert_eq!(active_only[0].get_str("name").unwrap(), "b");

    let deleted_only = db
        .find(
            collection,
            doc! {},
            FindOptions {
                deleted_only: true,
                ..FindOptions::default()
            },
            None,
        )
        .await
        .expect("find deleted only");
    assert_eq!(deleted_only.len(), 1);
    assert_eq!(deleted_only[0].get_bool("_deleted").unwrap(), true);

    let explicit_deleted_count = db
        .count(collection, doc! { "_deleted": true }, None)
        .await
        .expect("count deleted");
    assert_eq!(explicit_deleted_count.count, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn purge_lifecycle_matches_contract() {
    let mongo = TestMongo::start("k2db_rust_purge", false).await;
    let db = &mongo.db;
    let collection = "purge_items";

    let old = db.create(collection, "owner", doc! { "name": "old" }).await.expect("create old");
    let recent = db.create(collection, "owner", doc! { "name": "recent" }).await.expect("create recent");

    db.delete(collection, &old.id, None).await.expect("delete old");
    db.delete(collection, &recent.id, None).await.expect("delete recent");

    let raw = mongo.db.config().name.clone();
    let direct = mongodb::Client::with_uri_str(format!("mongodb://127.0.0.1:{}/", mongo.db.config().hosts[0].port.unwrap()))
        .await
        .expect("direct client");
    let coll = direct.database(&raw).collection::<mongodb::bson::Document>(collection);
    coll.update_one(doc! { "_uuid": &old.id }, doc! { "$set": { "_updated": 1_i64 } })
        .await
        .expect("backdate old");

    let purged = db.purge_deleted_older_than(collection, 1_000, None).await.expect("purge old");
    assert_eq!(purged.purged, 1);

    let old_missing = db.find_one(collection, doc! { "_uuid": &old.id, "_deleted": true }, None, None).await.expect("old missing");
    assert!(old_missing.is_none());

    let recent_deleted = db.find_one(collection, doc! { "_uuid": &recent.id, "_deleted": true }, None, None).await.expect("recent deleted");
    assert!(recent_deleted.is_some());

    let removed = db.purge(collection, &recent.id, None).await.expect("purge recent");
    assert_eq!(removed.id, recent.id);

    let gone = db.find_one(collection, doc! { "_uuid": &recent.id, "_deleted": true }, None, None).await.expect("gone");
    assert!(gone.is_none());

    let not_deleted = db.create(collection, "owner", doc! { "name": "live" }).await.expect("create live");
    let err = db.purge(collection, &not_deleted.id, None).await.expect_err("purge non-deleted should fail");
    assert_eq!(err.key.as_deref(), Some("sys_mdb_gcol_pg2"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn versioning_round_trip_matches_contract() {
    let mongo = TestMongo::start("k2db_rust_versioning", false).await;
    let db = &mongo.db;
    let collection = "versioned_items";

    let created = db
        .create(collection, "owner", doc! { "name": "v0", "keep": true })
        .await
        .expect("create versioned");

    let v1 = db
        .update_versioned(collection, &created.id, doc! { "name": "v1" }, false, Some(10), None)
        .await
        .expect("update versioned one");
    assert_eq!(v1[0].updated, 1);
    assert_eq!(v1[0].version_saved, 1);

    let v2 = db
        .update_versioned(collection, &created.id, doc! { "name": "v2", "extra": "x" }, true, Some(2), None)
        .await
        .expect("update versioned two");
    assert_eq!(v2[0].version_saved, 2);

    let current = db.get(collection, &created.id, None).await.expect("current");
    assert_eq!(current.get_str("name").unwrap(), "v2");
    assert_eq!(current.get_str("extra").unwrap(), "x");
    assert!(current.get("keep").is_none());

    let versions = db
        .list_versions(collection, &created.id, 0, 10, None)
        .await
        .expect("list versions");
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, 2);
    assert_eq!(versions[1].version, 1);

    let reverted = db
        .revert_to_version(collection, &created.id, 1, None)
        .await
        .expect("revert to version");
    assert_eq!(reverted.updated, 1);

    let after_revert = db.get(collection, &created.id, None).await.expect("after revert");
    assert_eq!(after_revert.get_str("name").unwrap(), "v0");
    assert_eq!(after_revert.get_bool("keep").unwrap(), true);
    assert_eq!(after_revert.get_str("_owner").unwrap(), "owner");

    let missing = db
        .revert_to_version(collection, &created.id, 999, None)
        .await
        .expect_err("missing version should fail");
    assert_eq!(missing.key.as_deref(), Some("sys_mdb_version_not_found"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn versioning_access_is_scope_gated() {
    let mongo = TestMongo::start("k2db_rust_version_scope", true).await;
    let db = &mongo.db;
    let collection = "scoped_versions";

    let created = db
        .create(collection, "user1", doc! { "name": "secret" })
        .await
        .expect("create scoped version");

    let owner_scope = Scope::owner("user1");
    let other_scope = Scope::owner("user2");

    db.update_versioned(collection, &created.id, doc! { "name": "secret2" }, false, Some(5), Some(&owner_scope))
        .await
        .expect("versioned update with owner scope");

    let owner_versions = db
        .list_versions(collection, &created.id, 0, 10, Some(&owner_scope))
        .await
        .expect("owner versions");
    assert_eq!(owner_versions.len(), 1);

    let denied = db
        .list_versions(collection, &created.id, 0, 10, Some(&other_scope))
        .await
        .expect_err("other owner should be denied by gate");
    assert_eq!(denied.key.as_deref(), Some("sys_mdb_get_not_found"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn aggregate_enforces_root_scope_and_non_deleted() {
    let mongo = TestMongo::start("k2db_rust_aggregate_root", true).await;
    let db = &mongo.db;
    let collection = "agg_root";

    db.create(collection, "user1", doc! { "name": "visible", "seq": 1 }).await.expect("create active");
    let deleted = db.create(collection, "user1", doc! { "name": "deleted", "seq": 2 }).await.expect("create deleted");
    db.create(collection, "user2", doc! { "name": "other-owner", "seq": 3 }).await.expect("create other owner");
    db.delete(collection, &deleted.id, Some(&Scope::owner("user1"))).await.expect("delete one");

    let result = db
        .aggregate(
            collection,
            vec![doc! { "$project": { "name": 1, "seq": 1, "_owner": 1 } }],
            0,
            20,
            Some(&Scope::owner("user1")),
        )
        .await
        .expect("aggregate root");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].get_str("name").unwrap(), "visible");
    assert_eq!(result[0].get_i32("seq").unwrap(), 1);
    assert_eq!(result[0].get_str("_owner").unwrap(), "user1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn aggregate_lookup_rewrite_filters_deleted_foreign_docs_and_secure_fields() {
    let mongo = TestMongo::start("k2db_rust_aggregate_lookup", true).await;
    let db = &mongo.db;
    let posts = "agg_posts";
    let profiles = "agg_profiles";

    let good_profile = db
        .create(profiles, "user1", doc! { "label": "good", "#secret": "s1" })
        .await
        .expect("create good profile");
    let deleted_profile = db
        .create(profiles, "user1", doc! { "label": "deleted", "#secret": "s2" })
        .await
        .expect("create deleted profile");
    let other_owner_profile = db
        .create(profiles, "user2", doc! { "label": "other-owner", "#secret": "s3" })
        .await
        .expect("create other-owner profile");

    db.delete(profiles, &deleted_profile.id, Some(&Scope::owner("user1")))
        .await
        .expect("delete foreign profile");

    db.create(posts, "user1", doc! { "name": "good-post", "profile_uuid": &good_profile.id })
        .await
        .expect("create good post");
    db.create(posts, "user1", doc! { "name": "deleted-post", "profile_uuid": &deleted_profile.id })
        .await
        .expect("create deleted post");
    db.create(posts, "user1", doc! { "name": "cross-owner-post", "profile_uuid": &other_owner_profile.id })
        .await
        .expect("create cross-owner post");
    db.create(posts, "user2", doc! { "name": "other-owner-root", "profile_uuid": &other_owner_profile.id })
        .await
        .expect("create other owner root post");

    let result = db
        .aggregate(
            posts,
            vec![
                doc! {
                    "$lookup": {
                        "from": profiles,
                        "localField": "profile_uuid",
                        "foreignField": "_uuid",
                        "as": "profile"
                    }
                },
                doc! {
                    "$project": {
                        "name": 1,
                        "profile": 1,
                        "_owner": 1
                    }
                },
                doc! { "$sort": { "name": 1 } },
            ],
            0,
            20,
            Some(&Scope::owner("user1")),
        )
        .await
        .expect("aggregate lookup");

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].get_str("name").unwrap(), "cross-owner-post");
    assert_eq!(result[0].get_array("profile").unwrap().len(), 0);

    assert_eq!(result[1].get_str("name").unwrap(), "deleted-post");
    assert_eq!(result[1].get_array("profile").unwrap().len(), 0);

    assert_eq!(result[2].get_str("name").unwrap(), "good-post");
    let profiles_out = result[2].get_array("profile").unwrap();
    assert_eq!(profiles_out.len(), 1);
    let profile = profiles_out[0].as_document().expect("profile doc");
    assert_eq!(profile.get_str("label").unwrap(), "good");
    assert!(profile.get("#secret").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn aggregate_unionwith_rewrites_scope_and_deleted_filters() {
    let mongo = TestMongo::start("k2db_rust_aggregate_union", true).await;
    let db = &mongo.db;
    let root = "agg_union_root";
    let extra = "agg_union_extra";

    db.create(root, "user1", doc! { "name": "root-visible" }).await.expect("create root visible");
    let deleted_root = db.create(root, "user1", doc! { "name": "root-deleted" }).await.expect("create root deleted");
    db.create(root, "user2", doc! { "name": "root-other" }).await.expect("create root other");
    db.delete(root, &deleted_root.id, Some(&Scope::owner("user1"))).await.expect("delete root");

    db.create(extra, "user1", doc! { "name": "union-visible" }).await.expect("create union visible");
    let deleted_extra = db.create(extra, "user1", doc! { "name": "union-deleted" }).await.expect("create union deleted");
    db.create(extra, "user2", doc! { "name": "union-other" }).await.expect("create union other");
    db.delete(extra, &deleted_extra.id, Some(&Scope::owner("user1"))).await.expect("delete union");

    let result = db
        .aggregate(
            root,
            vec![
                doc! { "$project": { "name": 1, "_owner": 1 } },
                doc! { "$unionWith": { "coll": extra, "pipeline": [ { "$project": { "name": 1, "_owner": 1 } } ] } },
                doc! { "$sort": { "name": 1 } },
            ],
            0,
            20,
            Some(&Scope::owner("user1")),
        )
        .await
        .expect("aggregate unionWith");

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].get_str("name").unwrap(), "root-visible");
    assert_eq!(result[1].get_str("name").unwrap(), "union-visible");
    assert!(result.iter().all(|row| row.get_str("_owner").unwrap() == "user1"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn aggregate_facet_rewrites_nested_scope_and_deleted_filters() {
    let mongo = TestMongo::start("k2db_rust_aggregate_facet", true).await;
    let db = &mongo.db;
    let collection = "agg_facet_items";

    db.create(collection, "user1", doc! { "name": "a" }).await.expect("create a");
    db.create(collection, "user1", doc! { "name": "b" }).await.expect("create b");
    let deleted = db.create(collection, "user1", doc! { "name": "c" }).await.expect("create c");
    db.create(collection, "user2", doc! { "name": "d" }).await.expect("create d");
    db.delete(collection, &deleted.id, Some(&Scope::owner("user1"))).await.expect("delete c");

    let result = db
        .aggregate(
            collection,
            vec![doc! {
                "$facet": {
                    "kept": [
                        { "$project": { "name": 1, "_owner": 1 } },
                        { "$sort": { "name": 1 } }
                    ]
                }
            }],
            0,
            20,
            Some(&Scope::owner("user1")),
        )
        .await
        .expect("aggregate facet");

    assert_eq!(result.len(), 1);
    let kept = result[0].get_array("kept").unwrap();
    assert_eq!(kept.len(), 2);
    assert_eq!(kept[0].as_document().unwrap().get_str("name").unwrap(), "a");
    assert_eq!(kept[1].as_document().unwrap().get_str("name").unwrap(), "b");
    assert!(kept.iter().all(|row| row.as_document().unwrap().get_str("_owner").unwrap() == "user1"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn aggregate_graphlookup_rewrites_nested_scope_and_deleted_filters() {
    let mongo = TestMongo::start("k2db_rust_aggregate_graph", true).await;
    let db = &mongo.db;
    let collection = "agg_graph_items";

    let good_parent = db.create(collection, "user1", doc! { "name": "good-parent", "kind": "node" }).await.expect("good parent");
    let deleted_parent = db.create(collection, "user1", doc! { "name": "deleted-parent", "kind": "node" }).await.expect("deleted parent");
    let other_parent = db.create(collection, "user2", doc! { "name": "other-parent", "kind": "node" }).await.expect("other parent");
    db.delete(collection, &deleted_parent.id, Some(&Scope::owner("user1"))).await.expect("delete parent");

    db.create(collection, "user1", doc! { "name": "child-deleted", "kind": "child", "parent_uuid": &deleted_parent.id })
        .await
        .expect("child deleted parent");
    db.create(collection, "user1", doc! { "name": "child-good", "kind": "child", "parent_uuid": &good_parent.id })
        .await
        .expect("child good parent");
    db.create(collection, "user1", doc! { "name": "child-other", "kind": "child", "parent_uuid": &other_parent.id })
        .await
        .expect("child other parent");
    db.create(collection, "user2", doc! { "name": "child-root-other-owner", "kind": "child", "parent_uuid": &other_parent.id })
        .await
        .expect("child root other owner");

    let result = db
        .aggregate(
            collection,
            vec![
                doc! { "$match": { "kind": "child" } },
                doc! { "$graphLookup": { "from": collection, "startWith": "$parent_uuid", "connectFromField": "parent_uuid", "connectToField": "_uuid", "as": "ancestors" } },
                doc! { "$project": { "name": 1, "ancestors": 1 } },
                doc! { "$sort": { "name": 1 } },
            ],
            0,
            20,
            Some(&Scope::owner("user1")),
        )
        .await
        .expect("aggregate graphLookup");

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].get_str("name").unwrap(), "child-deleted");
    assert_eq!(result[0].get_array("ancestors").unwrap().len(), 0);
    assert_eq!(result[1].get_str("name").unwrap(), "child-good");
    let ancestors = result[1].get_array("ancestors").unwrap();
    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0].as_document().unwrap().get_str("name").unwrap(), "good-parent");
    assert_eq!(result[2].get_str("name").unwrap(), "child-other");
    assert_eq!(result[2].get_array("ancestors").unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ensure_indexes_and_create_index_match_contract() {
    let mongo = TestMongo::start("k2db_rust_indexes", false).await;
    let db = &mongo.db;
    let collection = "indexed_items";

    db.ensure_indexes(collection, EnsureIndexesOptions::default())
        .await
        .expect("ensure indexes");
    db.create_index(
        collection,
        doc! { "name": 1 },
        Some(IndexOptions::builder().name(Some("name_idx".to_owned())).build()),
    )
    .await
    .expect("custom index");

    let direct = mongodb::Client::with_uri_str(format!("mongodb://127.0.0.1:{}/", db.config().hosts[0].port.unwrap()))
        .await
        .expect("direct client");
    let coll = direct.database(&db.config().name).collection::<Document>(collection);
    let indexes = coll
        .list_indexes()
        .await
        .expect("list indexes")
        .try_collect::<Vec<_>>()
        .await
        .expect("collect indexes");

    assert!(indexes.iter().any(|index| index.keys == doc! { "_uuid": 1, "_deleted": 1 } && index.options.as_ref().and_then(|opts| opts.unique) == Some(true)));
    assert!(indexes.iter().any(|index| index.keys == doc! { "_owner": 1 }));
    assert!(indexes.iter().any(|index| index.keys == doc! { "_deleted": 1 }));
    assert!(indexes.iter().any(|index| index.keys == doc! { "name": 1 } && index.options.as_ref().and_then(|opts| opts.name.as_deref()) == Some("name_idx")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drop_collection_enforces_scope_contract() {
    let strict = TestMongo::start("k2db_rust_drop_strict", true).await;
    strict
        .db
        .create("drop_items", "owner1", doc! { "name": "x" })
        .await
        .expect("create strict item");

    let missing_scope = strict
        .db
        .drop_collection("drop_items", None)
        .await
        .expect_err("strict drop should require scope");
    assert_eq!(missing_scope.key.as_deref(), Some("sys_mdb_drop_scope_required"));

    let owner_scope = strict
        .db
        .drop_collection("drop_items", Some(&Scope::owner("owner1")))
        .await
        .expect_err("strict drop should reject owner scope");
    assert_eq!(owner_scope.key.as_deref(), Some("sys_mdb_drop_scope_required"));

    let ok = strict
        .db
        .drop_collection("drop_items", Some(&Scope::all()))
        .await
        .expect("strict all-scope drop");
    assert_eq!(ok.status, "ok");

    let lax = TestMongo::start("k2db_rust_drop_lax", false).await;
    lax.db
        .create("drop_items", "owner1", doc! { "name": "x" })
        .await
        .expect("create lax item");

    let invalid = lax
        .db
        .drop_collection("drop_items", Some(&Scope::owner("owner1")))
        .await
        .expect_err("lax drop should reject owner scope");
    assert_eq!(invalid.key.as_deref(), Some("sys_mdb_drop_scope_invalid"));

    let ok = lax.db.drop_collection("drop_items", None).await.expect("lax drop without scope");
    assert_eq!(ok.status, "ok");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drop_database_removes_collections() {
    let mongo = TestMongo::start("k2db_rust_drop_db", false).await;
    let db = &mongo.db;

    db.create("db_items", "owner1", doc! { "name": "x" })
        .await
        .expect("create db item");
    db.drop_database().await.expect("drop database");

    let direct = mongodb::Client::with_uri_str(format!("mongodb://127.0.0.1:{}/", db.config().hosts[0].port.unwrap()))
        .await
        .expect("direct client");
    let names = direct
        .database(&db.config().name)
        .list_collection_names()
        .await
        .expect("list collection names");

    assert!(names.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn observability_hooks_and_ratatouille_logger_match_contract() {
    let before_calls: Arc<Mutex<Vec<(String, serde_json::Value)>>> = Arc::new(Mutex::new(Vec::new()));
    let after_calls: Arc<Mutex<Vec<(String, serde_json::Value, u64)>>> = Arc::new(Mutex::new(Vec::new()));
    let hooks = k2db::QueryHooks::default()
        .with_before_query({
            let before_calls = before_calls.clone();
            move |op, details| {
                before_calls
                    .lock()
                    .expect("before hook lock")
                    .push((op.to_owned(), details.clone()));
            }
        })
        .with_after_query({
            let after_calls = after_calls.clone();
            move |op, details, duration_ms| {
                after_calls
                    .lock()
                    .expect("after hook lock")
                    .push((op.to_owned(), details.clone(), duration_ms));
            }
        });

    let mongo = TestMongo::start_with_config(DatabaseConfig {
        name: "k2db_rust_observability".to_owned(),
        hosts: vec![HostConfig {
            host: "127.0.0.1".to_owned(),
            port: None,
        }],
        user: None,
        password: None,
        auth_source: None,
        replica_set: None,
        slow_query_ms: Some(0),
        ownership_mode: OwnershipMode::Lax,
        aggregation_mode: Default::default(),
        secure_field_prefixes: vec!["#".to_owned()],
        secure_field_encryption: None,
        hooks,
    })
    .await;
    let db = &mongo.db;

    let lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let logger = k2db::RatatouilleLogger::with_sink(
        k2db::ratatouille::LoggerConfig {
            filter: Some("*".to_owned()),
            format: k2db::ratatouille::Format::Ndjson,
            ..k2db::ratatouille::LoggerConfig::default()
        },
        k2db::ratatouille::FnSink::new({
            let lines = lines.clone();
            move |line| {
                lines.lock().expect("logger lock").push(line.to_owned());
            }
        }),
    );
    db.set_logger(logger.clone())
    .expect("set logger");

    let created = db
        .create("obs_items", "owner1", doc! { "name": "alpha" })
        .await
        .expect("create obs item");

    for seq in 0..32 {
        db.create("obs_items", "owner1", doc! { "name": format!("alpha-{seq}") })
            .await
            .expect("create extra obs item");
    }

    let fetched = db
        .find_one("obs_items", doc! { "_uuid": &created.id }, None, None)
        .await
        .expect("find one for hooks");
    assert!(fetched.is_some());

    let err = db
        .aggregate("obs_items", vec![], 0, 10, None)
        .await
        .expect_err("empty pipeline should emit error event");
    assert_eq!(err.key.as_deref(), Some("sys_mdb_ag_empty"));

    let before_calls = before_calls.lock().expect("before calls");
    assert!(before_calls.iter().any(|(op, _)| op == "insertOne"));
    assert!(before_calls.iter().any(|(op, _)| op == "findOne"));

    let after_calls = after_calls.lock().expect("after calls");
    assert!(after_calls.iter().any(|(op, details, _)| op == "insertOne" && details.get("ok") == Some(&serde_json::Value::Bool(true))));
    assert!(after_calls.iter().any(|(op, details, _)| op == "findOne" && details.get("ok") == Some(&serde_json::Value::Bool(true))));

    let joined = lines.lock().expect("lines").join("\n");
    assert!(joined.contains("k2db:debug"), "{joined}");
    assert!(joined.contains("k2db:slow_query"), "{joined}");
    assert!(joined.contains("k2db:error"), "{joined}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn schema_strip_and_passthrough_modes_match_contract() {
    let mongo = TestMongo::start("k2db_rust_schema_modes", false).await;
    let db = &mongo.db;

    db.set_schema(
        "strip_items",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        }),
        SchemaMode::Strip,
    )
    .expect("set strip schema");

    let strip_id = db
        .create("strip_items", "owner", doc! { "name": "ok", "extra": true })
        .await
        .expect("create strip item")
        .id;
    let strip_doc = db.get("strip_items", &strip_id, None).await.expect("get strip item");
    assert_eq!(strip_doc.get_str("name").unwrap(), "ok");
    assert!(strip_doc.get("extra").is_none());

    db.set_schema(
        "passthrough_items",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        }),
        SchemaMode::Passthrough,
    )
    .expect("set passthrough schema");

    let pass_id = db
        .create("passthrough_items", "owner", doc! { "name": "ok", "extra": true })
        .await
        .expect("create passthrough item")
        .id;
    let pass_doc = db.get("passthrough_items", &pass_id, None).await.expect("get passthrough item");
    assert_eq!(pass_doc.get_bool("extra").unwrap(), true);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn schema_strict_and_partial_update_match_contract() {
    let mongo = TestMongo::start("k2db_rust_schema_strict", false).await;
    let db = &mongo.db;

    db.set_schema(
        "strict_items",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "count": { "type": "integer" }
            },
            "required": ["name", "count"]
        }),
        SchemaMode::Strict,
    )
    .expect("set strict schema");

    let strict_error = db
        .create("strict_items", "owner", doc! { "name": "bad", "count": 1, "extra": true })
        .await
        .expect_err("strict schema should reject unknown field");
    assert_eq!(strict_error.key.as_deref(), Some("sys_mdb_schema_validation"));

    let created = db
        .create("strict_items", "owner", doc! { "name": "ok", "count": 1 })
        .await
        .expect("create strict item");

    db.update("strict_items", &created.id, doc! { "name": "patched" }, false, None)
        .await
        .expect("partial update should allow missing required fields");

    let replaced_error = db
        .update("strict_items", &created.id, doc! { "name": "replaced" }, true, None)
        .await
        .expect_err("replace should still require all required fields");
    assert_eq!(replaced_error.key.as_deref(), Some("sys_mdb_schema_validation"));

    db.update_all("strict_items", doc! { "_uuid": &created.id }, doc! { "count": 2 }, None)
        .await
        .expect("update all partial should allow missing required fields");

    let after = db.get("strict_items", &created.id, None).await.expect("get strict item");
    assert_eq!(after.get_str("name").unwrap(), "patched");
    assert_eq!(after.get_i32("count").unwrap(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn schema_non_object_keeps_partial_behavior_unchanged() {
    let mongo = TestMongo::start("k2db_rust_schema_scalar", false).await;
    let db = &mongo.db;

    let created = db
        .create("scalar_items", "owner", doc! { "name": "before" })
        .await
        .expect("create before schema");

    db.set_schema("scalar_items", json!({ "type": "string" }), SchemaMode::Strip)
        .expect("set scalar schema");

    let patch_error = db
        .update("scalar_items", &created.id, doc! { "name": "after" }, false, None)
        .await
        .expect_err("non-object schema should reject patch object too");
    assert_eq!(patch_error.key.as_deref(), Some("sys_mdb_schema_validation"));

    db.clear_schema("scalar_items").expect("clear schema");
    db.update("scalar_items", &created.id, doc! { "name": "after" }, false, None)
        .await
        .expect("update should succeed after clear_schema");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transaction_wrapper_commits_on_success_and_aborts_on_failure() {
    let mongo = TestMongo::start_replica_set(DatabaseConfig {
        name: "k2db_rust_txn".to_owned(),
        hosts: vec![HostConfig {
            host: "127.0.0.1".to_owned(),
            port: None,
        }],
        user: None,
        password: None,
        auth_source: None,
        replica_set: None,
        slow_query_ms: Some(1),
        ownership_mode: OwnershipMode::Lax,
        aggregation_mode: Default::default(),
        secure_field_prefixes: vec!["#".to_owned()],
        secure_field_encryption: None,
        hooks: k2db::QueryHooks::default(),
    })
    .await;
    let db = &mongo.db;

    db.execute_transaction(|mut txn| {
        Box::pin(async move {
            txn.create("txn_items", "owner1", doc! { "value": 1 }).await?;
            Ok(())
        })
    })
    .await
    .expect("transaction should commit");

    let direct = mongodb::Client::with_uri_str(format!(
        "mongodb://127.0.0.1:{}/?replicaSet=rs0",
        db.config().hosts[0].port.unwrap()
    ))
    .await
    .expect("direct txn client");
    let committed = direct
        .database(&db.config().name)
        .collection::<Document>("txn_items")
        .find_one(doc! { "value": 1, "_owner": "owner1" })
        .await
        .expect("find committed");
    assert!(committed.is_some());

    let aborted = db
        .execute_transaction(|mut txn| {
            Box::pin(async move {
                txn.create("txn_items", "owner1", doc! { "value": 2 }).await?;
                Err::<(), K2DbError>(K2DbError::new(
                    k2db::ServiceError::BadRequest,
                    "boom",
                    Some("txn_boom".to_owned()),
                ))
            })
        })
        .await
        .expect_err("transaction should abort");
    assert_eq!(aborted.key.as_deref(), Some("sys_mdb_txn"));

    let rolled_back = direct
        .database(&db.config().name)
        .collection::<Document>("txn_items")
        .find_one(doc! { "value": 2, "_owner": "owner1" })
        .await
        .expect("find aborted");
    assert!(rolled_back.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scoped_transaction_enforces_owner_scope() {
    let mongo = TestMongo::start_replica_set(DatabaseConfig {
        name: "k2db_rust_txn_scope".to_owned(),
        hosts: vec![HostConfig {
            host: "127.0.0.1".to_owned(),
            port: None,
        }],
        user: None,
        password: None,
        auth_source: None,
        replica_set: None,
        slow_query_ms: Some(1),
        ownership_mode: OwnershipMode::Strict,
        aggregation_mode: Default::default(),
        secure_field_prefixes: vec!["#".to_owned()],
        secure_field_encryption: None,
        hooks: k2db::QueryHooks::default(),
    })
    .await;
    let db = &mongo.db;
    let collection = "txn_scoped_items";

    let user1 = db.create(collection, "user1", doc! { "name": "u1" }).await.expect("create user1");
    let user2 = db.create(collection, "user2", doc! { "name": "u2" }).await.expect("create user2");

    let scoped = db.with_scope(Scope::owner("user1"));
    scoped
        .execute_transaction(|mut txn| {
            Box::pin(async move {
                let own = txn.get(collection, &user1.id).await?;
                assert_eq!(own.get_str("_owner").unwrap(), "user1");

                let denied = txn.get(collection, &user2.id).await.expect_err("cross-owner transaction read should fail");
                assert_eq!(denied.key.as_deref(), Some("sys_mdb_get_not_found"));

                let visible = txn.find(collection, doc! {}, FindOptions::default()).await?;
                assert_eq!(visible.len(), 1);
                assert_eq!(visible[0].get_str("_owner").unwrap(), "user1");
                Ok::<(), K2DbError>(())
            })
        })
        .await
        .expect("scoped transaction should inherit scope");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn typed_operation_api_round_trip() {
    let mongo = TestMongo::start("k2db_rust_typed_ops", true).await;
    let db = &mongo.db;

    let created = db
        .create_with(CreateOp {
            collection: "typed_items".to_owned(),
            owner: "user1".to_owned(),
            data: doc! { "name": "alpha", "seq": 1 },
        })
        .await
        .expect("typed create");

    let scoped = db.with_scope(Scope::owner("user1"));
    let fetched = scoped
        .get_with(k2db::GetOp {
            collection: "typed_items".to_owned(),
            id: created.id.clone(),
            scope: None,
        })
        .await
        .expect("typed get");
    assert_eq!(fetched.get_str("name").unwrap(), "alpha");

    let one = scoped
        .find_one_with(FindOneOp {
            collection: "typed_items".to_owned(),
            criteria: doc! { "_uuid": &created.id },
            fields: Some(vec!["name".to_owned()]),
            scope: None,
        })
        .await
        .expect("typed find one")
        .expect("typed doc");
    assert_eq!(one, doc! { "name": "alpha" });

    scoped
        .update_with(UpdateOneOp {
            collection: "typed_items".to_owned(),
            id: created.id.clone(),
            data: doc! { "name": "beta" },
            replace: false,
            scope: None,
        })
        .await
        .expect("typed update");

    let listed = scoped
        .find_with(FindOp {
            collection: "typed_items".to_owned(),
            criteria: doc! {},
            options: FindOptions {
                projection: ProjectionMode::Include(vec!["name".to_owned()]),
                ..FindOptions::default()
            },
            scope: None,
        })
        .await
        .expect("typed find");
    assert_eq!(listed, vec![doc! { "name": "beta" }]);

    let count = scoped
        .count_with(CountOp {
            collection: "typed_items".to_owned(),
            criteria: doc! {},
            scope: None,
        })
        .await
        .expect("typed count");
    assert_eq!(count.count, 1);

    let aggregated = scoped
        .aggregate_with(AggregateOp {
            collection: "typed_items".to_owned(),
            pipeline: vec![doc! { "$project": { "name": 1 } }],
            skip: 0,
            limit: 10,
            scope: None,
        })
        .await
        .expect("typed aggregate");
    assert_eq!(aggregated, vec![doc! { "name": "beta" }]);
}