# Licensing policy (AGPL track)

Status: **adopted**  
Audience: copyright holder + contributors + release packaging  
Companion: [RELEASE_ARTIFACTS.md](RELEASE_ARTIFACTS.md), crate layout in [ARCHITECTURE.md](../ARCHITECTURE.md)

This note is the **adopted** per-crate license map for ResiduumDB along a
GPL-family track with **AGPL-3.0-or-later** for networked product bits. It is
product/engineering guidance, not legal advice. Confirm with counsel for your
distribution or SaaS model.

**History:** The workspace was uniformly MIT while the project was bootstrapped.
MIT was temporary scaffolding only; it is **not** the product license policy.

---

## 0. Locked decisions (copyright holder)

| # | Question | Decision |
|---|----------|----------|
| 1 | Strong copyleft for cluster / server / CLI | **AGPL-3.0-or-later** |
| 2 | Weak copyleft for store + embedded API | **MPL-2.0** (not LGPL) |
| 3 | SDA + wire format | **MIT** (remains permissive) |
| 4 | Thin network client (wire) | **MIT** (`residuum-client`) |
| 5 | Inbound contributions | **Inbound = outbound** (license of modified files / crate SPDX) |
| 6 | `residuum-format` | Stays **MIT** even when store is MPL |
| 7 | `residuum-sdk` crate split | **Done for publish path:** default features are MPL embedded + remote; optional `cluster` feature pulls AGPL `residuum-cluster` |

---

## 1. Goals

| Goal | Intent |
|------|--------|
| **AGPL-3.0-or-later** for networked bits | Strong copyleft + network-use source offer on cluster, serve path, multi-node replication |
| **MIT** for client / format / SDA | Closed and open apps can speak the wire protocol or use SDA without taking server copyleft |
| **MPL-2.0** for linkable / embedded | Apps can ship an embedded store in proprietary products; modifications to MPL files stay share-alike |

Those three goals are compatible **only if crate boundaries match license
boundaries**. Dependency direction must flow **permissive → weak copyleft →
strong copyleft**, never the reverse into a “MIT client” crate that still
links strong-copyleft code.

---

## 2. Rust-specific constraints

### 2.1 Static linking is the default

| License | Classic assumption | Rust reality |
|---------|-------------------|--------------|
| **LGPL-3.0** | Dynamic link → proprietary app can stay closed if it only *uses* the library | Static link: compliance is awkward; rarely clean for pure-Rust libs. |
| **MPL-2.0** | File-level weak copyleft | Works cleanly with static linking: only modified MPL files must be disclosed. |
| **GPL-3.0** | Combined work is GPL | Shipping a binary that includes GPL code means GPL compliance for that distribution. |
| **AGPL-3.0** | GPL + network use | SaaS that only *runs* the server still triggers source offer. |

Embedded tier uses **MPL-2.0**, not LGPL.

### 2.2 crates.io SPDX is not the whole story

- Declare the **strongest license that applies to that crate’s own sources**.
- Document **effective** license of recommended feature sets in the README.
- Avoid advertising “MIT SDK” if default features pull AGPL deps.

### 2.3 Dependency direction

```text
OK:    mit-client  ──depends──►  (no copyleft)
OK:    agpl-server ──depends──►  mpl-store  ──depends──►  mit-format
OK:    mpl-store   ──depends──►  mit-format
BAD:   mit-client  ──depends──►  agpl-cluster
BAD:   mpl-store   ──depends──►  agpl-cluster
```

---

## 3. Adopted tiers

```text
┌─────────────────────────────────────────────────────────────┐
│  MIT — protocol, pure algebra, wire format, pure clients    │
│  residuum-sda · residuum-sda-cli · residuum-format · residuum-client · residuum-heap   │
└────────────────────────────▲────────────────────────────────┘
                             │ may depend only upward
┌────────────────────────────┴────────────────────────────────┐
│  MPL-2.0 — linkable embedded engine + collection SDK        │
│  residuum-store · residuum-examine · residuum-sdk · residuum-testrig    │
└────────────────────────────▲────────────────────────────────┘
                             │ may depend only upward
┌────────────────────────────┴────────────────────────────────┐
│  AGPL-3.0-or-later — networked product                      │
│  residuum-cluster · residuum-server · residuum-cli · residuum-authority │
│  (+ residuum-sdk when built with features = ["cluster"])       │
└─────────────────────────────────────────────────────────────┘
```

