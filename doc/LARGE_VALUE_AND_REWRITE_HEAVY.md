# Large-value policy and rewrite-heavy workloads (DEF-103)

Contract id: `dingo-large-value-v1`

## What the 64 KiB threshold is

The default **chunk threshold** (64 KiB) and **chunk payload size** (16 KiB) are
**storage-layout** switches. They are **not** a document-size promise and do not
weaken atomicity, verification, or durability rules.

The admitted **maximum logical payload** for new writes under the application
profile is **16 MiB** (aligned with scanner / SDK ceilings). The effective write
ceiling is the **minimum** of store policy, scanner `max_body_len`, client, and
negotiated transport limits.

## Policy shape

```text
LargeValuePolicy {
  profile_id
  max_logical_payload_bytes
  chunk_threshold_bytes
  chunk_payload_bytes
  max_manifest_bytes
  max_reassembly_bytes
  max_write_peak_memory
}
```

Inspect via `Store::large_value_policy()`. Write receipts report non-secret
layout facts: `layout` (`inline` | `chunked`), `logical_len`, `chunk_count`,
`profile_id`.

Admission runs **before** event IDs, appends, or derived effects. Over-limit
rejection has zero authoritative or derived effect. Tightening policy later does
**not** make existing above-policy values unreadable.

## Rewrite-heavy documents (transcripts, agents, timelines)

Do **not** keep one ever-growing JSON document under a single key that is
rewritten on every turn. Prefer independently meaningful records:

```text
transcript/{id}/meta
transcript/{id}/turn/{monotonic-id}
transcript/{id}/timeline/{bounded-block-id}
transcript/{id}/snapshot/{generation}   # derived / rebuildable only
```

Helpers: `dingo_store::rewrite_heavy::*`.

Losing one turn/block must not make surviving units unqueryable. Aggregate
snapshots are optional and replaceable.

## Forbidden application reactions

- Treating `PayloadTooLarge` as “empty store”
- Silently overwriting partial evidence with `{}` / `[]`
- Raising only an RPC limit and assuming store admission rose with it
- Using chunk threshold as a product “max document size” claim without the
  logical payload ceiling
