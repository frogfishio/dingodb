# H6 residual disposition — what needs a third party?

**Date:** 2026-07-30  
**Status:** process honesty (does not flip `qualified=true`)  
**Companions:** [HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md](HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md),
[HEAP_COMPLETE_PATH_REVIEW.md](HEAP_COMPLETE_PATH_REVIEW.md),
[HEAP_NEXT_TASKS.md](HEAP_NEXT_TASKS.md)

After the connected pure-lemma / Kani cut, two Gate H6 blockers remain in common
conversation as “1 & 2”:

| # | Residual | Blocks Level-2 claim? | Needs third party? |
|---|----------|----------------------|--------------------|
| **1** | **CPR-005** — signed **external** security review | **Yes** | **Yes** — by definition |
| **2** | **Verus** machine-checked proofs (`VERUS_PROOFS_CONNECTED`) | **Landed** — pure-kernel lemmas verified under CI `verus-heap` | **No** — in-house (done) |

(Item **3** from the last queue note — live HSM/KMS backend — is an H4 ops
residual, not an H6 claim gate; scaffold is honest “not configured”.)

## 1. CPR-005 — signed external security review

### Can we finish this without a third party?

**No**, not if we keep the word **external** and the Gate H6 bar:

> only a **signed external review** with accepted residual disposition  
> ([HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md](HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md))

What “external” means for this product:

| Role | Acceptable? | Notes |
|------|-------------|--------|
| Independent security firm / researcher under NDA | **Yes** | Preferred; signed PDF under `doc/` |
| Second org / academic group not on the shipping team | **Yes** | Still external |
| Same authors re-read the brief and “sign” it | **No** | Self-review ≠ external; do not flip CPR-005 |
| Automated scanner-only report | **No** | Useful input; not a signed isolation review |

### How to get it (operator checklist)

1. **Ship the pack** (already in-tree): brief, matrix, complete-path review,
   `./scripts/verify-heap.sh quick|full`, Accept drills named in the matrix.
2. **Engage reviewer** with the brief §5 attack questions; fix time-box and
   TCB boundary (single-node `dingo-heap-v1`, not cluster).
3. **Receive signed report** (PDF + hash + signer identity / firm letterhead).
4. **File artifacts**:
   - `doc/HEAP_EXTERNAL_SECURITY_REVIEW_REPORT.md` (summary + link/hash to PDF),
     or store the signed PDF under `doc/` if license allows.
   - Update matrix H6 evidence + disposition open findings.
5. **Disposition findings** (accept / fix / document residual). Only then CPR-005
   can move from open → closed **without** lying about independence.

There is no in-tree labor package that produces a signed external receipt by
coding alone.

## 2. Verus — machine-checked proofs (**landed**)

### Can we finish this without a third party?

**Yes — and it is done in-tree.** Verus is an open-source toolchain; proofs are
first-party engineering.

| Flag | Today | Meaning |
|------|-------|---------|
| `KANI_HARNESSES_CONNECTED` | **true** | `dingo_heap` pure_proofs; CI `kani-heap` |
| `VERUS_PROOFS_CONNECTED` | **true** | `verification/heap-verus/verus/pure_kernel.rs`; CI `verus-heap` |

| How to re-run locally | Command |
|----------------------|---------|
| Install pinned Verus | `./scripts/setup_verus.sh` |
| Verify pure_kernel | `./scripts/check_verus_heap.sh` (or `DINGO_REQUIRE_VERUS=1`) |

Scope honesty: pure_kernel models §39 binding / gen-grace / blacklist /
admission / isolation with integer stand-ins (not full COSE crypto). Executable
Rust lemmas in `dingo_heap::pure_proofs` remain the product-code stand-ins.

## 3. What still blocks `qualified=true`

Even with Kani connected:

| Gate | Open item | Third party? |
|------|-----------|--------------|
| H6 | CPR-005 signed external report | **Yes** |
| H6 | Verus (optional residual) | No |
| H6 | Other H-gates not all accept | No |
| Product | Level-2 marketing language | Requires matrix `qualified=true` |

**Bottom line:**

- **Item 1 (external review):** cannot close honestly without an **independent**
  third party (or independent org). Brief is ready; labor is **procurement +
  engagement + disposition**, not more product code.
- **Item 2 (Verus):** can close **without** a third party; it is optional relative
  to already-connected Kani for the “Verus or Kani” D1 bar.
- **Do not** self-sign CPR-005 or flip `qualified=true` to “finish” the queue.

## 4. Self-check (this disposition)

- [x] CPR-005 requires independence (documented)
- [x] Verus does not require a third party (documented)
- [x] Kani already satisfies D1 “or Kani” path; Verus optional
- [x] Matrix claim remains `qualified=false` until H6 exit criteria met