### 3.1 Per-crate SPDX (current and adopted planned crates)

| Crate (dir → package) | SPDX today | Notes |
|----------------------|------------|-------|
| `sda-core` → `residuum-sda` | **MIT** | SDA+ENR1 hybrid; not bare `sda`/`sda-lib` |
| `sda-cli` → `residuum-sda-cli` | **MIT** | Binary `residuum-sda` |
| `residuum-format` | **MIT** | |
| `residuum-client` | **MIT** | Wire framing + handshake |
| `residuum-heap` | **MIT** | Planned heap identity, certificate, capability, and pure decision kernel |
| `residuum-store` | **MPL-2.0** | |
| `residuum-examine` | **MPL-2.0** | |
| `residuum-sdk` | **MPL-2.0** | Default: embedded + remote; optional `cluster` → AGPL dep |
| `residuum-testrig` | **MPL-2.0** | Unpublished store stress/chaos tool |
| `residuum-cluster` | **AGPL-3.0-or-later** | |
| `residuum-server` | **AGPL-3.0-or-later** | Enables `residuum-sdk/cluster` |
| `residuum-cli` → `dingo` | **AGPL-3.0-or-later** | Enables `residuum-sdk/cluster` |
| `residuum-authority` | **AGPL-3.0-or-later** | Planned separate local-only heap authority executable; never linked by data server |
| `dingo-studio-core` | **AGPL-3.0-or-later** | Planned Studio orchestration and remote-management core |
| `apps/dingo-studio` | **AGPL-3.0-or-later** | Planned Residuum Studio desktop product |

### 3.2 License files

| Path | Content |
|------|---------|
| `LICENSE` | Multi-license notice + map |
| `LICENSE-MIT` | MIT full text |
| `LICENSE-MPL-2.0` | MPL-2.0 full text |
| `LICENSE-AGPL-3.0` | AGPL-3.0 full text (project applies **or-later**) |

---

## 4. Split status: `residuum-sdk` was three products

### 4.0 Done

| Package | Status | License |
|---------|--------|---------|
| `residuum-client` | **Extracted** — framed RPC + handshake only | MIT |
| `residuum-server` | **Extracted** — accept loop, authz, admission, raft RPC glue, `serve_*` | AGPL-3.0-or-later |
| `residuum-sdk` | **MPL default**; remote client + TLS always on; `cluster` feature optional | MPL-2.0 |

### 4.1 Modules in `residuum-sdk`

| Module group | Natural tier | Status |
|--------------|--------------|--------|
| `collection`, `dingo` (local open), `filter`, `history`, `indexes`, `value`, `receipt`, `error` | **Embedded** (MPL) | Always on |
| `remote`, `directory_cache` (wire types, no `residuum-cluster`), client TLS, connect helpers | **Remote client** (MPL; wire re-export MIT) | Always on; directory cache no longer imports AGPL types |
| `cluster_backend`, `Dingo::open_cluster` / `create_cluster` | **Networked / AGPL** | Behind `features = ["cluster"]` only |

**Today:** default `residuum-sdk` is honestly **MPL-2.0** (depends on `residuum-store` +
`residuum-client`, not `residuum-cluster`). Builds with `cluster` pull AGPL
`residuum-cluster` — document that effective license for those binaries follows
the AGPL dependency. Serve path lives only in `residuum-server`.

### 4.2 Crate apportionment

