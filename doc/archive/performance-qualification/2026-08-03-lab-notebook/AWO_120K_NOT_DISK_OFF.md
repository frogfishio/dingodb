# Clarify — ~120k was not “disk off”

Status: **labor evidence** (not package accept)  
Card: `21c32343-b3ea-41de-95b6-da91b1f1c544`  
Date: 2026-08-03  
Corrects shorthand that could be heard as “120k = disk disabled.”

## Short answer

**No.** The ~120–140k class was **not** measured with the disk turned off.

What differed from the ~10k PEER long peer (and from T11’s Durable ~2×):

| Knob | ~10k PEER Mode A | ~135k Campaign G.2 micro | T11 AWO 2× |
|------|------------------|---------------------------|------------|
| Durability | **Buffered** | **Buffered** (`file_sync` **n=0**) | **Durable** (fsync barriers) |
| Work length | Long peer (~256 MiB logical) | Short 20k × 8 KiB | Tiny smoke cell |
| Seals mid-run | Yes (e.g. 64 MiB threshold) | **No** (512 MiB seal → 0 mid seals) | N/A to that story |
| Media | Real Scratch | Real Scratch (still writing) | Real APFS |
| What “n=0 file_sync” means | — | No **fsync** per put; still **`write_all` to OS** | Opposite: sync-bound |

**Buffered ≠ disk off.** Buffered means: acknowledge after transfer into the OS
page cache (product contract). Bytes still hit the volume via ordinary writes.
**Durable** adds explicit durability barriers (`sync_all` / fsync class) — that is
why T11 lives at ~0.5–1.1k ops/s, not 10k or 120k.

## Where “disk off” *almost* appears (do not confuse)

Campaign H separates **Discard** (diagnostic short-circuit / page-cache micro)
from **Real** media:

| Band | Ballpark | Disk? |
|------|----------|-------|
| Scratch cook micro / Discard-class | ~**330k** | Closest to “not asking the disk” — **not** the 120k claim |
| `/tmp` 1 GiB phase-bench **Real** | ~**100–160k** | **Real APFS disk** (Real ≈ 58% of Discard on that ladder) |
| G.2 Scratch Buffered no mid-seal | ~**135k** | Real Scratch; cook-bound short run |
| Long peer / multi-seal | ~**10k** | Real disk **+ seal wall** |

So: ~120k sits in the **short Buffered / low-or-no mid-seal** band — sometimes on
real `/tmp`, sometimes Scratch micro — **not** Discard-only, **not** disk powered
off.

## Back to the 2×

T11’s ~2× is only **Durable barrier amortization at collection depth k≈2**.
It never claimed to reproduce the ~120k Buffered short band. Comparing them as
“same machine, missing 8×” mixes durability class and seal/length bed.

## Sources

- `TEST_RESULTS.md` Campaign G.2 + H three-band rule  
- `PARKED-write-path-wall-20260801.md` Discard vs Real  
- `AWO_10X_VS_2X_ACCOUNTING.md` (prior; this note sharpens “not disk off”)  
- `AWO_THREE_WAY_T11_FIRST_POSITIVE_SIGNAL.md`  
