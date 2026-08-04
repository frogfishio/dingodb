# CSE-3 Stage 0 — Materialized recovery bound

Status: **labor complete** (analysis / proof only — **no** code selection).  
Date: 2026-08-04.  
Depends: CSE-0 baseline, CSE-1 Compact FAIL, CSE-2R safety rollback.  
Charter: [`CSE3_COMPACT_RECOVERY_CODE.md`](./CSE3_COMPACT_RECOVERY_CODE.md),
[`CHIMERA_SALVAGE_EQUIVALENCE.md`](./CHIMERA_SALVAGE_EQUIVALENCE.md).

## Question

> What is the strongest precisely stated damage pattern that Materialized
> Chimera guarantees recovery from?

That bound decides whether a Compact+reduced-overhead recovery layer can be
mathematically equivalent.

## 1. Exact Materialized recovery set (CSE-0 channels)

Freeze scope: one sealed segment \(S\) whose Materialized `.cmr` was built from
the live \((key, value)\) pairs established on \(S\) at seal
(`build_materialized_layout`). Let \(V_S\) be that embedded live value set
(exact bodies). Let \(C_S\) be the Chimera sidecar file for \(S\).

| Channel | Recovers \(V_S\) when… |
|---|---|
| `layout_direct` | \(C_S\) loads and decodes; **no** segment bytes required |
| `chimera` (`get_via_chimera`) | \(C_S\) OK **and** PrimaryIndex still has a live entry for the key |
| `auth` (`Store::get`) | Independent of Chimera — needs index + readable establishing frame |

From frozen \(F\) (CSE-0):

| Failure | Materialized Chimera contribution |
|---|---|
| F0 | All channels recover \(V_S\) |
| F1 / F2 | Chimera gone/corrupt → Chimera channels empty; auth intact |
| F3 | Auth loses damaged key; **`chimera` + `layout_direct` still yield exact body** from embed |
| F4 | Segment file deleted → auth + product `chimera` empty (index rebuild); **`layout_direct` recovers all of \(V_S\)** from \(C_S\) alone |
| F5 | Auth damaged + Chimera wiped → no invented exact body |

### Strongest precise pattern (Chimera format guarantee)

**P★ — Total authoritative-segment payload loss with Chimera sidecar intact**

Assumptions:

- \(C_S\) remains present, magically valid, and decodable for store/segment ids.
- Values in \(V_S\) are exactly the bodies embedded at seal (generation field on
  locators is the layout generation used at encode — see §2).

Guarantee:

\[
\operatorname{Recoverable}_{\mathrm{layout\_direct}}(\mathrm{P}^\star)
= V_S
\]

i.e. after **complete destruction** of segment \(S\)’s on-disk payloads (F4),
or **arbitrary corruption** of establishing item bodies while \(C_S\) is intact
(F3 for each damaged key), Materialized Chimera still yields every exact body
in \(V_S\) via format-only resolve.

Product `chimera` is weaker than P★ under F4 because it is **index-gated**; the
**format** guarantee that CSE-1/CSE-3 must match for `layout_direct` is P★.

## 2. Live values vs historical generations

| Question | Answer |
|---|---|
| What is embedded? | **Current live values** for subjects whose establishing put on \(S\) was selected into the seal layout (last-wins per subject on that segment / live index projection). |
| Historical generations? | **Not** covered by Materialized Chimera. Superseded values, deleted subjects, and prior generations live in segment history / subject-history derived indexes — not in \(C_S\). |
| Layout `generation` field? | Relocation / layout generation baked into locators; **not** a multi-generation value archive. |

So P★ is a **live-set** salvage guarantee for the sealed projection, not a
full multi-version database guarantee.

## 3. Independence / placement assumptions

| Assumption | CSE-0 evidence | Honest limit |
|---|---|---|
| File separation | F4 deletes `segments/{S}.residiuum` while leaving `indexes/chimera/{S}.cmr` | Same store directory tree; **same volume** in the campaign |
| Independent media / failure domains | **Not proven** | A disk that loses both files does not satisfy P★ |
| Derived-only | Chimera loss must not block segment salvage (F1: auth still works) | Symmetric: segment loss must not be required for Chimera load |

P★ therefore assumes **sidecar survival independent of segment-file survival**,
not geo/media independence unless operators place Chimera elsewhere.

