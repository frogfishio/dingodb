# Core-storage qualification registries (CSQ-0)

Closed-world registries for profile `residiuum-core-storage-v1`.

## Validate

```bash
bash scripts/verify-core-storage-registry.sh
cargo test -p residiuum-store --test csq0_registry
```

## Authority

- [CORE_STORAGE_QUALIFICATION_SPEC.md](../../../doc/todo/core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md) §5
- [CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md](../../../doc/todo/core-storage/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md) §4
- Identity: [REBRAND_PROTOCOL_IDENTITY_RESET.md](../../../doc/done/rebrand/REBRAND_PROTOCOL_IDENTITY_RESET.md)

## Crash matrix import

Historical failpoint names from `crates/residiuum-store/crash_matrix.v1.json`
are preserved in `crash-matrix-import-v1.json` (`historical_cell_id`).
