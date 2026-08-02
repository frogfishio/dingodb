# AWO golden vectors (verification tree)

Canonical golden decisions live in the executable contract tree:

```text
spec/performance/awo/golden-decisions-v1.json
```

This directory holds a **symlink** to that file so formal/verification
tooling can resolve goldens under `verification/awo/` without forking the
closed set.

Do not edit a second copy. Amend `spec/performance/awo/` and re-run:

```bash
bash scripts/verify-awo-contract.sh
bash scripts/verify-awo.sh
```
