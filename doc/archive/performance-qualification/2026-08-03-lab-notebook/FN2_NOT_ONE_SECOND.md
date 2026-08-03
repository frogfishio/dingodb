# Did we only test for 1 second? (FN-2)

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *“so we tested it for 1 second?”*

## Short answer

**No.** We did not run a 1-second timed benchmark.

We ran until a **fixed amount of work** was done: **256 MiB logical** payload
(= **32 768** × 8 KiB keys). Wall-clock is whatever that took.

SQLite finished that work in ~**1.1 s** because it was fast (~29k/s). Residiuum
took longer. Static/Adaptive took ~**13 s**.

## FN-2 wall times (same work each cell)

| Cell | keys | elapsed | ops/s |
|------|-----:|--------:|------:|
| SQLite A | 32 768 | **~1.1 s** | ~29 200 |
| Residiuum-off | 32 768 | **~2.6 s** | ~12 600 |
| Residiuum Static | 32 768 | **~13.3 s** | ~2 460 |
| Residiuum Adaptive | 32 768 | **~13.3 s** | ~2 470 |

Stop rule in peer-pump: keep putting until `keys × payload_size ≥ target_bytes`
(default `256M`). Not “run for T seconds.”

## Why SQLite looks like “1 second”

```text
time ≈ keys / rate
     ≈ 32768 / 29200  ≈ 1.1 s
```

Same keys at Residiuum-off ~12.6k → ~2.6 s. Same keys at ~2.5k → ~13 s.
The clock follows the rate; we did not choose “1 second” as the test length.

## Still diagnostic

~1–13 s walls / 256 MiB is a **short peer cell**, not a sustained soak or PQH
qualification. Fine for odometer comparison; not a 120 s product floor.

## Source

`artifacts/firm-numbers-fn2-mode-a-apfs/*.json` — `elapsed_ms`, `keys_written`,
`target_bytes`.
