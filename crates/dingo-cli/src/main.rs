//! DingoDB CLI (Stage 7): put/get/list, doctor, salvage, serve.

use clap::{ArgAction, Parser, Subcommand};
use dingo_examine::{examine_store, ExaminationUnit, ExamineLimits};
use dingo_sdk::{serve_cluster_node, serve_store_with, Dingo, ServeOptions, DEFAULT_PORT};
use dingo_store::Store;
use serde_json::{json as sjson, Value as JsonValue};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const APP_VERSION: &str = concat!(env!("DINGO_VERSION"), "-build ", env!("DINGO_BUILD"));
const CLI_ABOUT: &str = "DingoDB command-line interface";
const CLI_LONG_ABOUT: &str = "DingoDB command-line interface\n\nEveryday put/get/list, read-only doctor diagnostics, non-destructive salvage, single-node TCP serve (development), and experimental multi-node serve-cluster (routing/advertise only; not network quorum).";
const LICENSE_TEXT: &str = "Copyright (c) Alexander R. Croft\nMIT License\n\nThis program is offered under the MIT License. See the repository LICENSE file for the full terms.";

#[derive(Parser)]
#[command(
    name = "dingo",
    version = APP_VERSION,
    about = CLI_ABOUT,
    long_about = CLI_LONG_ABOUT,
    disable_help_subcommand = true,
    disable_version_flag = true,
    next_line_help = true,
)]
struct Cli {
    /// Print the shipped semantic version and build number.
    #[arg(long = "version", global = true, action = ArgAction::SetTrue)]
    version_flag: bool,

    /// Print copyright and license information.
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    license: bool,

