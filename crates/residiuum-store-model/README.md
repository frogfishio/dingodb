# residiuum-store-model

CSQ-1 / **CSQ-4** sequential logical model oracle for Residiuum core storage.

## CSQ-4

- Publication kernel + `TransitionCoverage` (every ordinary transition)
- Historical reads / gaps / last-complete (DEF-099)
- Coverage-aware key scan (DEF-100 / CSQ-ABS-002)
- Generated command histories + shrinker / exact replay
- Deliberately false harness controls

```bash
cargo test -p residiuum-store-model
cargo test -p residiuum-store-model --test csq4_state_machine
bash scripts/verify-csq-state-machine.sh
```

## Firewall

This crate **MUST NOT** depend on:

- `residiuum-store`
- store index/chunk/compaction/recovery/catalog modules
- production expected-state helpers

Enforced by `scripts/verify-csq-oracle-firewall.sh`.