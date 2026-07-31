# residiuum-store-model

CSQ-1 sequential logical model oracle for Residiuum core storage.

## Firewall

This crate **MUST NOT** depend on:

- `residiuum-store`
- store index/chunk/compaction/recovery/catalog modules
- production expected-state helpers

Enforced by `scripts/verify-csq-oracle-firewall.sh`.
