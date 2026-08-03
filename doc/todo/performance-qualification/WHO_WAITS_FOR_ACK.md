# What doesn’t send N+1 until ack?

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *“so wait… what doesn’t send N+1 until ack?”*

## One sentence

**The peer-pump test client** (`residiuum-testrig`) — our Mode A harness loop —
not AWO as a law of nature, and not “the database refuses overlapping puts.”

## Who

| Layer | Holds N+1 until N acks? |
|-------|-------------------------|
| **`peer-pump` Mode A + AWO path** | **Yes** — by construction (QD=1) |
| AWO collector | No — it would happily queue more if the client admitted them |
| `Store` / product API | No absolute “only one put forever” rule on this path |

So the empty waiting window is a **client presentation choice** matching PEER
Mode A (fair vs SQLite one-insert-at-a-time), implemented in the testrig.

## The loop (FN-2 Static/Adaptive)

In `crates/residiuum-testrig/src/peer.rs`, Mode A + lease:

```text
for each key:
    admit_put(key N)
    completion.wait()     ← blocks here until N is fully acked
    // only then does the for-loop start key N+1
```

That `wait()` is what “doesn’t send N+1 until ack.” One thread; next admit is
literally the next line after wait returns.

## Off path (same Mode A idea)

Residiuum-off Mode A also presents one key per `put_many([1])` and only then
the next — same **serial client**, different flush path (no collector delay).

## What this is *not*

- Not SQLite deciding Residiuum’s QD
- Not “Adaptive refuses concurrency”
- Not a claim that production apps must be QD=1 (they can overlap; then
  microbatch can work — T11-style)

## Related

- [ZERO_IN_WAITING_WINDOW.md](ZERO_IN_WAITING_WINDOW.md)
- [WHY_CANT_WE_MICROBATCH.md](WHY_CANT_WE_MICROBATCH.md)
