# Formal assurance registries (Wave 0 scaffold)

**Status: incomplete fail-closed scaffold (FA0-W0-T3).**  
Does **not** mean FAS-0 package accept.

Normative file list: `doc/todo/formal-assurance/FORMAL_ASSURANCE_REGISTRY_CONTRACT.md` §1.

- `FAS0_CLOSED` marker file is **absent** → `scripts/check-formal-registry.sh` exits non-zero.
- Only FAS-0-T1/T2 may create `FAS0_CLOSED` after full catalogue + linter exit.
- Theorem stubs may use status `proposed` / `specified` only; no fake proof hashes.

Migration of existing artifacts: `doc/wip/status/FAS_MIGRATION_MAP.md`.
