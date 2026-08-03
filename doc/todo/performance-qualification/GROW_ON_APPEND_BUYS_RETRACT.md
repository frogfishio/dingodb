# Correction — those grow-on-append “buys” were soft

**Date:** 2026-08-03  
**Card:** `32a89702`  
**Ask:** Principal call-out on [GROW_ON_APPEND_BUYS.md](GROW_ON_APPEND_BUYS.md) — “so basically nothing… What the fuck are we talking about?”

## Short answer

**You’re right.** Those three bullets were **rationalizations**, not hard product requirements.  
What we are actually talking about is simpler and uglier:

```text
GrowOnAppend is the status-quo default.
It costs ~5× Mode A thr vs pre-touched pages on this APFS bed.
The “benefits” I listed do not justify that tax by themselves.
```

## Line-by-line

### “Small/empty stores stay small (no forced ~512 MiB/active)”

**Weak.** Half a gig vs a fat binary is not a civilization-scale constraint on a laptop/server with tens of GiB free. It *can* matter for **many** tiny tenants / CI matrices / phones — but we have **not** shown that is the product’s binding case. Treating it as a sacred win was sales copy.

### “You pay disk when you write, not at create”

**Not a requirement for anyone we named.** It’s a preference some hosts like (defer ENOSPC). Other hosts prefer fail-fast at create with a reserved arena. Neither is Residiuum law. Disk-full-on-append is not inherently more honest than disk-full-on-create.

### “Growing prefix stays a plain scannable log”

**Misstated.** We are **not** saying watermark/prealloc makes salvage impossible. Frames are still frames; scan reads verified prefixes. Prealloc/zero/watermark change **file length / first-touch timing**, not “can we scan.” After the seal truncate fix, sealed images are the durable prefix either way. Salvage was a red herring here.

## What is actually true

| Fact | Strength |
|------|----------|
| Default is `GrowOnAppend` (historic) | Strong |
| On this bed, grow-in-timer ≈ **~10k**; pre-touch offline ≈ **~35–50k** pump | Strong (measured) |
| Watermark is **opt-in**, not default-on | Strong (code) |
| Space amp / create-time reserve are **real costs** worth disclosing | Medium — real, but size of pain is host-dependent |
| Those costs **justify staying slow by default** | **Weak / retract** until principal says so |

## What we should talk about instead

A **principal decision**, not more poetry:

1. **Keep grow-on-append default** — accept ~10k Real class as the default product face; thr seekers flip a flag.  
2. **Flip default toward watermark / seal-sized pre-touch** — disclose ~512 MiB/active (or tune capacity); chase the put-path band with an honest meter.  
3. **Something in between** (e.g. watermark only after N MiB written; smaller capacity).

Until that call, “what does grow-on-append get us?” honest answer:

> **Inertia + avoiding an undiscussed space amp — not a proven product virtue that outweighs the thr hit.**

## Non-claims

Not that we should flip default tonight without a decision note. Not that 50k is guaranteed E2E with seal in the meter. Not that SQLite’s growth path is free.
