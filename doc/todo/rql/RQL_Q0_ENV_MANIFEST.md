# RQL-Q0 — Environment and engine manifest

Status: **labor complete · principal freeze pending**

Package: RQL-Q0 deliverable 1
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §3
Board task: Q0.1
Effective: 2026-08-07

This file freezes **what is compared** and **how environment is fingerprinted**.
It does not claim performance competitiveness or Gate-1 pass.

---

## 1. Residiuum pin

| Field | Primary-profile value |
|---|---|
| Workspace package version | **0.2.2** (`VERSION`, `[workspace.package]`) |
| MSRV | **1.88.0** (build must honor; host may run newer rustc) |
| Git identity | Full SHA of the commit under test (see §6) |
| Dirty tree | Forbidden for qualification campaigns; allow only with explicit principal waiver + `dirty=true` in fingerprint |
| Product surfaces under test | `residiuum-sdk` (preferred); `residiuum-store` embedded path; `residiuum serve` / client for local c/s lane |
| Query authority | Canonical QVM (`query_bytecode_v1` / QVM1); one product runtime |
| Release profile | `--release` for Q5/Q7 campaigns; debug builds are not comparative |

**Git SHA policy**

1. Evidence bundles record `git_sha` = `git rev-parse HEAD` (40 hex).
2. Optional `git_describe` for humans; SHA is authority.
3. Comparative cells across engines in one campaign share the same Residiuum SHA.
4. Re-runs that change SHA are a **new campaign**, not a continuation of a frozen baseline.

---

## 2. Comparator engine pins (primary profile)

These pins are the **primary Gate-1 comparator profile**. Patch-level bumps within
the same declared minor require a one-line amendment + principal note; major/minor
bumps require re-accept of Q0 (or a delta accept).

### 2.1 MongoDB Community — local client/server lane

| Field | Pin |
|---|---|
| Product | MongoDB Community Server |
| Version | **8.0.4** |
| Deployment | Single-node local `mongod` (no Atlas, no replica-set multi-node) |
| Wire | Standard MongoDB TCP / official drivers |
| Auth | None on loopback for baseline (or SCRAM with secrets outside evidence JSON) |
| Storage engine | WiredTiger (default) |
| Journal / durability | Default community durable write concern for measured writes; read-only cells use steady open |
| Install notes | Official tarball or package for host OS; record exact binary version string from `mongod --version` |

**Why 8.0.4:** current 8.0-line Community pin for operational document comparison;
not a claim that Residiuum implements the full Mongo surface.

### 2.2 Couchbase Lite — embedded lane

| Field | Pin |
|---|---|
| Product | Couchbase Lite |
| Version | **3.2.1** |
| Edition | Community-capable embedded library used for local queries (SQL++ / QueryBuilder) |
| Deployment | In-process / embedded database file on local FS |
| Sync Gateway | **Out of scope** for Gate-1 query qualification |
| Install notes | Language binding used by the harness must record crate/npm/pod version + CBL core version |

**Why 3.2.1:** stable embedded document query comparator for the embedded lane;
not mobile-platform matrix coverage.

---

## 3. Host and OS class (controlled primary host)

Primary controlled host class (Apple silicon laptop / workstation):

| Field | Primary class |
|---|---|
| OS | macOS 15.x (Darwin 24.x) |
| Arch | arm64 (Apple Silicon) |
| Filesystem for data dirs | APFS on local SSD |
| Network | Loopback only for Mongo lane; no WAN |
| Thermal / power | Plugged-in preferred; record power source when available |

**Labor host sample (non-binding discovery, 2026-08-07):** Apple M4, 16 GiB RAM,
macOS 15.5 (24F74), APFS, rustc/cargo 1.92.0. Campaigns must re-fingerprint;
this row is not a frozen performance claim.

Secondary host class (optional, not required for first Gate-1 pass):

| Field | Secondary class |
|---|---|
| OS | Linux 6.x |
| Arch | x86_64 or aarch64 |
| Filesystem | ext4 or xfs on local NVMe |

Cross-class geometric means are **not** allowed to substitute for same-class cells.

---

## 4. Build toolchain

| Field | Pin / policy |
|---|---|
| rustc | Host-installed; record full `rustc --version` |
| cargo | Matching cargo; record `cargo --version` |
| Features | Default product features for SDK path under test; list every `--features` flag |
| LTO / codegen | Workspace defaults unless a campaign explicitly freezes alternate flags |

MSRV 1.88.0 remains the supported floor; qualification hosts may use newer stable
rustc and must record it.

---

## 5. Filesystem and store layout assumptions

- Residiuum data directories live on the **same volume class** as comparator DB files
  within a cell (no mixing network FS vs local SSD in one comparative cell).
- No NFS / shared multi-writer media.
- Exclusive writer flock semantics apply for Residiuum store opens.
- Temporary and durable paths are recorded in the evidence bundle.

---

## 6. Environment fingerprint (evidence bundles)

Every Q4/Q5/Q7 evidence bundle MUST include an `environment.json` (or equivalent)
with at least:

```text
residiuum_version          # from VERSION
git_sha                    # 40 hex
git_dirty                  # bool
rustc_version
cargo_version
cargo_features             # list
build_profile              # release | debug (debug forbidden for comparative)
os_name, os_version, kernel
arch
cpu_model, cpu_logical_cores
ram_bytes
fs_type, volume_encryption_if_known
storage_device_class       # local_ssd | local_hdd | other
mongodb_version            # lane-local; omit if lane unused
couchbase_lite_version     # lane-local; omit if lane unused
lane_id                    # embedded | local_client_server
seed
campaign_id
host_class                 # primary_apple_silicon | secondary_linux | other
```

Optional but recommended: free disk bytes, thermal state, power source, process
limits, active background agents.

**Invalid environment** (cell must not support competitive claims):

- debug Residiuum build in a comparative cell;
- dirty git without waiver;
- mismatched engine pin vs this manifest;
- thermal/power throttling observed mid-run when the harness can detect it;
- ENOSPC or media errors;
- mixed lanes collapsed into one score.

Aligns with performance harness environment honesty
([PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md](../performance-qualification/PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md) §15).

---

## 7. Amendment

Changes to pins in §2 require principal review. Editorial clarification of
fingerprint fields does not re-open Q0 if pins are unchanged.

---

## 8. Exit (Q0.1)

- [x] Residiuum version + git policy named
- [x] MongoDB Community pin named (8.0.4)
- [x] Couchbase Lite pin named (3.2.1)
- [x] OS / FS / host class named
- [x] Fingerprint schema for evidence bundles named
- [ ] Principal accept of this freeze (package-level Q0)
