//! Stage 7 CLI integration tests: put/get/list, doctor, salvage, serve+connect.

use dingo_sdk::{client_handshake, read_frame, write_frame, DEFAULT_MAX_FRAME_BYTES};
use std::fs;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

/// Wait until a dingo serve process completes a framed handshake + ping.
///
/// A bare TCP connect is not enough: production servers (DEF-031) require the
/// `dingo-rpc-v1` hello/welcome exchange before application RPCs. Any framed
/// JSON response (including auth failure on a later store_info) means live.
fn wait_for_dingo_ping(bind: &str) {
    for _ in 0..100 {
        if let Ok(mut stream) = TcpStream::connect(bind) {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(400)));
            let _ = stream.set_write_timeout(Some(Duration::from_millis(400)));
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            if client_handshake(&mut reader, &mut stream).is_ok() {
                if write_frame(&mut stream, br#"{"id":1,"op":"ping"}"#).is_ok() {
                    if let Ok(Some(bytes)) = read_frame(&mut reader, DEFAULT_MAX_FRAME_BYTES) {
                        if bytes.contains(&b'{') {
                            return;
                        }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(30));
    }
    panic!("dingo serve did not answer ping on {bind}");
}

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
fn license_prints_notice() {
    let out = run_ok(&["--license"]);
    assert!(out.contains("Alexander R. Croft"), "license={out}");
    assert!(
        out.contains("AGPL") || out.contains("Affero"),
        "dingo CLI must advertise AGPL, license={out}"
    );
    assert!(
        !out.contains("MIT License") || out.contains("multi-licensed"),
        "dingo must not claim pure MIT, license={out}"
    );
}

#[test]
fn version_and_help() {
    let out = run_ok(&["--version"]);
    assert!(out.contains("-build"), "version={out}");
    let help = run_ok(&["--help"]);
    assert!(help.contains("doctor"));
    assert!(help.contains("salvage"));
    assert!(help.contains("serve"));
    assert!(help.contains("serve-cluster"));
    assert!(
        help.contains("experimental") || help.contains("development"),
        "top-level help should qualify serve maturity, help={help}"
    );
    let serve_help = run_ok(&["serve", "--help"]);
    assert!(
        serve_help.contains("allow-insecure-bind"),
        "serve help={serve_help}"
    );
    let sc_help = run_ok(&["serve-cluster", "--help"]);
    assert!(
        sc_help.contains("experimental-network-cluster"),
        "serve-cluster help={sc_help}"
    );
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
    assert!(report.contains("evidence"));
    assert!(report.contains("frames_copied:"));
    assert!(report.contains("manifest:"));

    // Source still readable via open (segments intact).
    let src_get = run_ok(&["get", src_s, "users/alice"]);
    assert!(src_get.contains("Alice"));

    // Destination has recovered values.
    let dst_get = run_ok(&["get", dst_s, "users/bob"]);
    assert!(dst_get.contains("Bob"));
}

#[test]
fn export_live_is_distinct_from_salvage() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.dingo");
    let dst = dir.path().join("export.dingo");
    let src_s = src.to_str().unwrap();
    let dst_s = dst.to_str().unwrap();

    run_ok(&["put", src_s, "users/alice", "--json", r#"{"name":"Alice"}"#]);
    let report = run_ok(&["export-live", src_s, "--output", dst_s]);
    assert!(report.contains("live_state_export") || report.contains("subjects_copied: 1"));
    assert!(report.contains("frames_copied: 0"));
    let dst_get = run_ok(&["get", dst_s, "users/alice"]);
    assert!(dst_get.contains("Alice"));
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

    wait_for_dingo_ping(&bind);

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

    // Stop the server so exclusive writer ownership is released (DEF-020),
    // then reopen embedded and confirm the remote write is durable.
    let _ = child.kill();
    let _ = child.wait();

    let mut local = dingo_sdk::Dingo::open(&store).unwrap();
    let v = local
        .collection("users")
        .unwrap()
        .get("remote")
        .unwrap()
        .unwrap();
    assert_eq!(v["from"], "sdk");
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

    wait_for_dingo_ping(&bind);

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

#[test]
fn serve_cluster_missing_root_fails_fast() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("no-such-cluster");
    let out = dingo_bin()
        .args([
            "serve-cluster",
            missing.to_str().unwrap(),
            "--node",
            "0",
            "--bind",
            "127.0.0.1:0",
            "--experimental-network-cluster",
        ])
        .output()
        .expect("run serve-cluster");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("cluster") || err.contains("error"),
        "stderr={err}"
    );
}

#[test]
fn serve_cluster_requires_experimental_flag() {
    let dir = tempdir().unwrap();
    let out = dingo_bin()
        .args([
            "serve-cluster",
            dir.path().to_str().unwrap(),
            "--node",
            "0",
            "--bind",
            "127.0.0.1:0",
        ])
        .output()
        .expect("run serve-cluster without flag");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("experimental-network-cluster") || err.contains("DEF-002"),
        "stderr={err}"
    );
}

