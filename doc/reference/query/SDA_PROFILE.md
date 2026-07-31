# Residiuum SDA Examination Profile

Status: Draft v0.1  
Depends on: SDA core and standalone profile

## 1. Purpose

This profile defines how Residiuum presents stored material, recovery evidence,
holes, partial payloads, and query pages to SDA.

SDA remains pure. Storage access, tier staging, decryption, decoding, and
resource control occur in the Residiuum host before SDA evaluation. The host
supplies explicit values describing their outcomes.

The profile rule is:

> If Residiuum can recover it, SDA can examine it.

## 2. Normative conventions

Field names and string tags in this document are case-sensitive.

Identifiers are lowercase hexadecimal strings unless a future named profile
declares another canonical representation.

Byte offsets and lengths are non-negative `Num` values.

Unavailable values use `None`. An explicit stored SDA `Null` remains
`Some(Null)` and MUST NOT be converted to `None`.

Operational storage failures are represented as examination data. They are not
converted into SDA `Fail` values unless evaluation of the SDA program itself
fails.

## 3. Examination unit

Every discovered frame, logical item, or hole is presented as an
`ExaminationUnit` with this product shape:

```sda
Prod{
  unit_kind: "item",
  status: "verified-complete",
  store_id: Some("001122..."),
  segment_id: Some("aabbcc..."),
  item_id: Some("112233..."),
  event_id: Some("445566..."),
  event_kind: Some("put"),
  physical: Prod{
    source: "segments/0001.residiuum",
    offset: Some(4096),
    encoded_length: Some(512),
    wire_major: Some(1),
    wire_minor: Some(0)
  },
  integrity: Prod{
    framing: "verified",
    structural: "verified",
    content: "verified",
    authentication: "not-present"
  },
  envelope: Map{},
  payload: Prod{
    availability: "complete",
    representation: "bytes",
    media_type: Some("application/octet-stream"),
    logical_length: Some(128),
    value: Some(Bytes("...")),
    extents: Seq[]
  },
  holes: Seq[],
  provenance: Seq[],
  uncertainty: Set{}
}
```

The example values are illustrative. The field set is normative.

### 3.1 `unit_kind`

Allowed values are:

- `item`;
- `event`;
- `chunk`;
- `structural-frame`;
- `hole`;
- `conflict`;
- `query-control`.

Unknown future unit kinds are preserved as strings.

### 3.2 `status`

Core values are:

- `verified-complete`;
- `verified-partial`;
- `verified-envelope`;
- `candidate-unverified`;
- `corrupt`;
- `missing`;
- `encrypted-unavailable`;
- `format-unsupported`;
- `conflicting`;
- `uncertain-derived-state`.

A host MAY add a namespaced status. It MUST NOT use a core status with weaker
semantics.

### 3.3 Optional identities

`store_id`, `segment_id`, `item_id`, `event_id`, and `event_kind` are `Opt`
because a damaged or purely physical recovery result may not expose them.

Failure to recover an identity is absence. An identity whose encoded bytes were
explicitly null is invalid input and MUST NOT be treated as absence.

## 4. Physical location

The `physical` product has the fixed fields:

- `source : Str` — stable scan-report name for the medium object;
- `offset : Opt[Num]`;
- `encoded_length : Opt[Num]`;
- `wire_major : Opt[Num]`;
- `wire_minor : Opt[Num]`.

`source` is provenance, not logical identity. Moving a segment may change the
source string without changing item or segment identity.

Unknown offset or length is `None`. Zero is `Some(0)`.

## 5. Integrity evidence

The `integrity` product has:

- `framing`;
- `structural`;
- `content`;
- `authentication`.

Each value is one of:

- `verified`;
- `failed`;
- `not-checked`;
- `not-present`;
- `unsupported`;
- `unavailable`.

`authentication` describes signature or authenticated-encryption evidence. It
does not describe semantic truth.

A unit MUST NOT use `verified-complete` when a required framing, structural, or
content check is anything other than `verified`.

## 6. Envelope

`envelope` is an SDA `Map`.

Known wire-envelope integer keys are projected to their canonical profile names
as strings. Unknown integer keys use:

```text
wire:<unsigned-decimal-key>
```

Unknown text extension keys use:

```text
ext:<original-key>
```

Duplicate encoded envelope keys make structural integrity `failed`; they MUST
NOT be silently normalized.

Envelope bytes that survive but cannot be decoded are represented through the
payload mechanism as a structural-frame unit, with the original bytes
preserved when available.

## 7. Payload

The `payload` product has:

- `availability`;
- `representation`;
- `media_type`;
- `logical_length`;
- `value`;
- `extents`.

### 7.1 Availability

Allowed values are:

- `complete`;
- `partial`;
- `unavailable`;
- `not-applicable`;
- `unsupported`;
- `encrypted-unavailable`;
- `conflicting`.

### 7.2 Representation

Allowed values are:

- `sda`;
- `bytes`;
- `chunk-map`;
- `external-reference`;
- `unknown`.

When `representation = "sda"`, `value` contains the decoded SDA tree.

When `representation = "bytes"`, `value` contains SDA `Bytes`.

When bytes are intentionally not materialized because of an explicit resource
limit, `value = None`, availability is `unavailable`, and uncertainty contains
`resource-limited`. The host MUST NOT represent that condition as missing
stored content.

### 7.3 Extents

`extents` is a `Seq` of products:

```sda
Prod{
  logical_start: 0,
  logical_length: 65536,
  status: "verified",
  chunk_id: Some("aabbcc..."),
  value: Some(Bytes("..."))
}
```

