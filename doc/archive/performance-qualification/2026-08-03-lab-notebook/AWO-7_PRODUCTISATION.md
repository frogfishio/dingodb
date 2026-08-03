# AWO-7 productisation floor

Status: **labor scaffold — not package accept; default-on NOT enabled**  
Date: 2026-08-02  
Card: `e27b864d-3977-46f5-ae4f-82c907709a25`

## Product surface (delivered)

| API | Role |
|---|---|
| `StoreHost::create/open_with_adaptive_write` | Opt-in attach |
| `adaptive_write_status` / `adaptive_write_inspect` | Live + operator report |
| `drain_writes` | Bounded drain + pipeline shutdown |
| `reset_adaptive_write` | Detach lease; natural path resumes |
| `SUPPORT_MATRIX` / upgrade / benchmark strings | Closed disclosure |

## Default posture (G12)

- `AdaptiveWritePolicy::machine_defaults().mode == Disabled`
- Ordinary `create` / `open` do **not** attach AWO
- **Default-on requires principal accept only** — this labor never flips it

## Support matrix (summary)

See `adaptive_write::telemetry::SUPPORT_MATRIX` — eligible vs natural classes,
no Tokio, pipeline depth 2, crash matrix honesty, PQH G8 not claimed by smoke.

## Upgrade / rollback

See `UPGRADE_ROLLBACK_NOTE`: attach is opt-in; rollback is reopen without attach
or `reset_adaptive_write` / mode disabled. No format migration from AWO alone.

## Explicit residuals

1. SDK client docs beyond inspect strings  
2. Server config keys / metrics export  
3. Full multi-process crash campaign per failpoint  
4. PQH controlled qualification (E3)  
5. **Default-on** principal decision  

## Exit commands

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_productisation -- --test-threads=1
cargo test -p residiuum-store --features legacy-raw-store --test awo_crash_matrix -- --test-threads=1
```
