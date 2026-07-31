# Crash-consistency boundaries (DEF-022)

Status: **hardened beyond skeleton** (failpoints + matrix + multi-process abort +
I/O fault injection)  
Audience: store implementers, reviewers, CI  
Companion: [`crates/residuum-store/crash_matrix.v1.json`](../crates/residuum-store/crash_matrix.v1.json),
[DEFECTS.md](../DEFECTS.md) DEF-022

## Goal

Every durable acknowledgement must survive the documented crash boundary.
Unacknowledged work must not appear as a fabricated committed event after
reopen. Derived state (indexes, catalogs, checkpoints) must never outrank
authoritative segment bytes.

## Machine-readable matrix

The source of truth for operation → failpoint → expected reopen state is:

```text
crates/residuum-store/crash_matrix.v1.json
```

It is embedded in the crate (`CRASH_MATRIX_JSON` / `load_crash_matrix`) and
validated on every CI run. Each operation lists:

1. **persistence_order** — ordered durable steps for humans and future drivers
2. **failpoints** — named injection points with:
   - `ci_subset: true` → run on every PR
   - optional `fault` — `enospc` | `permission` | `short_write` | `process_abort` | `panic` | `error`
   - `expected_on_reopen` — reopen assertions (no fabricated commit, salvageable,
     prior durable retained, optional visibility of the in-flight op)

## Failpoint framework

```rust
use residuum_store::{arm_failpoint_once, clear_failpoints, FailpointAction};

arm_failpoint_once("store.active.write_tail.before", FailpointAction::Error);
// ... drive put/delete/seal ...
clear_failpoints();
```

| Action | Behavior |
|--------|----------|
| `Error` / `Return` | Return `StoreError::Failpoint` (clean error path) |
| `Panic` | Unwind current thread (process-local crash simulation) |
| `Abort` | `std::process::abort()` — true process death (multi-process harness) |
| `IoEnospc` | `StoreError::Io` with `ErrorKind::StorageFull` |
| `IoPermission` | `StoreError::Io` with `ErrorKind::PermissionDenied` |
| `ShortWrite` | Consumed at instrumented write sites; partial bytes + `WriteZero` |

Failpoints are always compiled and are **no-ops when unarmed**. Names are stable
identifiers listed in the JSON matrix and hit from store/index/dedup/catalog
code paths.

### Instrumented short-write sites

| Name | Where |
|------|--------|
| `store.active.write_tail.short_write` | Active segment append |
| `atomic.tmp.short_write` | Control-document temp body (`atomic_file`) |

## Multi-process abort harness

Integration tests spawn the helper binary `residuum-store-crash-child`:

```text
RESIDUUM_CRASH_STORE=<path>
RESIDUUM_CRASH_OP=put_durable|delete_durable|seed_prior
RESIDUUM_CRASH_FP=<failpoint name>   # armed with Abort
RESIDUUM_CRASH_KEY / RESIDUUM_CRASH_VAL
```

Parent seeds durable prior state, child aborts mid-op, parent reopens and
asserts matrix expectations. CI always runs kill-before-write and kill-after-sync
cells for durable put.

## CI vs nightly

| Mode | How | Coverage |
|------|-----|----------|
| PR / default CI | `cargo test -p residuum-store --test stage_def_022_crash_matrix` | Document validation + `ci_subset` cells + I/O suite + multi-process abort |
| Nightly / full | `RESIDUUM_CRASH_MATRIX_FULL=1 cargo test -p residuum-store --test stage_def_022_crash_matrix` | Every matrix cell |

`scripts/nightly.sh` and `.github/workflows/nightly.yml` set the full env.

## What is covered now

- Process-local failpoint + drop + reopen (Error / Panic)
- Multi-process `Abort` at durable put before-write and after-sync
- ENOSPC and permission-denied injection at write_tail boundaries
- Short-write injection on active append and atomic control temps
- Best-effort real OS `chmod` permission loss on the active segment (Unix)

## What is still not claimed

- Power-loss equivalence for buffered writes (page cache can survive process death
  on the same machine without a full machine power cycle)
- Full production release gate on every matrix cell under adversarial FS
  (rename races, flaky disks, every seal/compact/tier cell under real kill -9)
- Distributed / multi-node crash coordination (see Raft defects)

Expand the JSON matrix and drivers as remaining boundaries harden; keep CI
subset small enough for PR latency.