Extent status is one of:

- `verified`;
- `missing`;
- `corrupt`;
- `conflicting`;
- `encrypted-unavailable`;
- `unsupported`;
- `not-loaded`.

Extents MUST be ordered by `logical_start`.

Extents MUST NOT overlap unless status is `conflicting`.

Missing extents have `value = None`. They MUST NOT be filled with zero bytes.

## 8. Holes

A hole is an examination unit whose `unit_kind` is `hole`. Its envelope MUST
contain:

```sda
Map{
  "scope" -> "physical-range",
  "reason" -> "checksum-failure",
  "certainty" -> "known",
  "affects" -> Set{"payload", "state-completeness"}
}
```

Core scopes are:

- `physical-range`;
- `frame`;
- `segment`;
- `chunk`;
- `event-history`;
- `dependency`;
- `catalog-coverage`;
- `tier-availability`.

Core reasons are:

- `unreadable`;
- `missing`;
- `truncated`;
- `checksum-failure`;
- `hash-failure`;
- `invalid-framing`;
- `unsupported-format`;
- `key-unavailable`;
- `conflicting-evidence`;
- `resource-limit`;
- `offline-tier`;
- `unknown`.

Certainty is:

- `known` — direct evidence establishes the hole;
- `bounded` — evidence establishes a containing range;
- `inferred` — discontinuity is inferred but its physical extent is unknown.

The physical product carries the known byte range. An unknown boundary is
`None`.

## 9. Provenance

`provenance` is a `Seq` of products:

```sda
Prod{
  action: "recovered",
  source_id: Some("..."),
  tool: "residiuum-examine",
  tool_version: "0.1.0",
  evidence: Map{}
}
```

Core actions include:

- `ingested`;
- `copied`;
- `recovered`;
- `reconstructed`;
- `migrated`;
- `compacted`;
- `decoded`;
- `indexed`.

Sequence order is the declared provenance order, not necessarily wall-clock
order.

## 10. Uncertainty

`uncertainty` is a `Set[Str]`.

Core tags are:

- `history-gap`;
- `missing-dependency`;
- `partial-payload`;
- `catalog-incomplete`;
- `index-stale`;
- `offline-data`;
- `resource-limited`;
- `unsupported-decoder`;
- `conflicting-source`;
- `clock-order-unsafe`.

An implementation MAY add namespaced tags.

If a known hole could change a derived state result, the result MUST contain
`history-gap` or `missing-dependency` and MUST NOT have status
`verified-complete`.

## 11. Query pages

Massive query results are supplied to SDA as bounded pages. A page has:

```sda
Prod{
  query_id: "001122...",
  page_number: 0,
  complete: false,
  units: Seq[],
  coverage: Prod{
    catalogs: "partial",
    indexes: "partial",
    requested_partitions: Set{"0001", "0002"},
    completed_partitions: Set{"0001"},
    unavailable_partitions: Set{"0002"},
    partition_frontiers: Map{"0001" -> "term=8,position=419"},
    tiers: Set{"hot", "warm"},
    excluded_tiers: Set{"archive"}
  },
  continuation: Some(Bytes("...")),
  uncertainty: Set{"offline-data"}
}
```

`units` are ordered deterministically according to the query's declared order.

`complete = true` means no continuation is required for the declared query
scope. It does not imply that physically missing or offline data never existed.
Coverage and uncertainty remain authoritative.

Clustered coverage contains:

- `requested_partitions : Set[Str]`;
- `completed_partitions : Set[Str]`;
- `unavailable_partitions : Set[Str]`;
- `partition_frontiers : Map[Str, Str]`.

Every requested partition MUST appear in exactly one of `completed_partitions`
or `unavailable_partitions` when the page declares the distributed query
finished. A frontier string is a deterministic profile encoding of the term,
position, and read mode observed for that partition.

Continuation tokens are opaque bytes authenticated by the host when needed.
SDA programs may carry them but do not interpret them.

## 12. Ordering

When a query does not request another deterministic order, examination units
are ordered by:

1. `segment_id`, with `None` after all present identifiers;
2. physical `source`;
3. physical `offset`, with `None` after known offsets;
4. `event_id`, with `None` last;
5. the canonical encoding of the complete unit as a final tie-breaker.

The host MUST NOT expose filesystem enumeration, hash-table iteration, thread
completion, or replica encounter order as SDA sequence order.

## 13. Evaluation failures

Residiuum conditions are examination data. SDA language errors remain `Fail`.

Examples:

- a corrupt frame is an `ExaminationUnit` with status `corrupt`;
- a missing chunk is an extent with status `missing`;
- an unavailable archive tier is coverage and uncertainty data;
- applying a total selector to a `Map` is
  `Fail(t_sda_wrong_shape, "wrong shape")`;
- exceeding an SDA evaluator limit is a host/profile diagnostic and MUST NOT
  become a complete empty query result.

## 14. Conformance

A conforming Residiuum SDA host MUST test:

- explicit `Null` versus absence;
- complete and partial opaque payloads;
- ordered non-overlapping extent maps;
- holes with one or both unknown boundaries;
- corrupt frames that retain physical evidence;
- unsupported but verified formats;
- ciphertext with unavailable keys;
- conflicting verified replicas;
- stale and incomplete indexes;
- offline tiers;
- resource-limited pages;
- deterministic ordering across different thread and filesystem orders;
- preservation of unknown envelope fields and status tags.

For identical authoritative evidence, query scope, host profile, and SDA
program, the observable SDA result MUST be deterministic.