## 4. Information-theoretic minimum redundancy

Let \(L = \sum_{v \in V_S} |v|\) be the total payload length of the live set
(incompressible / adversarial).

**Theorem (P★ lower bound).** Any recovery scheme that, after total loss of
all authoritative segment bytes for \(V_S\), still reconstructs every \(v \in
V_S\) exactly, must retain at least \(L\) bits of **independent** information
about \(V_S\) outside the destroyed segment.

*Proof sketch.* If fewer than \(L\) independent bits remain, there exist two
distinct live sets \(V, V'\) of length \(L\) (incompressible) consistent with
the surviving metadata; the decoder cannot map both to the unique correct
bodies. Hence ≥ \(L\) bits of independent redundancy are necessary.

Materialized Chimera meets the bound by storing ≈ \(L\) bits of embedded
payloads in \(C_S\) (≈ **100% Chimera amplification** for those values, matching
observed ~98% class amp).

Compact SegmentFrame stores locators (segment id, frame offset, body_len) —
\(o(L)\) metadata, **zero** independent payload bits. Under P★ it cannot
recover \(V_S\).

## 5. Reduced-overhead Compact equivalence — impossible for P★

**Corollary.** No Compact+recovery design with redundancy \(o(L)\) (including
XOR with one parity stripe at \(1/k\), or MDS \(k+m\) with \(m/k \ll 1\) when
\(k\) is large) can match **P★** for incompressible \(V_S\).

| Candidate weaker pattern | Overhead class | Matches P★? |
|---|---|---|
| Loss of any 1 of \(k\) **segments** (need one other segment) | \(~1/k\) | **No** — P★ is total loss of **the** establishing segment’s payloads with only \(C_S\) surviving; other segments do not hold \(V_S\) bodies under Compact |
| Up to \(m\) missing fragments **within a stripe that already stores the data** | \(m/k\) | **No** unless the stripe already contains ≥ \(L\) data symbols — then overhead is on top of full data, not a substitute for the Materialized full copy |
| Full independent replica (Materialized embed / second copy) | \(~100\%\) | **Yes** — this *is* Materialized |

Therefore:

- Matching Materialized’s **strongest CSE-0 format guarantee (P★)** requires
  ≈100% independent redundancy for incompressible live data.
- **Reduced-overhead equivalence to P★ is information-theoretically impossible.**
- XOR / Reed–Solomon / other codes may still be valuable for a **explicitly
  weaker, named** failure set (e.g. “any one of \(k\) independently placed
  Chimera parity extents”), but that is a **different product claim**, not
  Compact≡Materialized under CSE-0 `layout_direct` / F3–F4.

### Stage 0 decision (no codec selection)

| Option | Meaning |
|---|---|
| **A — Keep P★** | Product salvage continues to require full-copy redundancy (Materialized or equal). Compact remains ETQ-only for amp/TPS. |
| **B — Weaken claim** | Principal renames the durability target to a smaller pattern; then Stage 1 may select XOR/MDS against that pattern. |
| **C — Hybrid** | Compact hot path + optional full-copy salvage tier (still ≈100% when salvage tier is on). |

**Stage 0 does not choose XOR vs RS.** Codec choice is blocked until principal
picks A/B/C (or an equally precise alternative failure set).

## Required Stage 0 checklist

| # | Output | Status |
|---|---|---|
| 1 | Exact Materialized recovery set | Done — \(V_S\) via P★ / CSE-0 table |
| 2 | Live vs historical | Done — **live sealed projection only** |
| 3 | Independence / placement | Done — file-level sidecar vs segment; not multi-media |
| 4 | Info-theoretic min redundancy | Done — ≥ \(L\) bits for P★ |
| 5 | Proof Compact reduced-overhead covers same set **or** impossibility | Done — **impossibility** for P★ |

## Non-claims

- Does not implement recovery code.
- Does not flip product default to Compact.
- Does not resume ETQ-2.
- Does not claim Materialized recovers historical generations.
- Does not claim Chimera survives shared-disk total loss.

## Evidence / next

- Archive: `doc/archive/performance-qualification/2026-08-04-cse3-stage0-recovery-bound/`
- Principal fork: choose **A / B / C** before any Stage 1 codec labor.
