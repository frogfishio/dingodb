# CSE-3 Stage 1 — Hybrid Recovery Shadow (2026-08-04)

Status: **principal-accepted** — Stage 1 specification complete with
deletion/lifecycle addendum.  
Principal: **C — Hybrid**.

```text
Compact Chimera → query acceleration; tiny, derived, disposable
Recovery Shadow → full-copy salvage; authoritative recovery artifact
```

Preserves P★ at ≈100% recovery overhead; separates cheap sequential Shadow
construction from query-oriented Chimera layout work. Materialized remains
product until Shadow passes CSE equivalence.

**Addendum (2026-08-04):** tombstones, latest-generation merge, compaction
coverage, retention/secure delete, backup/scrub/encryption, partial-file
exclusion from P★, and observable `protected_frontier`. Stage 2 gates listed
in the spec.

## Artifacts

| Path | Role |
|---|---|
| Spec (todo) | `doc/todo/performance-qualification/CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md` |
| Spec (copy) | `CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md` (this archive) |
| Machine summary | `stage1.json` |

## Non-claims

No Shadow code landed; no product flip; no ETQ-2 resume; ack ≠ P★.
