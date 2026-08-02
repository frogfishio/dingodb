# AWO formal artifacts

Profile: `residiuum-adaptive-write-v1`

| Path | Role |
|---|---|
| `tla/AdaptiveWrite.tla` | AWO-0 skeleton: plan §17 variables/transitions + core invariants |
| `tla/AdaptiveWrite.cfg` | TLC constants (2 requests, 1 lane, small bounds) |
| `verus/model.rs` | Pure-kernel stub; deepen in AWO-6 |

Executable goldens (shared with product contracts):

- `spec/performance/awo/golden-decisions-v1.json`
- linked from `verification/awo/golden/`

Verify entry (AWO-0):

```bash
bash scripts/verify-awo.sh
```

Optional TLC (if `tlc` / `java -cp tla2tools.jar tlc2.TLC` is available):

```bash
# from repo root, tools permitting
tlc -config formal/awo/tla/AdaptiveWrite.cfg formal/awo/tla/AdaptiveWrite.tla
```

AWO-0 does not require TLC green as a hard CI gate; presence of the skeleton
and `verify-awo.sh` source checks is the package exit. Full model campaigns
are AWO-6.
