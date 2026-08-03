# Do we write like SQLite pages?

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: concurrent+multicore didn’t move thr → wall is past “chew”; bets it’s
**how we write**. Gemini description of SQLite 4 KiB pages / WAL — **do we?**

## Short answer

**No.** Residiuum does **not** use SQLite-style fixed **pages**.

We use **append-only segment files** of **variable-length frames** (envelope +
body + integrity). That is a different physical write model — closer to a
hashed log/heap than to a B-tree page cache.

Your bet that the remaining wall is **down-line of cook, in how we write** is
consistent with evidence. The Gemini text describes **SQLite**, not us.

## Side-by-side

| | SQLite (Gemini sketch) | Residiuum (what we actually do) |
|--|------------------------|----------------------------------|
| Unit on disk | Fixed **page** (often 4 KiB) | Variable **frame** (payload + envelope + Blake/suffix) |
| Update model | Load page → mutate → write page (via pager/WAL) | **Cook new frame** → **append** to active **segment** tail |
| Partial row change | Still writes whole page | We don’t patch bytes in place; new event/frame |
| Batch at COMMIT | Dirty pages flushed together | `put_many` / AWO flush: cook N frames, **ordered install**, one or more **tail `write_all`s** |
| WAL | Separate `.wal` of page images, later checkpoint into main DB | No SQLite WAL. Active **segment** is already append-oriented; **seal** rotates when large |
| Match disk 4 KiB? | Page size chosen to align with common block size | Frame size follows **record** (e.g. 8 KiB peer payload ≫ 4 KiB); OS still does block I/O underneath |
| Per-write integrity | Page checksums / SQLite structures | **BLAKE3 (etc.) per frame** — cook CPU we already measured |

## What “how we write” means in Residiuum code

Typical Buffered put path (product, not diagnostic sink):

1. **Cook:** encode item envelope + hash/frame (CPU — Blake/encode).
2. **Install/append:** `append` / `append_preencoded_frame` into the active
   segment buffer.
3. **Tail transfer:** seek + `write_all` on the segment file
   (`DiagnosticIoSink::Real`) — OS page cache for Buffered; barriers for Durable.
4. **Publish indexes** after persist (AWO-1 persist-before-publish on lease
   paths).
5. **Seal** when active segment hits soft threshold (default tens of MiB).

So: we **do** buffer into OS/page-cache and we **can** batch multiple frames
into fewer syscalls on `put_many` — but we are **not** maintaining a 4 KiB
page image of a B-tree the way SQLite does.

## Why this matters for the ~14k vs ~30k wall

Concurrent + multicore cook didn’t lift us → not “need more chew cores.”

SQLite Mode A on APFS can sit near **page/WAL append efficiency** for small
autocommit rows (with its own pager). Residiuum Mode A still pays:

- full **frame cook** per 8 KiB logical put, and
- **append-log + index publish** shape,

even when disk is fast. That is a plausible “how we write” ceiling relative to
SQLite pages — **hypothesis for next measure**, not yet a closed proof that
page-shaped I/O alone explains the 2×.

## Honest caveats on the Gemini blurb

- SQLite page size is **configurable**; 4 KiB is common default, not sacred.
- Modern SSDs often use larger erase/program units; “match 4 KiB” is a useful
  rule of thumb, not a full SSD physics lecture.
- WAL vs rollback journal differs; peer-pump uses **WAL + synchronous=NORMAL**.

None of that makes Residiuum a page store.

## One line

```text
SQLite: fixed pages (+ WAL page append).
Residiuum: variable hashed frames appended to segments.
We do not write the way Gemini described SQLite — and that difference is
in-bounds for the post-cook wall.
```

## Related

- Cook vs disk ladder: `doc/wip/status/surveys/PARKED-write-path-wall-20260801.md`
- Concurrent multicore null result: [FIRM_NUMBERS_CONCURRENT_MULTICORE.md](FIRM_NUMBERS_CONCURRENT_MULTICORE.md)
- Format/frames: `doc/wip/format/`, `crates/residiuum-format`
