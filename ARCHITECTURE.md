# DingoDB architecture map

This file points implementers at the **normative specs** and the **crate layout**.
It is not a second architecture document.

## Product thesis

> Put anything in. Keep it at scale. Damage it. Find what survived.

Governing recovery rule: *What is gone is gone. What remains still lives.*

## Normative documents

| Concern | Document |
|---------|----------|
| System architecture, storage model, recovery, quality bars | [OVERVIEW.md](OVERVIEW.md) |
| Survival wire format, frames, segments, scanner tests | [FORMAT_SPEC.md](FORMAT_SPEC.md) |
| Everyday API, CLI, progressive disclosure | [DX_SPEC.md](DX_SPEC.md) |
| Structured Data Algebra (standalone language) | [SDA_SPEC.md](SDA_SPEC.md) |
| SDA examination of recovered DingoDB units | [SDA_PROFILE.md](SDA_PROFILE.md) |
| Cluster federation and coverage | [CLUSTER_SPEC.md](CLUSTER_SPEC.md) |
| Product framing | [USP.md](USP.md) |
| Staged delivery and exit criteria | [DELIVERY_PLAN.md](DELIVERY_PLAN.md) |

Prefer amending a named section of a normative doc before inventing new behavior.

## Delivery stages (summary)

See [DELIVERY_PLAN.md](DELIVERY_PLAN.md) for full exit criteria.

| Stage | Focus | Status |
|-------|--------|--------|
| 0 | Repo + CI harness | **done** (workspace, CI, language decision) |
| 1 | SDA standalone (pure) | **§14.1 suite automated**; full freeze open |
| 2 | Wire format + salvage scanner | **2a–2d** — frames, seal, fwd/rev scan, §13 corpus |
| 3 | Single-node store | **3a–3c** — put/get/delete, §16 suite, descriptor + index cache |
| 4 | Collection SDK | **4a–4d** — `dingo-sdk` open, JSON/bytes, scan/stream, filters, `ErrorCode` |
| 5 | SDA examination profile | **done** — `dingo-examine` ExaminationUnit + SDA over salvage |
| 6 | Indexes, catalogs, chunks | **done** — secondary indexes, history, chunks, compact, checkpoints |
| 7 | CLI doctor/salvage + server | **done** — `dingo-cli`, `open_inspect`/`salvage_to`, `Dingo::connect` |
| 8+ | Cluster, tiering | blocked until single-node salvage is real |

## Crate layout (current)

```text
dingodb/
  crates/
    sda-core/       # package name sda-lib; pure SDA (Stage 1)
    sda-cli/        # package name sda; `sda` binary (Stage 1)
    dingo-format/   # frames, seal, fwd/rev scan, §13 corpus (Stage 2a–2d)
    dingo-store/    # single-node append store (Stages 3 + 6 + 7 inspect/salvage_to)
    dingo-sdk/      # collection API + remote connect (Stages 4 + 6 + 7)
    dingo-examine/  # ExaminationUnit + SDA over salvage (Stage 5)
    dingo-cli/      # `dingo` binary: put/get, doctor, salvage, serve (Stage 7)
```

Planned crates (do **not** add until the owning stage starts):

| Stage | Crate (proposed) | Role |
|-------|------------------|------|
| 2 | `dingo-format` | **Present** — frames, segment seal, fwd/rev scanner, §13 corpus (2a–2d) |
| 3+6+7 | `dingo-store` | **Present** — put/get/delete, salvage, open_inspect, salvage_to, catalogs, chunks, history, compact |
| 4+6+7 | `dingo-sdk` | **Present** — collections, filters, indexes, history, `Dingo::connect` RPC client/server helpers |
| 5 | `dingo-examine` | **Present** — ExaminationUnit projection, salvage stream, SDA filter/map, bounded pages |
| 7 | `dingo-cli` | **Present** — `dingo` put/get/list/doctor/salvage/serve (server lives in CLI + sdk remote module) |
| 8 | `dingo-cluster` | Partition ownership, coverage (later) |

Rule of thumb from the delivery plan: **vertical slices over empty package trees.**
Do not scaffold cluster/server before Stage 3 store salvage works.

## Language decisions (Stage 0)

| Choice | Decision |
|--------|----------|
| Core implementation language | **Rust** |
| First embedded surface | Rust library API; TypeScript-like examples in DX_SPEC remain the product shape |
| First CLI | `sda` (Stage 1) + `dingo` (Stage 7) |
| SDA packaging | `sda-lib` (lib) + `sda` (CLI binary); no storage IO inside SDA core |
| Wire format versioning | Draft until wire major 1 freeze after Stage 2–3 soak |
| Default license | MIT (repo root) |

## SDA import convention

- Package name on crates.io path: `sda-lib`
- Workspace dependency key: `sda-core` → Rust path `sda_core::…` (CLI)
- Integration tests of the library use `sda_lib::…`

## Non-goals until listed stages

- Cluster / consensus (Stage 8+)
- Object-store backends and archive tiering (Stage 9)
- Marketing-grade Redis-class latency claims without OVERVIEW §12.2 disclosure
