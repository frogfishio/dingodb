# dingo-authority

**AGPL-3.0-or-later** local-only heap authority ceremony tool (`HEAP_SPEC` HP-005).

Issues HeapKeys, commits genesis, cycles masters, and mutates blacklist/grace
through a two-slot authority store under `authority_root`. The qualified data
server (`dingo-server`) MUST NOT depend on this crate or any concrete
`MasterKeyProvider`.

```bash
dingo-authority --license
dingo-authority genesis --help
```