| Package | Contents | License | Depends on |
|---------|----------|---------|------------|
| `residuum-format` | unchanged | MIT | — |
| `residuum-client` | wire framing + handshake | MIT | — |
| `residuum-heap` | heap identity, credentials, capability and pure decision kernel | MIT | format |
| `residuum-store` | unchanged | MPL-2.0 | format |
| `residuum-sdk` | `Dingo::open`, connect, collections, filters, indexes; optional cluster | MPL-2.0 | store, client, residuum-sda; optional cluster |
| `residuum-testrig` | unpublished store stress, chaos, and performance rig | MPL-2.0 | store |
| `residuum-examine` | unchanged | MPL-2.0 | store, format, residuum-sda |
| `residuum-cluster` | unchanged | AGPL-3.0-or-later | store |
| `residuum-server` | accept loop, authz, admission, raft RPC glue | AGPL-3.0-or-later | sdk+cluster, store |
| `residuum-cli` | CLI + doctor/salvage/serve | AGPL-3.0-or-later | server, sdk+cluster, examine |
| `residuum-authority` | separate local authority mutation and genesis executable | AGPL-3.0-or-later | heap, format, store (`authority-provisioning`) |
| `residuum-sda` / `residuum-sda-cli` | SDA+ENR1 hybrid | MIT | — |

### 4.3 Remaining optional polish

1. ~~**`residuum-client`** (MIT) — protocol framing~~ **done**
2. ~~**`residuum-server`** (AGPL) — serve modules out of sdk~~ **done**
3. ~~**`residuum-sdk`** → MPL-2.0 default; cluster feature-gated~~ **done**
4. Optional: move remote/TLS into a separate MIT/MPL crate later; dual-crate
   is not required for an honest MPL embedded + remote publish.

---

## 5. GPL-track matrix (adopted)

```text
MIT                → residuum-sda, residuum-sda-cli, residuum-format, residuum-client,
                     residuum-heap
MPL-2.0            → residuum-store, residuum-examine, residuum-sdk (default features),
                     residuum-testrig
AGPL-3.0-or-later  → residuum-cluster, residuum-server, residuum-cli, residuum-authority
                     (+ residuum-sdk when features = ["cluster"])
```

AGPL protects “networked bits” against pure SaaS freeloading (source offer on
network use). Commercial exception / dual-license for AGPL server and/or MPL
store remains an optional business track; keep pure client and format MIT.

---

## 6. Release checklist

1. **Per-crate `license` in Cargo.toml** — done for existing crates; HP-001
   adds `residuum-heap` as MIT and HP-005 adds `residuum-authority` as AGPL.
2. **LICENSE files** — root multi-license tree (done).
3. **README + CONTRIBUTING** — multi-license notice; inbound = outbound (done).
4. **CLI `--license`** — `residuum-sda` MIT and `dingo` AGPL are done;
   `residuum-authority` MUST report AGPL when HP-005 creates it.
5. **Publish `residuum-sdk` as MPL-2.0** with default features only (no
   `residuum-cluster`). Document that `features = ["cluster"]` pulls AGPL.
6. **`cargo deny` / license policies** — optional hardening before crates.io.
7. ~~**Remaining sdk split**~~ — server extract + cluster feature-gate **done**.

---

## 7. Compatibility with current dependency edges

Today:

```text
residuum-cli      → residuum-sdk (cluster), residuum-server, residuum-store, residuum-examine  (AGPL)
residuum-server   → residuum-sdk (cluster), residuum-cluster, residuum-store               (AGPL)
residuum-sdk      → residuum-client, residuum-store, sda-core  (+ optional residuum-cluster) (MPL)
residuum-client   → (none of store/cluster)                                       (MIT)
residuum-cluster  → residuum-store                                                   (AGPL)
residuum-examine  → residuum-format, residuum-store, sda-core                           (MPL)
residuum-testrig  → residuum-store                                                   (MPL)
residuum-store    → residuum-format                                                  (MPL)
sda-cli        → sda-core                                                      (MIT)
```

All edges respect “stronger may depend on weaker.” Default `residuum-sdk` has no
AGPL dependency.

---

## 8. One-paragraph summary

**Adopted:** keep **SDA and the wire format MIT**; keep a **thin network
client MIT** (`residuum-client`); put **MPL-2.0 on the embedded store, examination
host, and default `residuum-sdk`** (embedded + remote); put **AGPL-3.0-or-later on
cluster, server, the `dingo` operator binary**, and any build that enables
`residuum-sdk`’s `cluster` feature.
