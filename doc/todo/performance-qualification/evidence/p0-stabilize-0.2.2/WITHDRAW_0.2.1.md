# Withdrawal — residiuum 0.2.1

**Status:** Published packages **yanked** on crates.io (2026-08-04).  
**Keep yanked:** 0.2.0 (segment-ID remint).  
**Do not push** `main` / `v0.2.1` as an approved stable release.

## Why

P0 segment-identity / immutable-publication engineering is retained, but packaging
0.2.1 shipped against a **red** full `residiuum-store` suite (~52 failures / 24
targets) touching durability cores: damage/salvage, tiering, coverage, backup,
and synchronous `seal_active`. That is not acceptable for a security/data-integrity
release.

## Tag

Local annotated tag `v0.2.1` → commit `7285c1e312e46b912c10108f4acbdb244e0aa178`
is preserved as the exact published-but-withdrawn source. **Do not move the tag.**

## Next

Stabilization packaging **0.2.2**: keep P0 fix; reconcile every suite failure
(contract update vs regression); full store suite green; re-run P0 1000-cycle +
collision matrices; publish/tag only afterward.

Yank log: `cargo_yank_0.2.1.log`
