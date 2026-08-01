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

- Profile must be `residiuum-core-storage-v1` (never a pre-reset product profile id).
- `not_run`, `infrastructure_failure`, retry-to-green, and prose cannot satisfy A2.
- A declared `result=pass` with missing or non-pass cells is **rejected** by the verifier.

## A2 vs A3 gates (work acceptance)

**Package labor acceptance** (board `in_review` → principal `done`) is **independent**
of full A2 pass. Principal may accept CSQ-0…11 labor floors without green A2.

| Gate | Level | Closes when |
|---|---|---|
| `CSQ12-GATE-PREDECESSOR-ACCEPT` | A2 | Scoreboard `CSQ-0`…`CSQ-11` are **`accept`** (not merely `active` / board `in_review`) |
| `CSQ12-GATE-FULL-BOUNDARY-MATRIX` | A2 | Boundaries registry + INSTRUMENT/CRASH suites active + verify scripts present |
| `CSQ12-GATE-P0-MUTATION-CATALOG` | A2 | Every mandatory P0 mutant has kill owners + MUT suite active |
| `CSQ12-GATE-INDEPENDENT-BUNDLE-PUBLICATION` | A2 | Independent builder/verifier + wrapper + vectors present |
| `CSQ12-GATE-PLATFORM-MATRIX` | **A3** | Multi-platform CI execution (registry alone insufficient) |
| `CSQ12-GATE-SOAK-72H` | **A3** | 72h/1B-op campaign (or `RESIDIUUM_CSQ_SOAK_PASSED=1` after attestation) |
| `CSQ12-GATE-FULL-MUTATION-THRESHOLD` | **A3** | Broader mutation % beyond P0 sentinel catalog |

`--level A2` does **not** list A3 campaign gates as missing.
`--level A3` adds them.

## Retention policy

See report attachment `retention_policy` (`residiuum-csq-evidence-retention-v1`):
failures retained; minimization never replaces originals; retries are additional
evidence; infrastructure failure never satisfies a gate.