#[test]
fn serve_refuses_public_plaintext_bind_without_override() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();
    // Create a store first so failure is about bind policy, not missing path.
    run_ok(&[
        "put",
        store_s,
        "t/k",
        "--json",
        r#"{"ok":true}"#,
    ]);
    let out = dingo_bin()
        .args(["serve", store_s, "--bind", "0.0.0.0:17434"])
        .output()
        .expect("run serve with public bind");
    assert!(!out.status.success(), "public bind must fail closed");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("non-loopback")
            || err.contains("allow-insecure-bind")
            || err.contains("DEF-002"),
        "stderr={err}"
    );
}

#[test]
fn serve_loopback_bind_is_allowed() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();
    run_ok(&["put", store_s, "t/k", "--json", r#"{"ok":true}"#]);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let bind = format!("127.0.0.1:{port}");

    let mut child = dingo_bin()
        .args(["serve", store_s, "--bind", &bind])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn dingo serve");
    wait_for_dingo_ping(&bind);

    // Startup report must not claim network quorum durability.
    // Kill after a successful ping; capture any stderr already written.
    let _ = child.kill();
    let output = child.wait_with_output().expect("wait serve");
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(
        err.contains("startup status") || err.contains("replication: none"),
        "expected structured startup report, stderr={err}"
    );
    assert!(
        !err.to_lowercase().contains("replicated durability"),
        "must not claim replicated durability, stderr={err}"
    );
}

#[test]
fn capability_matrix_document_present() {
    // DEF-001: release-facing capability matrix must stay in-tree.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let matrix = root.join("doc/CAPABILITY_MATRIX.md");
    let text = fs::read_to_string(&matrix).expect("doc/CAPABILITY_MATRIX.md must exist");
    assert!(
        text.contains("Three `serve-cluster` processes do not provide replicated durability")
            || text.contains("do not provide replicated durability"),
        "matrix must forbid multi-process replicated-durability inference"
    );
    assert!(text.contains("DEF-002") || text.contains("allow-insecure-bind"));
    let readme = fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("Not production-ready")
            || readme.contains("not production-ready")
            || readme.contains("not yet production-ready"),
        "README must state production maturity honestly"
    );
    assert!(
        !readme.contains("Extreme speed"),
        "README must not lead with unqualified extreme-speed marketing"
    );
}

#[test]
fn serve_public_bind_allowed_with_insecure_override() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("app.dingo");
    let store_s = store.to_str().unwrap();
    run_ok(&["put", store_s, "t/k", "--json", r#"{"ok":true}"#]);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    // Bind all interfaces on an ephemeral port with explicit override (DEF-002).
    let bind = format!("0.0.0.0:{port}");

    let mut child = dingo_bin()
        .args([
            "serve",
            store_s,
            "--bind",
            &bind,
            "--allow-insecure-bind",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn dingo serve with insecure bind");
    // Connect via loopback to the same port.
    let connect = format!("127.0.0.1:{port}");
    wait_for_dingo_ping(&connect);
    let _ = child.kill();
    let output = child.wait_with_output().expect("wait serve");
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(
        err.contains("allow_insecure_bind: yes") || err.contains("development override"),
        "stderr={err}"
    );
}

#[test]
fn serve_cluster_advertises_placement_and_endpoints() {
    use dingo_sdk::{serve_cluster_node, ClusterConfig, RemoteClient, ServeOptions};

    let dir = tempdir().unwrap();
    let root = dir.path().join("c");
    // Build a 3-node cluster on disk, then serve node 0 in-process over TCP.
    let mut db = dingo_sdk::Dingo::create_cluster(
        ClusterConfig::dependable_local(&root).with_virtual_partitions(8),
    )
    .expect("create cluster");
    db.collection("users")
        .unwrap()
        .put("seed", &serde_json::json!({"n": 1}))
        .unwrap();
    drop(db);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let bind = format!("127.0.0.1:{port}");

    let root_thread = root.clone();
    let bind_thread = bind.clone();
    let handle = thread::spawn(move || {
        let _ = serve_cluster_node(
            &root_thread,
            0,
            &bind_thread,
            ServeOptions::new().experimental_network_cluster(true),
        );
    });

    wait_for_dingo_ping(&bind);

    let url = format!("dingo://{bind}/c");
    let mut client = RemoteClient::connect(&bind, url.clone()).expect("connect to cluster node");
    let snap = client.fetch_directory().expect("directory");
    assert_eq!(snap.virtual_partitions, 8);
    assert_eq!(snap.assignments.len(), 8);
    assert!(snap.assignments.iter().all(|a| a.replicas.len() == 3));
    assert_eq!(
        snap.endpoints.get(&0).map(String::as_str),
        Some(bind.as_str()),
        "endpoints={:?}",
        snap.endpoints
    );
    // Ping/get on the same open connection (server is single-threaded).
    assert!(client.store_info().is_ok());
    drop(client);

    let ep = fs::read_to_string(root.join("endpoints.json")).unwrap();
    assert!(ep.contains(&bind));

    // Second client after the first connection is closed.
    let mut db = dingo_sdk::Dingo::connect(&url).unwrap();
    assert!(db.is_remote());
    let _ = db.collection("users").unwrap().get("seed");
    drop(db);

    // serve_cluster_node loops forever; thread ends with the test process.
    let _ = handle.thread().id();
}
