# Crash-consistency boundaries (DEF-022)

Status: skeleton (failpoints + machine-readable matrix)  
Audience: store implementers, reviewers, CI  
Companion: [`crates/dingo-store/crash_matrix.v1.json`](../crates/dingo-store/crash_matrix.v1.json),
[DEFECTS.md](../DEFECTS.md) DEF-022

## Goal

Every durable acknowledgement must survive the documented crash boundary.
Unacknowledged work must not appear as a fabricated committed event after
reopen. Derived state (indexes, catalogs, checkpoints) must never outrank
authoritative segment bytes.

## Machine-readable matrix

The source of truth for operation → failpoint → expected reopen state is:

```text
crates/dingo-store/crash_matrix.v1.json
```

It is embedded in the crate (`CRASH_MATRIX_JSON` / `load_crash_matrix`) and
validated on every CI run. Each operation lists:

1. **persistence_order** — ordered durable steps for humans and future drivers
2. **failpoints** — named injection points with:
   - `ci_subset: true` → run on every PR
   - `expected_on_reopen` — reopen assertions (no fabricated commit, salvageable,
     prior durable retained, optional visibility of the in-flight op)

## Failpoint framework

```rust
use dingo_store::{arm_failpoint_once, clear_failpoints, FailpointAction};

arm_failpoint_once("store.active.write_tail.before", FailpointAction::Error);
// ... drive put/delete/seal ...
clear_failpoints();
```

| Action | Behavior |
|--------|----------|
| `Error` | Return `StoreError::Failpoint` (clean error path) |
| `Panic` | Unwind current thread (process-local crash simulation) |
| `Return` | Alias of `Error` |

Failpoints are always compiled and are **no-ops when unarmed**. Names are stable
identifiers listed in the JSON matrix and hit from store/index/dedup/catalog
code paths.

## CI vs nightly

| Mode | How | Coverage |
|------|-----|----------|
| PR / default CI | `cargo test -p dingo-store --test stage_def_022_crash_matrix` | Document validation + cells with `ci_subset: true` |
| Nightly / full | `DINGO_CRASH_MATRIX_FULL=1 cargo test -p dingo-store --test stage_def_022_crash_matrix` | Every matrix cell |

`scripts/nightly.sh` and `.github/workflows/nightly.yml` set the full env.

## What this skeleton does *not* claim yet

- True multi-process kill -9 at every boundary (current driver is
  failpoint + drop + reopen in one process)
- Filesystem-full, short write, and permission-loss injection
- Power-loss equivalence for buffered writes (page cache survives process death)
- Production release gate for every cell (see DEF-022 remaining work)

Expand the JSON matrix and drivers as real fault injection lands; keep CI
subset small enough for PR latency.
