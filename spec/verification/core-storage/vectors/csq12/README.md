# CSQ-12 evidence bundle vectors

## Command

```bash
# Stand-in for residiuum verify --profile residiuum-core-storage-v1 --level A2
bash scripts/residiuum-verify-core-storage.sh --profile residiuum-core-storage-v1 --level A2
```

Produces under `target/csq-evidence/` (override with `RESIDIUUM_CSQ_EVIDENCE_DIR`):

- `residiuum-core-storage-report-v1.json` — core attachment
- `residiuum-verification-report-v1.json` — envelope
- `a2-evaluation.json` — independent A2 evaluator output

## Honesty

- Profile must be `residiuum-core-storage-v1` (not `dingo-core-storage-v1`).
- `not_run`, `infrastructure_failure`, retry-to-green, and prose cannot satisfy A2.
- A declared `result=pass` with missing or non-pass cells is **rejected** by the verifier.
- First labor cut lists exact residual gates in `missing_cells` (predecessor accept,
  full boundary/platform/soak/mutation/publication gates).

## Retention policy

See report attachment `retention_policy` (`residiuum-csq-evidence-retention-v1`):
failures retained; minimization never replaces originals; retries are additional
evidence; infrastructure failure never satisfies a gate.
