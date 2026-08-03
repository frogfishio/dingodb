# Answer — watermark / pre-touch drawbacks: real vs religion?

**Date:** 2026-08-03  
**Card:** `f6afdd08`  
**Ask:** “are there any real drawbacks to watermark pre-touch or is it just a religion?”

## Short answer

**There are a few real costs. Most of the “don’t do it” talk was religion.**  
Not doing watermark by default is **not** justified by salvage magic or “small stores must stay tiny.” It is justified only if you consciously accept those real costs — or if you have not yet **proven** an honest thr win under the meter you care about.

## Real drawbacks (keep these)

| Drawback | Why real | How big |
|----------|----------|--------:|
| **Space amp** | Watermark reserves **`capacity_bytes` per active** (product default **64 MiB**; host may set 100 MiB … 10 GiB). After seal, the **next** active does it again | Real; severity = host disk / #actives / chosen capacity. Tunable, not sacred |
| **You still pay zero/first-touch somewhere** | Bulk zero at setup and/or 64 MiB chunks ahead of the write head. Physics does not vanish — it moves | Real. Offline zero → better put odometer; mid-run chunks → blips inside puts |
| **ENOSPC timing changes** | Fail (or struggle) earlier when reserving capacity, instead of only when append grows | Real ops difference; preference, not morality |
| **Multi-segment amp** | Long peers that seal+rotate can stack reserved actives / sealed tails if you are sloppy about truncate (seal truncate fix addresses the worst cheat) | Real engineering residual |
| **Product incompleteness** | Policy is **process-local** today (not sticky across reopen); Linux/Scratch **unmeasured** on product flag | Real “not ready to default-on” |
| **CSQ / crash matrix not done for holes** | Preallocated / partially zeroed files change crash shapes; we have not run a dedicated campaign | Real residual — **evidence gap**, not theology |
| **Honest E2E thr not yet a win** | After seal-fix, product watermark ≈ grow on a noisy bed ([FIRM_NUMBERS_PRODUCT_WM.md](FIRM_NUMBERS_PRODUCT_WM.md)). Put-path pre-touch **can** lift (~35–50k diag with offline zero); full E2E with seal/chimera in the meter is a **different claim** still open | Real: don’t flip default on a fantasy SLO |

## Religion / soft anti-reasons (drop these)

| Claim | Grade |
|-------|-------|
| “We can’t salvage / scan otherwise” | **Religion** — false |
| “Pay-at-write is a hard product requirement” | **Religion** — unnamed customer |
| “512 MiB always outweighs thr” | **Religion** on fat hosts; situational elsewhere |
| “Grow-on-append is virtuous because survival thesis” | **Poetry** — does not decide Mode A thr |
| “Watermark is unsafe because digests feel scary” | **Religion** without a failing test |

## So is avoiding watermark a religion?

**Partly yes** if the argument is only the soft list.  
**Partly no** if the argument is: *space amp + unpaid CSQ/platform work + no honest E2E thr proof yet*.

```text
Pre-touch offline (diag)     →  real put-path upside (~35–50k class)
Product watermark today      →  real costs; E2E thr win unproven after cheat withdrawal
Keeping GrowOnAppend default →  OK as caution, not as dogma
```

## Practical take

- **Not religion:** disclose space, pay zero somewhere, finish sticky config + crash cells + quiet-disk re-pair before default-on.  
- **Religion:** blocking watermark forever because empty stores “must” stay tiny or salvage “needs” grow-on-append.  
- **Next principal call:** default-on vs stay opt-in after one **honest** put-path vs E2E campaign — not another myth fight.

## Non-claims

Not that default-on is decided. Not that 50k is a product SLO. Not Scratch/Linux. Not CSQ accept for prealloc holes.
