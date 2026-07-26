# Licensing policy (AGPL track)

Status: **adopted**  
Audience: copyright holder + contributors + release packaging  
Companion: [RELEASE_ARTIFACTS.md](RELEASE_ARTIFACTS.md), crate layout in [ARCHITECTURE.md](../ARCHITECTURE.md)

This note is the **adopted** per-crate license map for DingoDB along a
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
| 4 | Thin network client (after extract) | **MIT** |
| 5 | Inbound contributions | **Inbound = outbound** (license of modified files / crate SPDX) |
| 6 | `dingo-format` | Stays **MIT** even when store is MPL |
| 7 | `dingo-sdk` crate split | **Required before** publishing a non-AGPL “SDK” or MIT client; until then `dingo-sdk` is **AGPL-3.0-or-later** (honest for server modules + `dingo-cluster` dep) |

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
│  sda-lib · sda · dingo-format · dingo-client                │
└────────────────────────────▲────────────────────────────────┘
                             │ may depend only upward
┌────────────────────────────┴────────────────────────────────┐
│  MPL-2.0 — linkable embedded engine                         │
│  dingo-store · dingo-examine · (future) embedded-only SDK   │
└────────────────────────────▲────────────────────────────────┘
                             │ may depend only upward
┌────────────────────────────┴────────────────────────────────┐
│  AGPL-3.0-or-later — networked product                      │
│  dingo-cluster · dingo-server · dingo-cli · dingo-sdk       │
│  (sdk remains AGPL until embedded-only extract)             │
└─────────────────────────────────────────────────────────────┘
```

### 3.1 Per-crate SPDX (current tree)

| Crate (dir → package) | SPDX today | Target after full sdk split |
|----------------------|------------|-----------------------------|
| `sda-core` → `sda-lib` | **MIT** | MIT |
| `sda-cli` → `sda` | **MIT** | MIT |
| `dingo-format` | **MIT** | MIT |
| `dingo-client` | **MIT** | MIT (wire framing + handshake) |
| `dingo-store` | **MPL-2.0** | MPL-2.0 |
| `dingo-examine` | **MPL-2.0** | MPL-2.0 |
| `dingo-cluster` | **AGPL-3.0-or-later** | AGPL-3.0-or-later |
| `dingo-server` | **AGPL-3.0-or-later** | AGPL-3.0-or-later |
| `dingo-cli` → `dingo` | **AGPL-3.0-or-later** | AGPL-3.0-or-later |
| `dingo-sdk` | **AGPL-3.0-or-later** *(interim)* | MPL-2.0 embedded-only (after remote/cluster extract) |

### 3.2 License files

| Path | Content |
|------|---------|
| `LICENSE` | Multi-license notice + map |
| `LICENSE-MIT` | MIT full text |
| `LICENSE-MPL-2.0` | MPL-2.0 full text |
| `LICENSE-AGPL-3.0` | AGPL-3.0 full text (project applies **or-later**) |

---

## 4. Split status: `dingo-sdk` was three products

### 4.0 Done (first cut)

| Package | Status | License |
|---------|--------|---------|
| `dingo-client` | **Extracted** — framed RPC + handshake only | MIT |
| `dingo-server` | **Extracted** — accept loop, authz, admission, raft RPC glue, `serve_*` | AGPL-3.0-or-later |
| `dingo-sdk` | Server modules **removed**; still ships remote client, TLS helpers, and `dingo-cluster` for `open_cluster` | AGPL-3.0-or-later *(interim)* |

### 4.1 Remaining modules in `dingo-sdk` (why still AGPL)

| Module group | Natural tier | Status |
|--------------|--------------|--------|
| `collection`, `dingo` (local open), `filter`, `history`, `indexes`, `value`, `receipt`, `error` | **Embedded** (MPL) | Still in sdk |
| `remote`, `directory_cache`, client TLS, connect helpers | **Client** (MIT target) | Still in sdk (re-exports wire from `dingo-client`) |
| `cluster_backend`, `Dingo::open_cluster` | **Networked / AGPL** (pulls `dingo-cluster`) | Still in sdk — blocks MPL label |

**Today:** `dingo-sdk` is honestly **AGPL-3.0-or-later** because it still
depends on `dingo-cluster`. Do **not** publish it as MIT or MPL while that
remains true. Serve path lives only in `dingo-server`.

### 4.2 Target crate apportionment

| Package | Contents | License | Depends on |
|---------|----------|---------|------------|
| `dingo-format` | unchanged | MIT | — |
| `dingo-client` | wire framing + handshake (**done**) | MIT | — |
| `dingo-store` | unchanged | MPL-2.0 | format |
| `dingo-sdk` (embedded-only) | `Dingo::open`, collections, filters, indexes | MPL-2.0 | store, sda-lib optional |
| `dingo-examine` | unchanged | MPL-2.0 | store, format, sda-lib |
| `dingo-cluster` | unchanged | AGPL-3.0-or-later | store |
| `dingo-server` | accept loop, authz, admission, raft RPC glue (**done**) | AGPL-3.0-or-later | sdk, cluster, store |
| `dingo-cli` | CLI + doctor/salvage/serve | AGPL-3.0-or-later | server, sdk, examine |
| `sda-lib` / `sda` | unchanged | MIT | — |

### 4.3 Remaining work (before non-AGPL SDK publish)

1. ~~**`dingo-client`** (MIT) — protocol framing~~ **done**
2. ~~**`dingo-server`** (AGPL) — serve modules out of sdk~~ **done**
3. **`dingo-sdk`** → MPL-2.0 embedded only: move or dual-crate remote + drop
   `dingo-cluster` from default features (or extract `dingo-cluster-client`).

---

## 5. GPL-track matrix (adopted)

```text
MIT                → sda-lib, sda, dingo-format, dingo-client
MPL-2.0            → dingo-store, dingo-examine, (future) embedded-only dingo-sdk
AGPL-3.0-or-later  → dingo-cluster, dingo-server, dingo-cli, dingo-sdk (interim)
```

AGPL protects “networked bits” against pure SaaS freeloading (source offer on
network use). Commercial exception / dual-license for AGPL server and/or MPL
store remains an optional business track; keep pure client and format MIT.

---

## 6. Release checklist

1. **Per-crate `license` in Cargo.toml** — done (no uniform workspace MIT).
2. **LICENSE files** — root multi-license tree (done).
3. **README + CONTRIBUTING** — multi-license notice; inbound = outbound (done).
4. **CLI `--license`** — `sda` MIT; `dingo` AGPL (done).
5. **Do not publish** a MIT- or MPL-labeled `dingo-sdk` that still path-depends
   on AGPL `dingo-cluster` (or any AGPL dep).
6. **`cargo deny` / license policies** — optional hardening before crates.io.
7. **Remaining sdk split** — extract remote/cluster from sdk (or feature-gate)
   before first honest non-AGPL SDK publish. Server extract is **done**.

---

## 7. Compatibility with current dependency edges

Today (after first-cut split):

```text
dingo-cli      → dingo-sdk, dingo-server, dingo-store, dingo-examine  (AGPL)
dingo-server   → dingo-sdk, dingo-cluster, dingo-store               (AGPL)
dingo-sdk      → dingo-client, dingo-store, dingo-cluster, sda-core  (AGPL interim)
dingo-client   → (none of store/cluster)                             (MIT)
dingo-cluster  → dingo-store                                         (AGPL)
dingo-examine  → dingo-format, dingo-store, sda-core                 (MPL)
dingo-store    → dingo-format                                        (MPL)
sda-cli        → sda-core                                            (MIT)
```

All edges respect “stronger may depend on weaker.”

After full embedded extract (§4.3):

```text
dingo-client   (MIT)     → (standalone wire)
dingo-sdk      (MPL)     → dingo-store (MPL) → dingo-format (MIT)
dingo-cluster  (AGPL)    → dingo-store (MPL)
dingo-server   (AGPL)    → dingo-sdk/cluster/store (+ wire via sdk or client)
dingo-cli      (AGPL)    → dingo-server, dingo-sdk, dingo-examine
```

---

## 8. One-paragraph summary

**Adopted:** keep **SDA and the wire format MIT**; keep a **thin network
client MIT** (after extract); put **MPL-2.0 on the embedded store and
examination host**; put **AGPL-3.0-or-later on cluster, the `dingo` operator
binary, and (until split) the combined `dingo-sdk`**. The **`dingo-sdk` crate
must be split** (client vs embedded vs server) before MIT/MPL labels on those
surfaces are honest on crates.io.
