# CSE-0 failure / recovery table

Channels after cold `Store::open`:

- **auth** = `Store::get`
- **chimera** = `Store::get_via_chimera` (index-gated)
- **layout_direct** = Materialized `.cmr` resolve without index

| Failure | auth | chimera | layout_direct | Notes |
|---|---|---|---|---|
| F0_control | t,m,l | t,m,l | t,m,l | Baseline healthy |
| F1_wipe_chimera | t,m,l | ∅ | ∅ | Auth independent of Chimera |
| F2_corrupt_chimera | t,m,l | ∅ | ∅ | Fail-closed sidecar load |
| F3_corrupt_auth_body_t | m,l | t,m,l | t,m,l | Materialized expands ChimeraGet for damaged `t` |
| F4_delete_sealed_segment | ∅ | ∅ | t,m,l | Format recovers; product channels need index+segment |
| F5_corrupt_auth_t_wipe_chimera | m,l | ∅ | ∅ | No invented exact `t` |

## Oracle freezes for CSE-1

Compare Compact under the **same** \(F\) and channels. Pass only if for every
\(f \in F\):

\[
R_{\mathrm{compact}}(f,c) \supseteq R_{\mathrm{materialized}}(f,c)
\]

for each channel \(c \in \{\mathrm{auth},\mathrm{chimera},\mathrm{layout\_direct}\}\).

Materialized sets above are the frozen RHS.
