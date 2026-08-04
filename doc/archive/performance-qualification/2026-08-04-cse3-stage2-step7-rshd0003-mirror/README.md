# CSE-3 Stage 2 step 7 — RSHD0003 canonical segment mirror (2026-08-04)

Status: **labor complete / performance gate still OPEN** — wire+I/O change landed;
2 GiB/64 MiB still does not clear ≥7 seg/s on this host under contested disk.
**No product flip.** Step 8 remains blocked. Next experiment (principal):
write-time dual streaming — only after this mirror evidence.

## Wire (RSHD0003)

```text
Authoritative:  [segment header | encoded frames | summary]
Recovery Shadow: [envelope | exact canonical segment image | commitment]

magic[8] = "RSHD0003"
store_id[16] segment_id[16] encoded_len[u64 LE]
image[encoded_len]          # exact sealed .residiuum bytes
commitment[32] = blake3(envelope‖image)
```

- Physical buffered copy only (no APFS clone/reflink).
- Durability remains `sync_all` → rename → dir sync.
- V1/V2 record Shadows remain readable; product dual-run / qualify use V3 mirror.
- Salvage: scan mirrored image with the existing segment scanner.

## What landed

- `recovery_shadow/mirror.rs` — encode/publish/load + `mirror_to_decoded_shadow`
- Qualify / seal dual-run path publishes mirrors of sealed bytes (`encode_ns=0`)
- Step 7 rate accounts Shadow publish only (Compact decode excluded from ≥7 gate)
- Step 6 CSE matrix still green after the change

## Campaign results (release harness)

| Cell | Log | Shadow pub | encode_ns | Gates |
|---|---|---:|---:|---|
| 256 MiB quiet | `cse3-step7-rshd3-256m2.log` | **9.96 seg/s** | 0 | **PASS** (incl. ≥7) |
| 2 GiB/64 MiB best | `cse3-step7-rshd3-2g.log` | **6.99 seg/s** | 0 | FAIL ≥7 (safety/amp/recovery PASS) |
| 2 GiB contested | `cse3-step7-rshd3-2g-b.log` | 3.49 | 0 | FAIL |
| 2 GiB contested | `cse3-step7-rshd3-2g-c.log` | 2.30 | 0 | FAIL |

### Best 2 GiB stage medians (`cse3-step7-rshd3-2g.log`)

| Stage | Median | Range |
|---|---:|---:|
| encode | **0.000 ms** | 0–0 |
| sequential_write (copy+hash) | 103.7 ms | 63.5–154.0 |
| file_sync (`sync_all`) | 11.5 ms | 7.3–204.9 |
| frontier_publish | 11.9 ms | 7.0–29.2 |
| dir_sync | 3.9 ms | 2.8–13.1 |
| source_read_decode (Compact only) | 36.3 ms | 30.0–76.1 |
| wall (incl. Compact) | 182.9 ms | 136.6–480.1 |

Pre-mirror RSHD0002 baseline on same host: **3.69 seg/s** @ 2 GiB (encode≈51 ms).
Mirror removes the encode stage; remaining bound is durable physical copy + sync +
frontier. Still short of 7 at full ETQ cell except quiet 256 MiB.

## Residual

- Step 7 ≥7 @ 2 GiB/64 MiB **not** cleared.
- Next principal-approved experiment: **write-time dual streaming** (not started).
- Do **not** weaken `sync_all` → `sync_data` for the benchmark.
- Do **not** flip product sealing (step 8 blocked).
