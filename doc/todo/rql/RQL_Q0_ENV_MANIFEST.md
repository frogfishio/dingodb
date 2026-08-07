# RQL-Q0 — Environment and engine manifest

Status: **Q0.A1 amendment · principal freeze re-accept pending**

Package: RQL-Q0 deliverable 1 (amended)  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §3 ·
principal review finding #1 (obsolete comparator pins)  
Board: Q0.1 (first freeze) · **Q0.A1** (this amendment)  
Feature: `019fdac4-1408-7321-8edc-a09851c9e656`  
Effective: 2026-08-07 (A1 pin bump)

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

**Supersedes first freeze:** MongoDB **8.0.4** and Couchbase Lite **3.2.1** are
**retired** as primary pins (principal review: no longer representative).

### 2.1 MongoDB Community — local client/server lane (lane **S**)

#### 2.1.1 Server

| Field | Pin |
|---|---|
| Product | MongoDB Community Server |
| Version | **8.2.12** (exact; not “any 8.2.x”) |
| Line | 8.2 (minor Release; Community on-prem) |
| Authority | [MongoDB 8.2 release notes](https://www.mongodb.com/docs/manual/release-notes/8.2/) · patch **8.2.12** (2026-07-22) |
| Deployment | Single-node local `mongod` only — **no** Atlas, **no** multi-node replica set, **no** sharded cluster |
| Binary proof | Evidence must record full `mongod --version` string; must contain `v8.2.12` |
| Storage engine | **WiredTiger** (server default; do not enable alternate engines) |
| Install | Official Community tarball or OS package for host OS/arch; no custom builds |

#### 2.1.2 Client / driver (harness)

| Field | Pin |
|---|---|
| Primary harness driver | Official **MongoDB Rust driver** crate `mongodb` **3.8.0** ([crates.io](https://crates.io/crates/mongodb/3.8.0)) |
| Language | Rust (same toolchain class as Residiuum build host; record `rustc`/`cargo`) |
| Alternative driver | **Forbidden** in primary-profile comparative cells without Q0 delta accept (no silent Node/Java/Go swap) |
| Wire protocol | Standard MongoDB TCP as spoken by this driver to local `mongod` |

#### 2.1.3 Transport, auth, pool

| Field | Pin |
|---|---|
| Transport | TCP to **127.0.0.1** (or `localhost` resolving to loopback only) |
| TLS | **Off** for primary baseline (plain loopback) |
| Port | Record exact port; default **27017** if free |
| Auth mode | **None** (`auth` disabled / no credentials) for primary baseline |
| Auth residual | SCRAM with secrets **outside** evidence JSON is allowed only with principal waiver + `auth_mode=scram` in fingerprint — not the default comparative profile |
| Connection string shape | `mongodb://127.0.0.1:<port>/?…` with options below only |
| Max pool size | **10** (`maxPoolSize=10` / driver equivalent) |
| Min pool size | **0** |
| Connect timeout | **10s** |
| Server selection timeout | **30s** |
| Socket timeout | **0** (no artificial short socket timeout that truncates slow cells) |
| Retryable writes | Driver default for 8.2; record `retryWrites` value in fingerprint |
| Compressors | **None** (disable wire compression for baseline) |

#### 2.1.4 Durability / read / write settings

These must be identical for every write-bearing comparative cell unless the cell
explicitly names a different consistency class (and then it cannot be mixed into
an undifferentiated portfolio score).

| Field | Pin |
|---|---|
| Write concern | **`{ w: 1, j: true }`** (acknowledge + journal) |
| Read concern | **`local`** for primary baseline query cells |
| Read preference | **`primary`** only (single-node) |
| Causal consistency | **Off** unless a named cell freezes it on |
| Majority / w:majority | **Out of profile** for single-node baseline (would change work vs Residiuum single-node) |

**Why 8.2.12:** representative current 8.2-line Community server at labor date;
includes security/reliability patch content on that line. Not a claim that
Residiuum implements the full Mongo surface.

---

### 2.2 Couchbase Lite — embedded lane (lane **E**)

#### 2.2.1 Product and core

| Field | Pin |
|---|---|
| Product | Couchbase Lite |
| Product version | **4.1.0** (exact) |
| Authority | [CBL what’s new 4.1](https://docs.couchbase.com/couchbase-lite/current/cbl-whatsnew.html) · platform release notes (C/Java/…) |
| Edition | **Community** binaries for embedded local query (SQL++ / QueryBuilder) |
| Deployment | In-process / embedded database file on **local** FS |
| Sync Gateway / peer sync / BLE multipeer | **Out of scope** for Gate-1 query qualification |
| Mobile-only platform matrix | **Out of scope** (we pin desktop/CI host class; not iOS/Android fleet) |

#### 2.2.2 Binding and wrapper (harness)

Primary profile freezes **one** binding stack for comparative cells. The harness
must not mix bindings within a campaign without a named residual.

| Field | Pin |
|---|---|
| Primary binding | **Couchbase Lite for C 4.1.0** (Community download matching host OS/arch) |
| C++ wrapper | Official **cbl++** headers shipping **in the same 4.1.0 C distribution** (supported in 4.1) |
| Core library | The `libcblite` / LiteCore binary **bundled inside that 4.1.0 package** — record file name + version string from package metadata / `CBL_Version()` (or equivalent) |
| Alternate bindings | Java **4.1.0**, .NET **4.1.0**, etc. are **same product line** but **not** interchangeable in a single primary-profile cell without principal note (different wrapper cost) |
| Rust FFI residual | If Residiuum harness uses a sys crate, pin the **exact crate version + git SHA** of the binding **and** the linked CBL 4.1.0 core; do not treat “any CBL” as equal |

#### 2.2.3 Database open / durability / concurrency

| Field | Pin |
|---|---|
| Database path | Local FS on same volume class as Residiuum data dir in the cell |
| Encryption | **Off** for primary baseline |
| Full sync / revs | Product defaults for 4.1.0 Community embedded; record any non-default `DatabaseConfiguration` fields |
| Concurrency | Single writer thread for load phase unless cell names multi-writer; record thread model |
| Index build | Explicit indexes required by corpus only; record index definitions in evidence |
| Query API | **SQL++** and/or **QueryBuilder** as declared per corpus case — both must target the same CBL 4.1.0 open database |

**Why 4.1.0:** current C/Java/C# embedded line at labor date (principal review);
replaces retired **3.2.1**. Not mobile-platform matrix coverage.

---

### 2.3 Residiuum side of each lane (config honesty)

| Lane | Residiuum process | Transport | Durability class |
|---|---|---|---|
| **E** Embedded | In-process SDK / store | None (API) | Store default durable open/seal for the product path under test; record store features |
| **S** Local c/s | `residiuum serve` + `residiuum-client` / façade on **loopback** | Framed RPC over loopback TCP | Server single-node development path; record listen address + auth off for baseline |

Do **not** compare Residiuum embedded against MongoDB TCP in one undifferentiated
score (see [RQL_Q0_LANES_EXCLUSIONS.md](./RQL_Q0_LANES_EXCLUSIONS.md)).

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
| Comparator crates | `mongodb = "3.8.0"` (exact) when Rust harness is used; record Cargo.lock hash |

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
cargo_lock_hash            # recommended when comparator crates in-tree
build_profile              # release | debug (debug forbidden for comparative)
os_name, os_version, kernel
arch
cpu_model, cpu_logical_cores
ram_bytes
fs_type, volume_encryption_if_known
storage_device_class       # local_ssd | local_hdd | other
lane_id                    # embedded | local_client_server
seed
campaign_id
host_class                 # primary_apple_silicon | secondary_linux | other

# --- lane S (Mongo) when used ---
mongodb_server_version     # full mongod --version string; must match 8.2.12
mongodb_driver_crate       # e.g. mongodb
mongodb_driver_version     # e.g. 3.8.0
mongodb_uri_redacted       # host/port/options; no secrets
mongodb_write_concern      # e.g. w=1,j=true
mongodb_read_concern       # e.g. local
mongodb_read_preference    # e.g. primary
mongodb_max_pool_size      # e.g. 10
mongodb_tls                # false for baseline
mongodb_auth_mode          # none | scram_waived

# --- lane E (CBL) when used ---
cbl_product_version        # 4.1.0
cbl_binding                # c | java | csharp | ...
cbl_binding_version        # must match product line unless residual named
cbl_core_version           # from package / CBL_Version()
cbl_encryption             # false for baseline
cbl_database_config        # non-default fields only
```

Optional but recommended: free disk bytes, thermal state, power source, process
limits, active background agents.

**Invalid environment** (cell must not support competitive claims):

- debug Residiuum build in a comparative cell;
- dirty git without waiver;
- mismatched engine pin vs this manifest (including **retired** 8.0.4 / 3.2.1);
- wrong write/read concern vs §2.1.4 or undeclared pool/TLS/auth;
- CBL binding/core not recorded or mixed within a campaign without residual;
- thermal/power throttling observed mid-run when the harness can detect it;
- ENOSPC or media errors;
- mixed lanes collapsed into one score.

Aligns with performance harness environment honesty
([PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md](../performance-qualification/PERFORMANCE_QUALIFICATION_HARNESS_SPEC.md) §15).

---

## 7. Amendment

| Change class | Process |
|---|---|
| Patch bump within frozen minor (e.g. 8.2.12 → 8.2.13) | One-line pin edit + principal note; fingerprint must match new exact version |
| Minor/major engine bump | Q0 delta accept or full re-accept |
| Driver/binding swap | Q0 delta accept (changes competitor work) |
| Durability/read/write/pool/TLS/auth change | Q0 delta accept |
| Editorial fingerprint field names | No re-open if pins/config unchanged |

---

## 8. Exit

### Q0.1 (first freeze)

- [x] Residiuum version + git policy named
- [x] OS / FS / host class named
- [x] Fingerprint schema for evidence bundles named

### Q0.A1 (this amendment)

- [x] MongoDB Community primary pin **8.2.12** (retired 8.0.4)
- [x] Official driver pin **mongodb 3.8.0** (Rust harness primary)
- [x] Write/read concern, pool, TLS, auth, transport frozen for lane S
- [x] Couchbase Lite primary pin **4.1.0** (retired 3.2.1)
- [x] C binding + core recording policy frozen for lane E
- [x] Fingerprint fields extended for full comparator config
- [ ] Principal accept of amended freeze (package-level Q0 after remaining A* work)

---

## 9. References (pin authority)

- MongoDB 8.2 / 8.2.12: https://www.mongodb.com/docs/manual/release-notes/8.2/
- MongoDB Rust driver 3.8.0: https://crates.io/crates/mongodb/3.8.0
- Couchbase Lite 4.1: https://docs.couchbase.com/couchbase-lite/current/cbl-whatsnew.html
- Couchbase Lite C install 4.1.0: https://docs.couchbase.com/couchbase-lite/current/c/gs-install.html
