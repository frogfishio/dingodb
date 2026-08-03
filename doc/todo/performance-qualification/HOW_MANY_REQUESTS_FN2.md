# How many requests did we send? (FN-2 table)

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *“how many requests did we send in total?”*

## Answer

For the **FN-2 four-cell Mode A table** you have been looking at:

| Scope | Count | Meaning |
|-------|------:|---------|
| **Per cell** | **32 768** | One Mode A present per key (batch=1) |
| **All four cells** | **131 072** | SQLite + off + Static + Adaptive |

```text
256 MiB logical ÷ 8 KiB/key = 32 768 keys = 32 768 Mode A requests per cell
4 cells × 32 768 = 131 072 requests in that campaign
```

JSON field: `keys_written` (same number every cell).

## What counts as a “request” here

On Mode A: **one acked put/insert = one request** (one key presented, wait for
ack, next). That is why QD=1 never had a second key in the collector during a
wait — request #k+1 starts only after request #k completed.

Mode B (multicore follow-up only): still **32 768 keys**, but packaged as
32 768/128 = **256** `put_many` calls per Residiuum B cell — different packaging,
same key count.

## Not this

- Not HTTP requests
- Not “how many collection flushes” (Static/Adaptive still ~32 768 single-key
  flushes on FN-2, plus delay)
- Multicore campaign was a **separate** later run (more cells; see artifacts)

## Source

`artifacts/firm-numbers-fn2-mode-a-apfs/*.json` → `keys_written: 32768`
