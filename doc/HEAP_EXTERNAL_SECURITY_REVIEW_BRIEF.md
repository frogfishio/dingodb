# External security review brief — heap authorization (HP-010 / Gate H6)

**Status:** ready for external engagement; review **not** yet completed  
**Date:** 2026-07-29  
**Product claim today:** Level 1 namespaces only (`qualified=false`)  
**Target claim after successful review + residual close:** Level 2  
  *“Cryptographically authorized systems are logically isolated between heaps.”*

This brief packages scope, TCB, evidence, and attack questions for an independent
reviewer. Completing this document does **not** satisfy Gate H6; only a signed
external review with accepted residual disposition does.

## 1. Review objective

Assess whether the **single-node** `dingo-heap-v1` isolation kernel and authority
path prevent cross-heap data observation/mutation for cryptographically authorized
holders, under the published limitations (admin, physical access, side channels).

Out of scope for this engagement (unless contracted separately):

- Cluster leases / Raft control plane (HC1 / HP-011+)
- Full product pentest of CLI/UX
- Side-channel lab measurements beyond published limitations

## 2. Assets and threat model summary

| Asset | Protection goal |
|-------|-----------------|
| Application data labelled to heap H | Unreadable / unwritable without cap for H |
| Heap identity / authority chain | No silent epoch/generation rollback without ceremony |
| Operational observations | No cross-heap leak via metrics/logs/export/health/support |
| Recovery paths | Payload-only restore grants no ordinary access |

Attacker classes (HEAP_SPEC + this brief):

1. **Authorized holder of heap A** trying to touch heap B (logical isolation)
2. **Network client** with TLS but without valid HeapKey / holder proof
3. **Faulty/malicious planner** inside TCB attempting unconstrained scan
4. **Operator with recovery tools** (expected privileged; document residual)
5. **Physical / process compromise** (out of logical claim; published limitation)

## 3. TCB and repository map

| Component | Path | Role |
|-----------|------|------|
| Isolation kernel | `crates/dingo-heap` | decide, caps, constraints, confinement |
| Ownership / admit | `crates/dingo-format` | SubjectV2, frame admit |
| Store façades / lifecycle | `crates/dingo-store/src/heap/` | catalogs, purge, restore, migration |
| Local ceremony | `crates/dingo-authority` | issue / cycle (not linked by server) |
| Qualified network | `crates/dingo-server` heap_* + client handshake | HeapKey session |
| Spec | `HEAP_SPEC.md`, `spec/heap/` | Normative contract |
| Formal sketches | `formal/heap/*.tla` | Isolation + Authority models |
| Executable obligations | `decide_obligations`, IsolationModel, AuthorityModel | CI stand-ins |
| Qualification matrix | `spec/heap/qualification/hp010-matrix-v1.json` | Evidence index |
| Complete-path review | `doc/HEAP_COMPLETE_PATH_REVIEW.md` | Path inventory |

## 4. Evidence pack (reproducible)

```bash
./scripts/verify-heap.sh quick   # architecture + HP-005…HP-010 Accept
./scripts/verify-heap.sh full    # + TLC/Kani when installed
```

Key Accept drills (see matrix `drills`):

- Differential NI, query escape, single-owner admit
- Operational / metadata-hardened confinement
- Key-loss, restore payload-only, DR retain-ID
- Lifecycle crash-matrix (peer heaps unaffected)
- §39 decide obligations + connected TLA models

## 5. Suggested attack questions for the reviewer

1. Can a valid `HeapCap` for A ever observe or mutate B’s objects via query,
   index, history, stream, SDA, export, or support bundle paths?
2. Can frame/salvage paths attribute foreign ownership under damage?
3. Can authority generation grace or blacklist be bypassed to revive a revoked
   holder within the same process?
4. Does security-revision / chain-head refresh terminate all live caps?
5. Does the qualified TLS path reject token/RBAC shortcuts that would skip
   HeapKey?
6. Are published limitations accurate and complete for the claim level?
7. Is any default-on API (flat `collection(name)`) likely to be confused with
   the qualified profile by integrators?

## 6. Known residuals (must appear in review report)

| ID | Residual | Disposition required |
|----|----------|----------------------|
| CPR-001 | Flat SDK / legacy raw store still default-adjacent | Exclude from claim or rebind |
| CPR-002 | Reserved ops not fully activated (§32.4) | Confirm matrix incomplete ≠ bypass |
| CPR-004 | Verus/Kani not yet proving pure kernel | Treat formal stack as incomplete |
| CPR-005 | This external review itself | Produce signed report |
| HP-009 | Live FS tier wipe / HSM adapters | Ops residual; document |

## 7. Acceptance of review (product rule)

Gate H6 may advance only when:

1. Reviewer report is archived under `doc/security-reviews/` (or equivalent),
2. Every High/Critical finding is fixed or explicitly accepted with product
   owner sign-off,
3. Matrix H6 evidence lists the report path,
4. `qualified` remains false until **all** §27 H6 bullets are satisfied — not
   merely until the review meeting occurs.

## 8. Contact / version pin

- Spec status: HEAP_SPEC v0.9 Implementation progress date on cover
- Git: record commit SHA under review in the external report
- Profile: single-node `heap-data-isolated` (H6 minimum); optionally
  `heap-metadata-hardened` operational surface