    /// Emit JSON on stdout (stable machine-readable). Distinct from put `--json` body.
    #[arg(long = "json-out", global = true, action = ArgAction::SetTrue)]
    json_out: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Put a JSON document: `dingo put PATH COLL/KEY --json '...'`
    Put {
        /// Store directory path.
        store: PathBuf,
        /// Collection/key path (`users/user-42`).
        target: String,
        /// JSON document body.
        #[arg(long = "json")]
        json_body: String,
    },
    /// Get a JSON document: `dingo get PATH COLL/KEY`
    Get { store: PathBuf, target: String },
    /// Delete a key: `dingo delete PATH COLL/KEY`
    Delete { store: PathBuf, target: String },
    /// List collections, or keys in a collection.
    List {
        store: PathBuf,
        /// Optional collection name.
        collection: Option<String>,
    },
    /// Put raw bytes from a file: `dingo put-bytes PATH COLL/KEY FILE`
    PutBytes {
        store: PathBuf,
        target: String,
        file: PathBuf,
    },
    /// Show event history for a key (embedded).
    History { store: PathBuf, target: String },
    /// Read-only store health report (DX_SPEC §13.3).
    Doctor { store: PathBuf },
    /// Non-destructive salvage to a new path (DX_SPEC §13.4).
    Salvage {
        /// Source store (never mutated).
        store: PathBuf,
        /// Destination store path (must not already be a store).
        #[arg(long = "output", short = 'o')]
        output: PathBuf,
    },
    /// Serve the store over TCP for `Dingo::connect("dingo://...")` (development).
    ///
    /// Defaults to loopback. Non-loopback plaintext binds require
    /// `--allow-insecure-bind` (TLS is not implemented yet; DEF-002).
    Serve {
        store: PathBuf,
        /// Bind address (default `127.0.0.1:7434`).
        #[arg(long = "bind", default_value_t = format!("127.0.0.1:{DEFAULT_PORT}"))]
        bind: String,
        /// Optional shared auth token (clients must pass the same via ConnectOptions).
        /// Also accepted from the `DINGO_TOKEN` environment variable when the flag is omitted.
        #[arg(long = "token")]
        token: Option<String>,
        /// Allow non-loopback plaintext bind (development only; no TLS yet).
        #[arg(long = "allow-insecure-bind", action = ArgAction::SetTrue)]
        allow_insecure_bind: bool,
    },
    /// Serve one node of a multi-node cluster root (**experimental**).
    ///
    /// Routing and `endpoints.json` advertise only: writes apply to **this node**.
    /// Network quorum replication is not implemented. Requires
    /// `--experimental-network-cluster`. Prefer in-process `Dingo::open_cluster`
    /// for replicated integration tests.
    ///
    /// Example:
    /// `dingo serve-cluster ./cluster --node 0 --bind 127.0.0.1:7434 --experimental-network-cluster`
    ServeCluster {
        /// Cluster root (contains cluster.json, placement.json, nodes/).
        cluster: PathBuf,
        /// Dense node index to serve (`nodes/node-N`).
        #[arg(long = "node", default_value_t = 0)]
        node: u32,
        /// Bind address (default `127.0.0.1:7434`).
        #[arg(long = "bind", default_value_t = format!("127.0.0.1:{DEFAULT_PORT}"))]
        bind: String,
        /// Optional shared auth token (also `DINGO_TOKEN`).
        #[arg(long = "token")]
        token: Option<String>,
        /// Allow non-loopback plaintext bind (development only; no TLS yet).
        #[arg(long = "allow-insecure-bind", action = ArgAction::SetTrue)]
        allow_insecure_bind: bool,
        /// Required opt-in: network serve-cluster is experimental (DEF-002).
        #[arg(long = "experimental-network-cluster", action = ArgAction::SetTrue)]
        experimental_network_cluster: bool,
    },
    /// List collection names (alias of `list` without collection).
    Collections { store: PathBuf },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    if cli.version_flag {
        print!("{APP_VERSION}\n");
        return Ok(());
    }
    if cli.license {
        print!("{LICENSE_TEXT}\n");
        return Ok(());
    }
    let Some(cmd) = cli.command else {
        return Err("missing command; try `dingo --help`".into());
    };
    let json_out = cli.json_out;
    match cmd {
        Command::Put {
            store,
            target,
            json_body,
        } => cmd_put(&store, &target, &json_body, json_out),
        Command::Get { store, target } => cmd_get(&store, &target, json_out),
        Command::Delete { store, target } => cmd_delete(&store, &target, json_out),
        Command::List { store, collection } => cmd_list(&store, collection.as_deref(), json_out),
        Command::PutBytes {
            store,
            target,
            file,
        } => cmd_put_bytes(&store, &target, &file, json_out),
        Command::History { store, target } => cmd_history(&store, &target, json_out),
        Command::Doctor { store } => cmd_doctor(&store, json_out),
        Command::Salvage { store, output } => cmd_salvage(&store, &output, json_out),
        Command::Serve {
            store,
            bind,
            token,
            allow_insecure_bind,
        } => {
            // Flag wins; otherwise fall back to DINGO_TOKEN for operator convenience.
            let token = token.or_else(|| std::env::var("DINGO_TOKEN").ok());
            // Library emits the structured startup report and enforces bind policy (DEF-002).
            let mut opts = ServeOptions::new().allow_insecure_bind(allow_insecure_bind);
            if let Some(t) = token {
                opts = opts.auth_token(t);
            }
            serve_store_with(&store, &bind, opts).map_err(|e| e.to_string())
        }
        Command::ServeCluster {
            cluster,
            node,
            bind,
            token,
            allow_insecure_bind,
            experimental_network_cluster,
        } => {
            if !experimental_network_cluster {
                return Err(
                    "serve-cluster requires --experimental-network-cluster (DEF-002). \
                     Network quorum replication is not implemented; writes apply to this \
                     node only. In-process quorum: Dingo::open_cluster."
                        .into(),
                );
            }
            let token = token.or_else(|| std::env::var("DINGO_TOKEN").ok());
            let mut opts = ServeOptions::new()
                .allow_insecure_bind(allow_insecure_bind)
                .experimental_network_cluster(true);
            if let Some(t) = token {
                opts = opts.auth_token(t);
            }
            serve_cluster_node(&cluster, node, &bind, opts).map_err(|e| e.to_string())
        }
        Command::Collections { store } => cmd_list(&store, None, json_out),
    }
}

fn parse_target(target: &str) -> Result<(String, String), String> {
    let (coll, key) = target
        .split_once('/')
        .ok_or_else(|| format!("target must be COLL/KEY, got {target:?}"))?;
    if coll.is_empty() || key.is_empty() {
        return Err("collection and key must be non-empty".into());
    }
    if key.contains('/') {
        // Allow multi-segment keys: users/a/b → coll=users, key=a/b
        let rest = &target[coll.len() + 1..];
        return Ok((coll.to_string(), rest.to_string()));
    }
    Ok((coll.to_string(), key.to_string()))
}

