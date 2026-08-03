# Own it — disk fill from uncleaned peer work dirs

**Date:** 2026-08-03  
**Card:** `cfd5cbd5`  
**Principal:** Disk was almost full because labor kept writing test stores and did not clean them; principal had to clean up.

## Accepted

**Guilty.** Peer-pump / prealloc campaigns under `/var/tmp/...` left large store trees (and watermark prealloc made them worse). That self-filled the volume. Then we called the low TPS band “noisy/full disk” as if it were ambient weather. A large part of that fullness was **our leftover junk**.

Principal cleaned. Labor does not get to treat that as external.

## Mandatory rule (from now)

After every peer-pump / thr measure:

1. Work dir is ephemeral (`--work` under `/tmp/residiuum-peer-<pid>-<cell>` or similar).  
2. **`rm -rf` the work dir before the next cell** (success or fail).  
3. Do not leave watermark/prealloc actives sitting around.  
4. Before quoting “full disk,” run `df` and confirm leftovers are gone.  
5. JSON artifacts belong in-repo under `doc/todo/performance-qualification/artifacts/` — **not** giant on-disk stores.

If a run crashes mid-cell: still delete the work dir in the same turn before finishing.

## TPS note

~6.5–8k on that bed remains a measured TPS number for that moment — but blaming “the disk” without owning labor fill was dishonest. Quiet-bed ~12–14k is still the fair default band when the volume is not stuffed with our own test files.

## Non-claims

Not that every historical full-disk cell was only our junk (APFS can be busy for other reasons). Not that principal cleanup is optional for us next time.
