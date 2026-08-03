# During the waiting window: zero other requests?

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *“if we are sending at the staggering rate, you’re telling me in the
‘Waiting window’ there was ZERO other requests coming in?”*

## Yes

**Exactly.** On FN-2 Mode A Static/Adaptive, while key *N* sat in the collector
waiting (~250 µs collection delay), **zero** other peer-pump requests arrived.

Not “almost none.” **None** — by how the client is written.

## Why (harness, not mystery load)

peer-pump Mode A + AWO is **one thread, QD=1**:

```text
admit key N
  → (maybe) enqueue in collector
  → WAIT until that put is fully acked   ← includes the collection-delay wait
only then:
admit key N+1
```

So during the wait for N, the client has **not called** admit for N+1 yet.
Nothing else is sending. The collector’s queue is `[N]` alone. The delay expires.
Flush N alone. Then N+1 starts.

```text
time →

  admit(N) ==== wait (delay + flush + ack) ====| admit(N+1) ==== wait ====|
               ▲                               ▲
               only N in queue                 N+1 not sent yet
               other requests in window: 0
```

“Staggering rate” (~2.5k/s) is just **how often that serial loop completes** —
still one-at-a-time. High rate ≠ overlapping requests.

## Contrast (when the window is *not* empty)

If the client kept outstanding depth > 1 (admit N+1 before N acks), the same
delay window could collect `[N, N+1, …]` and microbatch. FN-2 Mode A **forbids**
that. T11 saturated used pile-up on purpose.

## One line

```text
Waiting window on FN-2 Static/Adaptive Mode A = empty of other requests.
That is QD=1, not bad luck.
```

## Related

- [WHAT_PARTNER_PUT_MEANS.md](WHAT_PARTNER_PUT_MEANS.md)
- [WHY_CANT_WE_MICROBATCH.md](WHY_CANT_WE_MICROBATCH.md)
- [AWO_MODE_A_QD1_DELAY_TAX.md](AWO_MODE_A_QD1_DELAY_TAX.md)