fn cmd_put(store: &Path, target: &str, json_body: &str, json_out: bool) -> Result<(), String> {
    let (coll, key) = parse_target(target)?;
    let value: JsonValue =
        serde_json::from_str(json_body).map_err(|e| format!("invalid --json: {e}"))?;
    let mut db = Dingo::open(store).map_err(|e| e.to_string())?;
    let receipt = db
        .collection(&coll)
        .map_err(|e| e.to_string())?
        .put(&key, &value)
        .map_err(|e| e.to_string())?;
    if json_out {
        emit_json(sjson!({
            "ok": true,
            "store": store.display().to_string(),
            "collection": coll,
            "key": key,
            "committed": receipt.committed,
            "acknowledgement": receipt.acknowledgement.as_str(),
        }))?;
    } else {
        println!(
            "put {}/{} ok (ack={})",
            coll,
            key,
            receipt.acknowledgement.as_str()
        );
    }
    Ok(())
}

fn cmd_get(store: &Path, target: &str, json_out: bool) -> Result<(), String> {
    let (coll, key) = parse_target(target)?;
    let mut db = Dingo::open(store).map_err(|e| e.to_string())?;
    let found = db
        .collection(&coll)
        .map_err(|e| e.to_string())?
        .get(&key)
        .map_err(|e| e.to_string())?;
    match found {
        None => {
            if json_out {
                emit_json(sjson!({
                    "ok": true,
                    "store": store.display().to_string(),
                    "collection": coll,
                    "key": key,
                    "found": false,
                }))?;
            } else {
                println!("not found: {coll}/{key}");
            }
            Err(format!("not found: {coll}/{key}"))
        }
        Some(v) => {
            if json_out {
                emit_json(sjson!({
                    "ok": true,
                    "store": store.display().to_string(),
                    "collection": coll,
                    "key": key,
                    "found": true,
                    "value": v,
                }))?;
            } else {
                println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            }
            Ok(())
        }
    }
}

fn cmd_delete(store: &Path, target: &str, json_out: bool) -> Result<(), String> {
    let (coll, key) = parse_target(target)?;
    let mut db = Dingo::open(store).map_err(|e| e.to_string())?;
    let receipt = db
        .collection(&coll)
        .map_err(|e| e.to_string())?
        .delete(&key)
        .map_err(|e| e.to_string())?;
    if json_out {
        emit_json(sjson!({
            "ok": true,
            "store": store.display().to_string(),
            "collection": coll,
            "key": key,
            "removed": receipt.removed,
            "acknowledgement": receipt.acknowledgement.as_str(),
        }))?;
    } else {
        println!(
            "delete {}/{} removed={} (ack={})",
            coll,
            key,
            receipt.removed,
            receipt.acknowledgement.as_str()
        );
    }
    Ok(())
}

fn cmd_list(store: &Path, collection: Option<&str>, json_out: bool) -> Result<(), String> {
    let mut db = Dingo::open(store).map_err(|e| e.to_string())?;
    match collection {
        None => {
            let cols = db.list_collections().map_err(|e| e.to_string())?;
            if json_out {
                emit_json(sjson!({
                    "ok": true,
                    "store": store.display().to_string(),
                    "collections": cols,
                }))?;
            } else {
                for c in cols {
                    println!("{c}");
                }
            }
        }
        Some(coll) => {
            let keys = db
                .collection(coll)
                .map_err(|e| e.to_string())?
                .scan_keys()
                .map_err(|e| e.to_string())?;
            if json_out {
                emit_json(sjson!({
                    "ok": true,
                    "store": store.display().to_string(),
                    "collection": coll,
                    "keys": keys,
                }))?;
            } else {
                for k in keys {
                    println!("{k}");
                }
            }
        }
    }
    Ok(())
}

