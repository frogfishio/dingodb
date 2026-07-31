//! Multi-process campaign child (DEF-041-N).
//!
//! Environment:
//! - `RESIDIUUM_MP_STORE` — store directory (required; created if missing)
//! - `RESIDIUUM_MP_HISTORY` — path to write MultiprocHistory JSON (required)
//! - `RESIDIUUM_MP_SEED` — u64 seed for diagnostic dump (default 1)
//! - `RESIDIUUM_MP_OPS` — number of durable puts (default 8)
//! - `RESIDIUUM_MP_ABORT_AFTER` — if set to N, `process::abort` after N successful acks
//! - `RESIDIUUM_MP_PREFIX` — key prefix (default `mp/`)
//! - `RESIDIUUM_MP_MODE` — `put_series` (default) | `try_open_only`
//!
//! Exit codes: 0 success, 2 usage, 3 store open/create, 4 write failure,
//! 6 writer lock held (try_open_only / open contention).

use residiuum_cluster::multiproc::{hex16, MultiprocHistory, MultiprocOp};
use residiuum_store::{DurabilityMode, Store, StoreError};
use std::env;
use std::path::PathBuf;
use std::process::{abort, ExitCode};

fn main() -> ExitCode {
    let store = match env::var_os("RESIDIUUM_MP_STORE") {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!("RESIDIUUM_MP_STORE required");
            return ExitCode::from(2);
        }
    };
    let history_path = match env::var_os("RESIDIUUM_MP_HISTORY") {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!("RESIDIUUM_MP_HISTORY required");
            return ExitCode::from(2);
        }
    };
    let seed: u64 = env::var("RESIDIUUM_MP_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let ops: u64 = env::var("RESIDIUUM_MP_OPS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    let abort_after: Option<u64> = env::var("RESIDIUUM_MP_ABORT_AFTER")
        .ok()
        .and_then(|s| s.parse().ok());
    let prefix = env::var("RESIDIUUM_MP_PREFIX").unwrap_or_else(|_| "mp/".into());
    let mode = env::var("RESIDIUUM_MP_MODE").unwrap_or_else(|_| "put_series".into());

    let mut history = MultiprocHistory::new(seed, store.display().to_string());
    history.note(format!("mode={mode} ops={ops} abort_after={abort_after:?}"));

    if mode == "try_open_only" {
        match Store::open(&store) {
            Ok(_) => {
                history.note("open_ok_unexpected");
                let _ = history.save(&history_path);
                ExitCode::from(0)
            }
            Err(StoreError::WriterLockHeld(obs)) => {
                history.note(format!("writer_lock_held class={}", obs.class.as_str()));
                let _ = history.save(&history_path);
                ExitCode::from(6)
            }
            Err(e) => {
                history.note(format!("open_err={e}"));
                let _ = history.save(&history_path);
                ExitCode::from(3)
            }
        }
    } else {
        let mut store_h = match Store::create(&store) {
            Ok(s) => s,
            Err(StoreError::AlreadyExists(_)) => match Store::open(&store) {
                Ok(s) => s,
                Err(e) => {
                    history.note(format!("open_err={e}"));
                    let _ = history.save(&history_path);
                    return ExitCode::from(3);
                }
            },
            Err(e) => {
                history.note(format!("create_err={e}"));
                let _ = history.save(&history_path);
                return ExitCode::from(3);
            }
        };

        for i in 0..ops {
            let subject = format!("{prefix}{i}");
            let value = format!("v-{seed}-{i}");
            match store_h.put(&subject, value.as_bytes(), DurabilityMode::Durable) {
                Ok(receipt) => {
                    history.ops.push(MultiprocOp {
                        index: i,
                        subject: subject.clone(),
                        value: value.clone(),
                        event_id_hex: Some(hex16(&receipt.event_id)),
                        acked: true,
                    });
                    // Persist after each ack so kill after this point retains evidence.
                    if let Err(e) = history.save(&history_path) {
                        eprintln!("history save: {e}");
                        return ExitCode::from(4);
                    }
                    if abort_after == Some(i + 1) {
                        history.note(format!("abort_after_ack={}", i + 1));
                        let _ = history.save(&history_path);
                        abort();
                    }
                }
                Err(e) => {
                    history.note(format!("put_err index={i} err={e}"));
                    let _ = history.save(&history_path);
                    return ExitCode::from(4);
                }
            }
        }
        history.note("put_series_complete");
        let _ = history.save(&history_path);
        ExitCode::SUCCESS
    }
}
