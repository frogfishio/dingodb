# Long-retention runbook (Stage 9)

Status: operational draft  
Normative anchors: [OVERVIEW.md](../OVERVIEW.md) §9, [USP.md](../USP.md) §5,
[CLUSTER_SPEC.md](../CLUSTER_SPEC.md) §18, [DELIVERY_PLAN.md](../DELIVERY_PLAN.md)
Stage 9.

This runbook is the multi-year story for operators: how to keep one logical
store across hot → warm → cold → archive media without rewrite-the-world
migrations, and how to behave when archive media is offline.

## 1. Principles

1. **Segment identity is stable.** Moving or copying a sealed segment never
   changes its 16-byte segment id, item ids, or event ids.
2. **Location is derived.** Placement catalogs under `catalogs/` accelerate
   discovery; they are not the only map. Media under `segments/` and
   `tiers/{warm,cold,archive}/` (or external roots) remain authoritative.
3. **Offline is a coverage hole.** An unmounted archive MUST NOT look like an
   empty successful query. Callers inspect tier coverage before treating
   absence as proven.
4. **Cold is not hot.** Archive-path latency is never claimed under hot-path
   (memory-resident index) SLOs. Benchmarks live in a separate class
   (`stage9_archive_bench`).
5. **Bytes outlive software.** Unsupported wire majors are preserved
   byte-for-byte (`format-unsupported`); interpretation may wait for a newer
   reader.

## 2. Layout

```text
store/
  store-info/          # store_id, meta, descriptor
  active/              # open append segment (hot only)
  segments/            # sealed hot segments
  tiers/
    roots.txt          # online/offline + optional external media roots
    warm/              # default warm media
    cold/              # default cold / object-store stand-in
    archive/           # default archive media
  catalogs/
    collections.cat    # collection names (Stage 6)
    tier-placement.cat # segment → tier + path + hash (Stage 9)
    segments.cat       # hierarchical segment summaries (Stage 9)
  indexes/             # derived only
  recovery/migrations/ # migration evidence (hashes, tool version)
```

## 3. Day-to-day operations

### 3.1 Seal before tiering

Only **sealed** segments move. Seal the active writer (auto-seal at threshold,
or explicit seal) so the segment is immutable.

### 3.2 Copy vs move

| Mode | Effect | Use when |
|------|--------|----------|
| **Copy** | Dual residency; primary placement updates to destination | Safety window before reclaiming hot space |
| **Move** | Source removed after verified destination write | Capacity reclaim after durability on colder media |

API (Rust): `Store::transfer_segment_to_tier(segment_id, tier, TierMoveMode::{Copy,Move})`.

Each transfer writes migration evidence under `recovery/migrations/` with
source/dest BLAKE3 hashes and `tool_version=dingo-store-9`.

### 3.3 Mark a tier offline

Simulate unmounted archive (or truly unmount external media), then:

```text
Store::set_tier_available(TierClass::Archive, false)
```

or edit `tiers/roots.txt`:

```text
archive offline /path/to/archive/media
```

and reopen the store.

**Expected behavior**

- `tier_coverage().is_incomplete() == true`
- `get_with_tier_coverage` returns `absence_proven == false` when the value is
  missing and archive is offline
- Salvage/index rebuild scans only **online** media

### 3.4 Bring archive back

1. Mount media / set path in `roots.txt`
2. `set_tier_available(Archive, true)` or reopen after editing roots
3. Rebuild index / segment catalog if needed
4. Coverage should complete when all registered segments are readable

### 3.5 Hierarchical cold search

Use `list_segment_summaries()` / `segment_catalog()` to prune by tier, size, and
frame counts **before** streaming full segment bytes. After `catalogs/` loss:

```text
Store::rebuild_segment_catalog()
```

Rediscovery walks available media roots; offline segments keep last-known
summary metadata when placement still records them.

## 4. Fifteen-year checklist (OVERVIEW §9.5)

| Policy | Operator action |
|--------|-----------------|
| Format-version support | Prefer readers that preserve unsupported majors; upgrade software before deleting old binaries |
| Integrity scrub | Periodically hash sealed segments; compare to placement `content_hash` |
| Replica / EC | Out of band for single-node Stage 9; cluster profiles add redundancy |
| Media refresh | Copy segment to new media (`Copy` then verify), then `Move` off dying media |
| Encryption keys | Not yet in Stage 9 wire; retain keys outside the store if payloads are encrypted at rest by the host |
| Migration evidence | Keep `recovery/migrations/`; do not treat as sole authority |
| Obsolete codecs | Classify with `classify_segment` / `format-unsupported`; do not rewrite |
| Catalog rebuild | Delete `catalogs/*` and open / `rebuild_segment_catalog` |

## 5. Performance disclosure

| Class | What it measures | SLO claim |
|-------|------------------|-----------|
| Hot path (Stage 6 bench) | Memory-backed index point read, append durability modes | May approach memory-store targets with disclosure |
| Archive path (Stage 9 bench) | Tier transfer, catalog rebuild, cold listing/get | **No** hot-path latency claim |

Publishing archive timings next to hot-path numbers without this distinction is
a product bug.

## 6. Failure modes (quick)

| Symptom | Check |
|---------|--------|
| “Data vanished” after archive offline | Coverage incomplete? Absence proven? Mount archive |
| Placement catalog missing | Segments still under `segments/` or `tiers/*`; rebuild |
| Hash mismatch on transfer | Source media flaky; abort; do not delete source |
| `format-unsupported` | Preserve file; upgrade reader; do not “fix” in place |

## 7. Media locators (object-store seam)

The third column of `tiers/roots.txt` is a **media root spec**:

| Spec | Meaning |
|------|---------|
| path / `file:///path` | Filesystem directory (Stage 9 baseline) |
| `object:local:/path` | Local object layout (in-tree stand-in; optional `#prefix`) |
| `s3://bucket/prefix` | Amazon S3 — live via `DINGO_S3_ROOT` mirror |
| `gs://bucket/prefix` | GCS — live via `DINGO_GS_ROOT` mirror |

Rust: `dingo_store::MediaLocator::parse`, `open_media` / `open_media_with`,
`CloudMirrorConfig`, `MirroredCloudMedia`, `FilesystemMedia`, `LocalObjectMedia`.

### Live cloud mirrors

| Env | Layout |
|-----|--------|
| `DINGO_S3_ROOT` | `{root}/{bucket}/{prefix}/…` object keys as files |
| `DINGO_GS_ROOT` | same for `gs://` |

Point these at an rclone/s3fs mount, MinIO disk tree, or offline copy. Without
a mirror, cloud roots stay **offline** for coverage honesty (`MediaUnsupported`
on put/get).

Lifecycle policy (declarative): `tiers/lifecycle.json` via
`dingo_store::LifecyclePolicy` — evaluation is pure; transfers remain explicit.
Erasure-coded archive shards are scaffolded (`ErasureManifest`) but codecs are
not shipped. Latency claims: see [BENCHMARK_DISCLOSURE.md](BENCHMARK_DISCLOSURE.md).

## 8. Non-goals (remaining polish)

- Native SigV4 HTTP SDK (mirror / fuse mount is the shipped connector path)
- Erasure encode/decode codecs (manifest + naming only)
- Background lifecycle scheduler (policy file + evaluate only)
- Claiming archive reads have memory latency