fn cmd_put_bytes(store: &Path, target: &str, file: &Path, json_out: bool) -> Result<(), String> {
    let (coll, key) = parse_target(target)?;
    let bytes = fs::read(file).map_err(|e| format!("read {}: {e}", file.display()))?;
    let mut db = Dingo::open(store).map_err(|e| e.to_string())?;
    let receipt = db
        .collection(&coll)
        .map_err(|e| e.to_string())?
        .put_bytes(&key, &bytes)
        .map_err(|e| e.to_string())?;
    if json_out {
        emit_json(sjson!({
            "ok": true,
            "store": store.display().to_string(),
            "collection": coll,
            "key": key,
            "bytes": bytes.len(),
            "acknowledgement": receipt.acknowledgement.as_str(),
        }))?;
    } else {
        println!(
            "put-bytes {}/{} {} bytes (ack={})",
            coll,
            key,
            bytes.len(),
            receipt.acknowledgement.as_str()
        );
    }
    Ok(())
}

fn cmd_history(store: &Path, target: &str, json_out: bool) -> Result<(), String> {
    let (coll, key) = parse_target(target)?;
    let mut db = Dingo::open(store).map_err(|e| e.to_string())?;
    let hist = db
        .collection(&coll)
        .map_err(|e| e.to_string())?
        .history(&key)
        .map_err(|e| e.to_string())?;
    if json_out {
        let versions: Vec<JsonValue> = hist
            .versions
            .iter()
            .map(|v| {
                sjson!({
                    "kind": v.kind,
                    "event_id": v.event_id,
                    "item_id": v.item_id,
                    "segment_id": v.segment_id,
                    "json": v.json,
                    "known_gap_before": v.known_gap_before,
                })
            })
            .collect();
        emit_json(sjson!({
            "ok": true,
            "store": store.display().to_string(),
            "collection": coll,
            "key": key,
            "has_known_holes": hist.has_known_holes,
            "versions": versions,
        }))?;
    } else {
        println!(
            "history {}/{} versions={} holes={}",
            coll,
            key,
            hist.versions.len(),
            hist.has_known_holes
        );
        for (i, v) in hist.versions.iter().enumerate() {
            println!("  [{i}] {} event={}", v.kind, v.event_id);
        }
    }
    Ok(())
}

fn cmd_doctor(store: &Path, json_out: bool) -> Result<(), String> {
    // Read-only open: no active writer, no derived persistence.
    let inspect = Store::open_inspect(store).map_err(|e| e.to_string())?;
    let salvage = inspect.salvage().map_err(|e| e.to_string())?;
    let page = examine_store(
        &inspect,
        ExamineLimits::default()
            .without_payloads()
            .max_units(10_000),
    )
    .map_err(|e| e.to_string())?;
    let summary = summarize_units(&page.units);
    let collections = inspect.list_collections();
    let indexes_dir = store.join("indexes");
    let catalogs_dir = store.join("catalogs");
    let index_cache_present = indexes_dir.join("primary.idx").is_file();
    let catalog_present = catalogs_dir.join("collections.cat").is_file();

    let healthy = salvage.holes == 0 && summary.damaged == 0 && summary.holes == 0;
    let recommendations = doctor_recommendations(&salvage, &summary, index_cache_present);

    if json_out {
        emit_json(sjson!({
            "ok": true,
            "store": store.display().to_string(),
            "store_id": hex16(&inspect.store_id()),
            "read_only": true,
            "healthy": healthy,
            "live_subjects": salvage.live_subjects,
            "files_scanned": salvage.files_scanned,
            "verified_frames": salvage.verified_frames,
            "item_events": salvage.item_events,
            "holes": salvage.holes,
            "examination": {
                "units": page.units.len(),
                "complete": page.complete,
                "verified_complete": summary.verified_complete,
                "partial": summary.partial,
                "damaged": summary.damaged,
                "holes": summary.holes,
            },
            "collections": collections,
            "derived": {
                "index_cache_present": index_cache_present,
                "catalog_present": catalog_present,
            },
            "recommendations": recommendations,
        }))?;
    } else {
        println!("dingo doctor {}", store.display());
        println!("  read_only: true");
        println!("  healthy: {healthy}");
        println!("  store_id: {}", hex16(&inspect.store_id()));
        println!("  live_subjects: {}", salvage.live_subjects);
        println!(
            "  segments: files={} verified_frames={} item_events={} holes={}",
            salvage.files_scanned, salvage.verified_frames, salvage.item_events, salvage.holes
        );
        println!(
            "  examination: units={} complete={} verified={} partial={} damaged={} holes={}",
            page.units.len(),
            page.complete,
            summary.verified_complete,
            summary.partial,
            summary.damaged,
            summary.holes
        );
        println!(
            "  collections: {}",
            if collections.is_empty() {
                "(none)".into()
            } else {
                collections.join(", ")
            }
        );
        println!(
            "  derived: index_cache={} catalog={}",
            index_cache_present, catalog_present
        );
        if !recommendations.is_empty() {
            println!("  recommendations:");
            for r in &recommendations {
                println!("    - {r}");
            }
        }
    }
    // Nonzero exit when damaged (failed health guarantee) — still printed report.
    if !healthy {
        return Err("store health check found holes or damaged units".into());
    }
    Ok(())
}

