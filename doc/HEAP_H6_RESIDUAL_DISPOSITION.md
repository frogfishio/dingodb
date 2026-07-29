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
| **2** | **Verus** machine-checked proofs (`VERUS_PROOFS_CONNECTED`) | **No** (optional if Kani already covers the pure kernel obligations) | **No** — in-house toolchain + proof engineering |

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

## 2. Verus — machine-checked proofs

### Can we finish this without a third party?

**Yes.** Verus is an open-source Rust verification toolchain. Running it and
landing proofs is **first-party engineering**, same class as Kani (already
connected).

| Piece | In-house? | Third party? |
|-------|-----------|--------------|
| Install Verus in CI | Yes | No |
| Write `spec fn` / proof over pure decide + models | Yes | No |
| Flip `VERUS_PROOFS_CONNECTED=true` when CI is green | Yes | No |
| External auditor “blesses” the proofs | Optional | Nice-to-have, not required for the flag |

### Relationship to Kani (already landed)

Gate D1 was: connect **Verus *or* Kani** to pure decide / isolation Inv in CI.

| Flag | Today | Meaning |
|------|-------|---------|
| `KANI_HARNESSES_CONNECTED` | **true** | Harnesses in `dingo_heap` pure_proofs; CI job `kani-heap` |
| `VERUS_PROOFS_CONNECTED` | **false** | Verus project still a scaffold |

So **CPR-004 is partially satisfied by Kani**. Verus is a **stronger / alternate**
machine-checked path over the same pure predicates, not a hard second dependency
for “or Kani”. Keeping Verus open is honest engineering debt, not a vendor
blocker.

### How to get Verus (if we still want it)

1. Install Verus (and keep versions pinned in CI docs).
2. Port pure targets listed in `verification/heap-verus` /
   `VERUS_TARGET_PREDICATES` to verified modules (or prove the existing pure
   functions via a Verus-friendly boundary).
3. Add CI job analogous to `kani-heap`.
4. Only then set `VERUS_PROOFS_CONNECTED = true` and update
   `h6_pure_proof_bundle_connected` honesty asserts.

Until then: **do not** set the flag, and **do not** claim Verus-checked isolation.

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
