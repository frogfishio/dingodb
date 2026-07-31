# Core-storage qualification registries (CSQ-0)

Closed-world registries for profile `residiuum-core-storage-v1`.

## Validate

```bash
bash scripts/verify-core-storage-registry.sh
bash scripts/verify-csq-oracle-firewall.sh
bash scripts/verify-csq-boundary-instrumentation.sh
cargo test -p residiuum-store --test csq0_registry
cargo test -p residiuum-store --features legacy-raw-store --test csq2_instrumentation
bash scripts/verify-csq-format-corpus.sh
cargo test -p residiuum-format --test csq3_format_corpus
bash scripts/verify-csq-state-machine.sh
cargo test -p residiuum-store-model --test csq4_state_machine
```

CSQ-2 also exercises the DEF-022 crash matrix driver:

```bash
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_022_crash_matrix
```

## Authority

- [CORE_STORAGE_QUALIFICATION_SPEC.md](../../../doc/todo/core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md) §5
- [CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md](../../../doc/todo/core-storage/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md) §4
- Identity: [REBRAND_PROTOCOL_IDENTITY_RESET.md](../../../doc/done/rebrand/REBRAND_PROTOCOL_IDENTITY_RESET.md)

## Crash matrix import

Historical failpoint names from `crates/residiuum-store/crash_matrix.v1.json`
are preserved in `crash-matrix-import-v1.json` (`historical_cell_id`).