# CSE-3 Stage 2 step 6 — CSE F0–F5 + lifecycle/security matrix

Status: **labor complete** (2026-08-04) — matrix implemented and green.
**No performance work. No product flip.** Materialized remains recovery
authority during dual-run.

Depends: Stage 2a/2b (`CSE3_STAGE2_RECOVERY_SHADOW_IMPLEMENT.md`).

## Dual-run vs post-flip reclaim (locked)

> During dual-run, Materialized may satisfy recovery authority. After the
> flip, reclaim must **always** require durable replacement Shadow coverage;
> “when present” is no longer sufficient.

| Policy | When | Reclaim without replacement `.rsh` |
|---|---|---|
| `DualRunMaterializedAuthority` | Steps 1–7 (default) | Allowed (Materialized covers) |
| `RequireReplacementShadow` | Step 8+ product flip | **Refused** — never retire last valid recovery source |

API: `ShadowReclaimPolicy` / `set_shadow_reclaim_policy`. Compaction checks
replacement **before** deleting sources under post-flip policy.

## Required matrix

| Id | Requirement | Test |
|---|---|---|
| **F0** | Materialized and Shadow reconstruct identical \(V_S\) | `f0_healthy_materialized_eq_shadow` |
| **F1** | Deleting Chimera/Compact does not affect Shadow recovery | `f1_index_loss_shadow_intact` |
| **F2** | Shadow corruption/truncation fails closed | `f2_shadow_damage_fail_closed` |
| **F3** | Total authoritative loss reconstructs from Shadow | `f3_authoritative_loss_shadow_reconstructs` |
| **F4** | Overwrites/tombstones → latest gen, no resurrection | `f4_generation_tombstone_no_resurrection` |
| **F5** | Lifecycle interruption: partial Shadow never P★ | `f5_lifecycle_partial_shadow_never_p_star`, `f5_frontier_update_after_publish` |

## Mandatory variants

| Variant | Test |
|---|---|
| Multi-shard gaps / out-of-order | `variant_multi_shard_gaps` |
| Wrong-store / wrong-segment substitution | `variant_wrong_store_segment_substitution` |
| Backup → restore → auth loss | `variant_backup_restore_auth_loss` |
| Encrypted Shadow has no plaintext | `variant_encrypted_no_plaintext` |
| Key rotation preserves recovery | `variant_key_rotation_preserves_recovery` |
| Crypto erase unrecoverable | `variant_crypto_erase_unrecoverable` |
| Compaction cannot retire last source | `variant_compaction_cannot_retire_last_source` |
| Dual-run Materialized authority | `clarification_dual_run_allows_materialized_authority` |

## Evidence

```text
cargo test -p residiuum-store --test cse3_stage2_shadow_f0_f5
```

Code: `crates/residiuum-store/tests/cse3_stage2_shadow_f0_f5.rs`  
Envelope crypto: `recovery_shadow/crypto.rs` (BLAKE3 keyed; CSE hook, not prod AEAD claim).

## Non-claims

- Does **not** flip product sealing.
- Does **not** claim ≥7 seg/s (step 7).
- Envelope crypto is a CSE security harness, not a production encryption product gate.
