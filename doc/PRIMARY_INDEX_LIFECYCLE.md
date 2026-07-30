# Primary index lifecycle (DEF-102)

## One-sentence rule

`indexes/primary.idx` is a **derived checkpoint/frontier cache**, never
authority. Its byte size is **not** stored-data size and **not** a health
signal.

## Authoritative vs disposable

| Artifact | Role |
|----------|------|
| `active/*.dingo` | **Authoritative** append log (live writer tail). |
| `segments/*.dingo` | **Authoritative** sealed segments. |
| `chunks/` | **Authoritative** large-value chunk frames (when used). |
| `indexes/primary.idx` | **Disposable** derived frontier + slim locator index. |
| `catalogs/*` | **Disposable** derived catalogs. |
| `indexes/seg/*.hdx` | **Disposable** per-segment hydra indexes. |
| `snapshots/` | **Disposable** derived snapshots. |

Deleting every disposable directory and reopening the store must reconstruct
the same logical live state from authority (DEF-023).

## Lifecycle (create → compact)

```text
create store
    → append to active/  (authority grows here)
    → checkpoint/frontier write of primary.idx  (derived only; may be tiny)
    → seal / async seal rotate
         active → pending seal → sealed segments/
    → open: accept matching frontier, apply active tail only
         or reject cache → rebuild from segments + active
    → compaction reclaim of superseded sealed sources (policy-gated)
```

A healthy store can show:

- large `active/`
- empty or sparse `segments/` / `chunks/`
- a very small `primary.idx` (classically tens–hundreds of bytes)

That shape means “events still live in the active log; the cache is only a
frontier,” **not** “the database is nearly empty.”

## Validation classes

`Store::primary_cache_diag` / `dingo doctor` report:

| Class | Meaning |
|-------|---------|
| `accepted` | Cache decodes and matches store + sealed frontier context. |
| `absent` | No file; open rebuilds from authority. |
| `stale` | Sealed fingerprint no longer matches current sealed set. |
| `corrupt` | Truncated, hash-bad, undecodable, or **ahead** of active length. |
| `foreign` | Embedded `store_id` does not match this store. |
| `unsupported` | Unknown magic/version (or legacy path not preferred). |

Every rejected class is fail-closed for **using** the cache; recovery always
falls back to authoritative segments. Diagnostics never change ordinary
`get`/`put` results.

## Doctor fields

```text
primary_cache {
    present, format_version, byte_len,
    validation, sealed_fingerprint,
    active_segment_id, active_covered_len, active_actual_len,
    replay_bytes, resident_entries, resident_body_bytes,
    authoritative: false
}

lifecycle {
    active_shards, pending_seals, sealed_segments,
    checkpoint_reason, derived_ops_since_checkpoint,
    primary_cache_authoritative: false
}
```

## Operator guidance

1. Do **not** interpret `primary.idx` size as data size.
2. Do **not** treat “file exists” as “index healthy.”
3. Safe recovery after cache damage: delete `indexes/` (and other derived
   dirs if needed) and reopen — authority is in `active/` + `segments/`.
4. Never delete `writer.lock` to force unlock (DEF-101); that is unrelated
   to primary-cache rebuild.

## Evidence

- APIs: `diagnose_primary_cache`, `Store::primary_cache_diag`,
  `Store::lifecycle_diag`
- CLI: `dingo doctor` / `dingo doctor --json-out`
- Tests: `crates/dingo-store/tests/stage_def_102_primary_cache_diag.rs`
- Matrix: `doc/CAPABILITY_MATRIX.md` § Derived-index lifecycle diagnostics
