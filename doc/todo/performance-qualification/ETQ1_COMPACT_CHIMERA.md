# ETQ-1 — Compact Chimera Persistence (frozen)

Status: **next implementer package** (principal freeze 2026-08-04).  
Depends on: **ETQ-0 package accept**
(`doc/archive/performance-qualification/2026-08-04-etq0-enrichment-stage-breakdown/`).

## ETQ-0 acceptance (locked)

Chimera currently writes ~**63 MiB** of derived data for every **64 MiB**
authoritative segment — nearly **2× write amplification** before replication,
backups, or history. Persist time (~366 ms/seg) is why enrichment service sits
at ~1.6–2.7 seg/s while the rest of the enrich path clears 7 seg/s.

> The database is fast; eager full-payload Chimera materialization is not.

**Do not** start ETQ-1 with more enrichment workers — they would compete for the
same disk while preserving the amplification.

## Product direction

Default Chimera (seal / enrichment) must persist primarily:

- segment identifiers (file / header identity);
- frame offsets and lengths into **authoritative** segments;
- classification / layout metadata;
- compact search structures (sorted key → locator table).

**Payload remains in authoritative segments.** Fully materialized alternative
layouts (today’s `PointContainer` + `ValueLog` full-body sidecars) are
**lazy**, hotness-driven, or explicitly requested — not rebuilt for every seal.

### In-tree hooks

- Wire today: `RCHIMR01` / `VERSION=1` embeds full bodies
  (`chimera/layout.rs` encode of containers + value log).
- Locator enum already has `ValueLocator::ScanExtent` (and friends) — extend or
  add a **segment-frame** locator and bump format version for default encode.
- Hot `Store::get` already prefers PrimaryIndex; Chimera is not required for
  correctness (rebuildable derived accelerator — Law 6).

## Acceptance gates (all required)

| Gate | Bound |
|---|---|
| Default Chimera derived bytes | **≤ 5%** of authoritative sealed bytes |
| Enrichment capacity | **≥ 7** segments/sec |
| Backlog slope | **≤ 0** during sustained ingestion (after warm-up) |
| Full lifecycle TPS | Approaches acknowledgement TPS |
| Reopen | Exact (`coverage_scan`) |
| Query | Verified with **locator-based** Chimera |
| Correctness independence | Chimera may be absent; salvage/get still correct |

## Explicit non-starts

- Parallel enrichment workers as the first lever.
- Optimising the cost of copying full payloads into `.cmr`.
- AWO / three-cell attribution (remain paused / deprioritized).

## Suggested labor slices

1. Spec + format bump: default encode = entry table + segment-frame locators;
   empty container/vlog regions; document rebuild from segments.
2. Seal path: `write_chimera_from_segment_puts` builds locator-only layouts
   (offsets from ItemEvent frames, not body clones).
3. `get_via_chimera`: resolve locator → segment pread (share PrimaryIndex
   resolve helpers).
4. Sustained 2 GiB enrichment-on campaign vs gates; archive evidence.
5. Optional later: compiler worker for eager physical recompile when product
   requests it.

## Evidence home

`doc/archive/performance-qualification/YYYY-MM-DD-etq1-compact-chimera/`.
