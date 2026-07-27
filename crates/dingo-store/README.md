# dingo-store

Single-node **authoritative store** for DingoDB: filesystem-backed append-only
segments, put/get/delete by subject, durability modes, catalog-independent
recovery, derived state (catalogs, secondary indexes, chunks, history,
compaction, checkpoints), inspect/salvage helpers, and tiering
(hot/warm/cold/archive) with offline coverage honesty.

Most applications should use [`dingo-sdk`](https://crates.io/crates/dingo-sdk)
(named collections, JSON, filters). Use **this crate** when you need the raw
store API, operator tools, or to embed storage without the collection layer.

## When to use this crate

| You want… | Use |
|-----------|-----|
| Named collections, JSON, filters, remote connect | [`dingo-sdk`](https://crates.io/crates/dingo-sdk) |
| Subject-keyed store, salvage, tiering, durability modes | **`dingo-store`** (this crate) |
| Frame codec only | [`dingo-format`](https://crates.io/crates/dingo-format) |
| Multi-node partitions / Raft | [`dingo-cluster`](https://crates.io/crates/dingo-cluster) |

## Install

```toml
[dependencies]
dingo-store = "0.1"
```

Or: `cargo add dingo-store`

## Quick example

```rust
use dingo_store::{DurabilityMode, Store};

# let dir = tempfile::tempdir().unwrap();
# let path = dir.path();
let mut store = Store::create(path)?;
store.put("user-42", br#"{"name":"Alice"}"#, DurabilityMode::Durable)?;
assert_eq!(
    store.get("user-42")?.as_deref(),
    Some(br#"{"name":"Alice"}"#.as_slice())
);
store.delete("user-42", DurabilityMode::Durable)?;
assert!(store.get("user-42")?.is_none());
# Ok::<(), dingo_store::StoreError>(())
```

## Status

**Shipped** (Stages 3 + 6 + 7 inspect/salvage + 9 tiering). Highlights:

| Area | What you get |
|------|----------------|
| Core | open/create, put/get/delete, durability modes (`Memory`, `Buffered`, `Durable`) |
| Ownership | exclusive writer lock; `open_inspect` is lock-free read-only |
| Recovery | rebuildable primary index, salvage after catalog wipe, evidence-preserving `salvage_to` |
| Derived | collection catalog, secondary indexes, subject history, checkpoints |
| Hydra | adaptive per-segment indexes at seal (Eytzinger / PGM·RadixSpline / compressed radix / MPHF); multithread rebuild |
| Chimera | value locators + seal/compaction layouts under `indexes/chimera/`; `get` may resolve sealed layouts (put still segment frames; dual-rep/ZNS deferred; compiler worker next) |
| Chunks | chunked payloads with partial maps; phased live compaction |
| Operator | `open_inspect` (doctor), `salvage_to`, `export_live_state`, `backup_to` / `restore_full_backup` (DEF-050), `scrub_once` / `scrub_status` (DEF-051), `migrate_to` (DEF-052) |
| Tiering | segment move/copy with stable identities; offline-tier coverage holes |
| Honesty | fail-closed logical scans; absence only proven when coverage is complete |

## Layout

```text
store/
  store-info/     # store_id + meta + descriptor + writer.lock
  active/         # open append segment (at most one)
  segments/       # sealed hot segments
  tiers/          # warm/cold/archive media
  catalogs/       # derived only (rebuildable)
  indexes/        # derived only (rebuildable)
  snapshots/      # derived checkpoints
  recovery/       # operator scratch + compaction jobs + scrub + migration jobs
```

## Format migration (DEF-052)

Phased, evidence-preserving copy into a **new** store (never in-place rewrite):

```rust
use dingo_store::{DurabilityMode, MigrateOptions, MigratePhase, Store};

# let dir = tempfile::tempdir().unwrap();
# let src = dir.path().join("src");
# let dst = dir.path().join("dst");
let mut store = Store::create(&src)?;
store.put("k", b"v", DurabilityMode::Durable)?;
let report = store.migrate_to(&dst, MigrateOptions::default())?;
assert_eq!(report.phase, MigratePhase::Done);
# Ok::<(), dingo_store::StoreError>(())
```

Job documents live under `recovery/migration/job.v1.json`. Wire support is
declared in `dingo-format` (`wire_compat_matrix` / `SUPPORTED_READER_MAJORS`).

## Integrity scrub (DEF-051)

Bounded verification of segments and chunks:

```rust
use dingo_store::{DurabilityMode, ScrubOptions, Store};

# let dir = tempfile::tempdir().unwrap();
# let root = dir.path().join("s");
let mut store = Store::create(&root)?;
store.put("k", b"v", DurabilityMode::Durable)?;
let report = store.scrub_to_completion(ScrubOptions::default())?;
assert!(report.cycle_completed);
assert_eq!(report.status.open_findings, 0);
# Ok::<(), dingo_store::StoreError>(())
```

State lives under `recovery/scrub/` (`state.v1.json`, `findings.v1.json`,
optional `quarantine/` copies). Scrub never deletes or rewrites authoritative
segment bytes. Pause with `pause_scrub` / resume with `resume_scrub`.

## Backup and restore (DEF-050)

Full backups are **packages**, not live stores:

```text
backup-package/
  backup-manifest.v1.json   # profile dingo-backup-v1 + blake3 of files
  store/                    # authoritative trees only (no lock files)
```

```rust
use dingo_store::{restore_full_backup, DurabilityMode, RestoreOptions, Store};

# let dir = tempfile::tempdir().unwrap();
# let src = dir.path().join("src");
# let bak = dir.path().join("bak");
# let dst = dir.path().join("dst");
let mut store = Store::create(&src)?;
store.put("k", b"v", DurabilityMode::Durable)?;
store.backup_to(&bak)?;
drop(store);
let report = restore_full_backup(&bak, &dst, RestoreOptions::default())?;
assert_eq!(report.live_subjects, 1);
# Ok::<(), dingo_store::StoreError>(())
```

Salvage remains the damage-recovery path; export-live re-materializes current
values with new lineage. Neither produces a `dingo-backup-v1` package.

Deleting `catalogs/`, `indexes/`, and `snapshots/` must not prevent recovery:
the store rebuilds current state by scanning `active/`, `segments/`, and online
tier media.

## API surface

| API | Role |
|-----|------|
| `Store::open` / `Store::create` | Create-or-open; exclusive writer lock |
| `Store::open_inspect` | Read-only open (no writer lock, no derived writes) |
| `put` / `get` / `delete` | Subject-keyed current-state operations |
| `get_payload` | Completeness-aware read (`PayloadResult`) |
| `history` | Per-subject event stream |
| `WriteReceipt` / `DurabilityMode` | Event identity + acknowledged durability |
| `rebuild_index` / `salvage` | Catalog-free scan of all segment files |
| `salvage_to` | Evidence-preserving frame copy + recovery manifest |
| `export_live_state` | Live-only re-put materialization (new lineage) |
| `backup_to` / `restore_full_backup` | Content-hashed full backup package (DEF-050) |
| `scrub_once` / `scrub_to_completion` / `scrub_status` | Bounded integrity scrub + findings (DEF-051) |
| `compact_live` / compact job helpers | Phased live projection; reclaim only with `allow_history_loss` |
| `scan_live_page` | Bounded page + continuation token |
| `transfer_segment_to_tier` | Copy/move sealed segment (stable id) |
| `get_with_tier_coverage` | Absence only proven when coverage complete |
| `examination_sources` | Ordered `(source_name, bytes)` for examination |

## Durability modes

| Mode | Meaning |
|------|---------|
| `Memory` | Acknowledged in process; may be lost on crash |
| `Buffered` | Written to OS buffers; may be lost on power failure |
| `Durable` | Flushed for the configured durable path |

## Design rule

> Durable truth must not depend on replaceable machinery.

Indexes and catalogs make access fast but are not the sole authority. They are
designed to be rebuilt from immutable, independently framed segments
([`dingo-format`](https://crates.io/crates/dingo-format)).

## Out of scope (this crate)

- Native SigV4 HTTP object SDK (use mirror / fuse mount)
- Erasure encode/decode codecs (manifest only)
- Background lifecycle scheduler (policy evaluate only)
- `replicated` durability — see [`dingo-cluster`](https://crates.io/crates/dingo-cluster)

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`dingo-format`](https://crates.io/crates/dingo-format) | MIT | Wire format this store writes |
| [`dingo-sdk`](https://crates.io/crates/dingo-sdk) | MPL-2.0 | Collection API over this store |
| [`dingo-examine`](https://crates.io/crates/dingo-examine) | MPL-2.0 | SDA examination over salvage |
| [`dingo-cluster`](https://crates.io/crates/dingo-cluster) | AGPL-3.0-or-later | Multi-node federation |

## Documentation

- Architecture: [OVERVIEW.md](https://github.com/frogfishio/dingodb/blob/main/OVERVIEW.md)
- Format: [FORMAT_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/FORMAT_SPEC.md)
- Crash consistency: [doc/CRASH_CONSISTENCY.md](https://github.com/frogfishio/dingodb/blob/main/doc/CRASH_CONSISTENCY.md)
- Retention runbook: [doc/RUNBOOK_RETENTION.md](https://github.com/frogfishio/dingodb/blob/main/doc/RUNBOOK_RETENTION.md)

## License

MPL-2.0 (file-level weak copyleft). Proprietary applications may embed the
store; modifications to MPL-covered files must be disclosed.

Part of [DingoDB](https://github.com/frogfishio/dingodb). Multi-tier license map:
[doc/LICENSING.md](https://github.com/frogfishio/dingodb/blob/main/doc/LICENSING.md).
