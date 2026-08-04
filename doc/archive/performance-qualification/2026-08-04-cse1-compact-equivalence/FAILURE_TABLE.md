# CSE-1 Compact vs Materialized recovery table

Channels after cold `Store::open` (identical to CSE-0):

- **auth** = `Store::get`
- **chimera** = `Store::get_via_chimera` (index-gated)
- **layout_direct** = `load_chimera_layout` + `ChimeraLayout::get` (format-only; no store pread)

## Compact measured (LHS)

| Failure | auth | chimera | layout_direct |
|---|---|---|---|
| F0_control | t,m,l | t,m,l | ∅ |
| F1_wipe_chimera | t,m,l | ∅ | ∅ |
| F2_corrupt_chimera | t,m,l | ∅ | ∅ |
| F3_corrupt_auth_body_t | m,l | m,l | ∅ |
| F4_delete_sealed_segment | ∅ | ∅ | ∅ |
| F5_corrupt_auth_t_wipe_chimera | m,l | ∅ | ∅ |

## Materialized frozen RHS (CSE-0)

| Failure | auth | chimera | layout_direct |
|---|---|---|---|
| F0_control | t,m,l | t,m,l | t,m,l |
| F1_wipe_chimera | t,m,l | ∅ | ∅ |
| F2_corrupt_chimera | t,m,l | ∅ | ∅ |
| F3_corrupt_auth_body_t | m,l | t,m,l | t,m,l |
| F4_delete_sealed_segment | ∅ | ∅ | t,m,l |
| F5_corrupt_auth_t_wipe_chimera | m,l | ∅ | ∅ |

## Inequality result

\(\mathrm{Recoverable}_{compact} \supseteq \mathrm{Recoverable}_{materialized}\) — **FAIL**

Gaps (keys Materialized recovers that Compact does not):

| Failure | Channel | Missing |
|---|---|---|
| F0 | layout_direct | t,m,l |
| F3 | chimera | t |
| F3 | layout_direct | t,m,l |
| F4 | layout_direct | t,m,l |

**Requires CSE-2** (minimum parity). Root cause: Compact `SegmentFrame` locators need segment pread; no embedded payloads → no format-only salvage and no ChimeraGet expansion when the establishing frame is damaged.