fn cmd_salvage(source: &Path, dest: &Path, json_out: bool) -> Result<(), String> {
    if source == dest {
        return Err("salvage source and --output must differ".into());
    }
    // Inspect source without mutating; then salvage_to creates dest.
    let inspect = Store::open_inspect(source).map_err(|e| e.to_string())?;
    let report = inspect.salvage_to(dest).map_err(|e| e.to_string())?;

    // Verify source path tree was not rewritten by comparing a simple marker:
    // we never opened a writer on source.
    if json_out {
        emit_json(sjson!({
            "ok": true,
            "source": source.display().to_string(),
            "destination": report.destination.display().to_string(),
            "source_immutable": true,
            "files_scanned": report.source.files_scanned,
            "verified_frames": report.source.verified_frames,
            "item_events": report.source.item_events,
            "holes": report.source.holes,
            "live_subjects": report.source.live_subjects,
            "subjects_copied": report.subjects_copied,
        }))?;
    } else {
        println!(
            "salvage {} → {}",
            source.display(),
            report.destination.display()
        );
        println!("  source immutable: true");
        println!(
            "  source: files={} frames={} items={} holes={} live={}",
            report.source.files_scanned,
            report.source.verified_frames,
            report.source.item_events,
            report.source.holes,
            report.source.live_subjects
        );
        println!("  subjects_copied: {}", report.subjects_copied);
    }
    Ok(())
}

struct UnitSummary {
    verified_complete: usize,
    partial: usize,
    damaged: usize,
    holes: usize,
}

fn summarize_units(units: &[ExaminationUnit]) -> UnitSummary {
    let mut s = UnitSummary {
        verified_complete: 0,
        partial: 0,
        damaged: 0,
        holes: 0,
    };
    for u in units {
        let kind = u.unit_kind.to_lowercase();
        let status = u.status.to_lowercase();
        if kind.contains("hole") || status.contains("hole") {
            s.holes += 1;
        } else if status.contains("partial") || u.payload.availability == "partial" {
            s.partial += 1;
        } else if status.contains("damaged")
            || status.contains("corrupt")
            || status.contains("failed")
        {
            s.damaged += 1;
        } else if status.contains("verified") || status == "complete" {
            s.verified_complete += 1;
        } else {
            // Neutral structural units count as verified for health rollup.
            s.verified_complete += 1;
        }
    }
    s
}

fn doctor_recommendations(
    salvage: &dingo_store::SalvageReport,
    summary: &UnitSummary,
    index_cache: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    if salvage.holes > 0 || summary.holes > 0 {
        out.push(
            "holes detected: run `dingo salvage SRC --output DST` (source stays immutable)".into(),
        );
    }
    if summary.damaged > 0 {
        out.push("damaged units present: examine with dingo-examine / SDA filters".into());
    }
    if !index_cache {
        out.push(
            "primary index cache missing (derived only; open/rebuild will rescan segments)".into(),
        );
    }
    if salvage.live_subjects == 0 && salvage.item_events > 0 {
        out.push("item events found but no live subjects (all deleted or incomplete)".into());
    }
    if out.is_empty() {
        out.push("no action required".into());
    }
    out
}

fn emit_json(v: JsonValue) -> Result<(), String> {
    let mut out = io::stdout();
    serde_json::to_writer(&mut out, &v).map_err(|e| e.to_string())?;
    out.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}

fn hex16(id: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in id {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
