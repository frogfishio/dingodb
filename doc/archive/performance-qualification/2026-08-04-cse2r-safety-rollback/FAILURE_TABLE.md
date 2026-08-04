# CSE-2R product Materialized restore (guard)

Channels after cold `Store::open`:

- **auth** = `Store::get`
- **chimera** = `Store::get_via_chimera`
- **layout_direct** = `load_chimera_layout` + `ChimeraLayout::get`

## Product seal after rollback (Materialized)

Matches CSE-0 Materialized RHS on every cell (expected — product path *is*
Materialized again):

| Failure | auth | chimera | layout_direct |
|---|---|---|---|
| F0_control | t,m,l | t,m,l | t,m,l |
| F1_wipe_chimera | t,m,l | ∅ | ∅ |
| F2_corrupt_chimera | t,m,l | ∅ | ∅ |
| F3_corrupt_auth_body_t | m,l | t,m,l | t,m,l |
| F4_delete_sealed_segment | ∅ | ∅ | t,m,l |
| F5_corrupt_auth_t_wipe_chimera | m,l | ∅ | ∅ |

## Classification

**Safety rollback** — not Compact parity. Compact SegmentFrame recovery sets
remain those of CSE-1 (FAIL). See `rollback.json`.
