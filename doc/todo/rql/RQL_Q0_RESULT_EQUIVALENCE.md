# RQL-Q0 — Equivalent-result definitions

Status: **Q0.A2 amendment · principal freeze re-accept pending**

Package: RQL-Q0 deliverable 3 (amended)  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §1.3, §3, §6 ·
principal review finding #2 (equivalence too loose)  
Board: Q0.3 (first freeze) · **Q0.A2** (this amendment)  
Feature: `019fdac4-1408-7321-8edc-a09851c9e656`  
Effective: 2026-08-07 (A2 tighten)

These definitions freeze **what “same answer” means** for Q3 differential tests
and Q4/Q5 cross-engine cells. Harness code may refine serialisation detail in Q4
but must not weaken these dimensions without principal amendment.

**Law (anti-escape):** Hard cases are not excluded after results are known.
`type_incomparable`, “mapping note”, and lane-local carve-outs may be used only
when the **corpus case freezes them before first measurement** (versioned case
record). Post-hoc exclusion of a diverging cell is a **qualification defect**,
not an equivalence success.

---

## 1. Global comparison dimensions

Unless a family table narrows the set, two results are equivalent only when all
of the following match under the family's canonicalisation rules:

| Dimension | Meaning |
|---|---|
| **Values** | Document field values after type-normalisation (§2) |
| **Keys** | Primary / document identity keys in the result set |
| **Multiplicity** | Bag vs set semantics; duplicate rows when allowed |
| **Order** | Sequence when the query declares order; otherwise order-insensitive multiset |
| **Continuation** | Page boundaries + ability to reconstruct the full ordered stream |
| **Coverage / validity** | Completeness grade; holes must not masquerade as empty complete results |
| **Refusal** | Unsupported queries refuse with stable codes — not empty success |

Row-count alone is **never** sufficient.

---

## 2. Type and document normalisation (all families)

### 2.0 Admission of non-compare (closed set)

A cell may be marked non-comparable **only** if one of the following holds
**in the versioned corpus case before first green claim**:

| Code | When allowed |
|---|---|
| `corpus_deliberate_exclusion` | Case is Tier C / deliberate-exclusion with stable refusal |
| `lane_local_only` | Case tagged to one lane; not scored cross-engine |
| `predeclared_native_diff` | Case lists an exact allowed document-native difference (expression shape only — **not** answer values) |
| `type_incomparable` | **Only** for types outside §2.1–§2.6 closed rules **and** listed on the case with a type id |

Anything else that diverges is a **fail**. Adding a mapping note after seeing
results requires a **versioned principal-reviewed corpus amendment**, not a
silent harness skip.

### 2.1 Strings — collation, case, normalisation, null ordering

**Primary-profile string order and equality (all engines in a cell):**

| Rule id | Law |
|---|---|
| `str.eq.binary` | Equality is **binary UTF-8 code-unit** equality after NFC is **not** applied unless the case freezes `normalize=nfc` |
| `str.eq.default` | Default Gate-1 cells use **no Unicode normalisation** (`normalize=none`); strings compare as raw UTF-8 byte sequences decoded as Unicode scalar values |
| `str.case.default` | Default is **case-sensitive**. Case-insensitive cells must freeze `case=insensitive` **and** the exact case-fold algorithm (`unicode_default` vs engine-native); cross-engine cells without that freeze are **fail** if engines disagree |
| `str.collation.default` | Default collation for `order by` string keys is **binary / code-point order** (UTF-8 byte order of NFC-off strings). Locale collations (ICU, language tags) are **out of primary profile** unless the case freezes `collation=<id>` for **every** engine in the cell |
| `str.nulls.order` | For ordered string keys: **nulls first** is **not** assumed. Primary profile freezes **`nulls last`** for ascending order and **`nulls first`** for descending order **only if** the query language surface declares nulls ordering; otherwise the case must freeze `nulls=first|last|refuse` before run |
| `str.missing.order` | **Missing** is not a string. Ordered projection of a missing field follows §2.3; it is not sorted as empty string `""` unless the case freezes `missing_as_empty_string=true` (Residiuum default: **missing ≠ empty string**) |
| `str.empty` | Empty string `""` is a present value; distinct from missing and from null |

Cross-engine: if Mongo/CBL default collation differs from binary, the corpus case
must either (a) restrict to ASCII binary-safe fixtures, or (b) freeze an explicit
collation id on all sides, or (c) mark `lane_local_only` / non-overlap. Silent
“looks sorted the same on ASCII” is not a freeze.

### 2.2 Integers — domain and overflow

| Rule id | Law |
|---|---|
| `int.domain` | Primary fixture integers used in equality/order/aggregate **must** lie in **signed 64-bit** range `[−2^63, 2^63−1]` unless the case freezes `bigint=true` |
| `int.canonical` | Within ±2^53, JSON number identity is the mathematical integer; engines must not diverge on `1` vs `1.0` for **integer-declared** fixture fields (canonical form: no fractional part) |
| `int.wide` | Integers outside ±2^53 but inside i64: compare as exact integers (string or binary integer encoding in digests); float demotion that changes value is a **fail** |
| `int.overflow.arith` | Arithmetic that overflows i64: product path must **refuse** with a stable code or produce a case-frozen wrap/saturate policy. Default primary profile: **refuse / error**, not silent wrap |
| `int.overflow.agg` | `sum` overflow follows `int.overflow.arith`. Cases that need big sums must freeze `sum_domain=i128|decimal|refuse` |
| `int.vs.float` | Mixing int and float in one ordered key without a frozen coercion rule is a **fail** if engines disagree; default: fixtures avoid mixed numeric kinds in one key |

`type_incomparable` is **not** valid for ordinary i64 fixture integers.

### 2.3 Missing versus null (all engines)

| Rule id | Law |
|---|---|
| `mn.three_way` | Values are a **three-way** distinction: **missing** (field absent) · **null** (present JSON null) · **other** |
| `mn.residiuum` | Residiuum product and oracle must **preserve** three-way semantics on Residiuum self-differential. Silent missing→null collapse is a **defect** |
| `mn.eq` | Equality predicates: default primary profile treats `field == null` as matching **null only**, not missing, unless the case freezes `eq_null_matches_missing=true` |
| `mn.exists` | Existence / “field present” predicates must be case-frozen when used cross-engine (`exists` vs `$exists` vs `IS NOT MISSING`) |
| `mn.mongo` | MongoDB: missing vs null follow Mongo query semantics; corpus cases that cross-compare to Residiuum must state the intended mapping **up front**. Default cross-engine overlap cases use fixtures where missing and null are **not confounded** (separate documents) so both engines can match without collapse |
| `mn.cbl` | Couchbase Lite / SQL++: same rule — freeze mapping; prefer non-confounded fixtures for overlap cells |
| `mn.project` | Projection of missing: omit key vs emit null is **not** free choice after the fact. Case freezes `project_missing=omit|null|refuse` |
| `mn.order` | See `str.nulls.order` / family §3.3 — missing is not ordered as null unless frozen |

### 2.4 Aggregates — type promotion, empty inputs, average precision

| Rule id | Law |
|---|---|
| `agg.empty.count` | `count` of empty group / no input rows: **0** (not null) unless case freezes otherwise |
| `agg.empty.sum` | `sum` of empty: **0** for numeric primary profile (not null) |
| `agg.empty.min_max` | `min`/`max` of empty: **null** (no value), not refuse, unless case freezes `empty_minmax=refuse` |
| `agg.empty.avg` | `avg` of empty: **null** |
| `agg.null_inputs` | Null inputs are **skipped** for sum/min/max/avg (SQL-ish); missing fields are **not present** and do not count as null rows unless projected to null first under frozen rules |
| `agg.count_nulls` | `count(*)` counts rows; `count(field)` counts non-null present values — cases must name which |
| `agg.promotion` | Mixed int/float group: promote to float for avg/sum only if case freezes `promote=float`; default primary fixtures keep homogeneous numeric kinds |
| `agg.avg.precision` | Default: compare `avg` with **exact rational** when all inputs are integers and count divides evenly; otherwise freeze `avg_eps` (ULPs or absolute) **on the case before run**. No post-hoc epsilon widening |
| `agg.types` | Output type of sum/avg must match the frozen rule; unexpected float for all-integer sum is a **fail** unless `promote=float` |

Status of product aggregates may still be Tier A **blocker** for implementation;
these laws bind Q3 once aggregates exist and bind corpus expected results now.

### 2.5 Arrays — matching, nested arrays, duplicates, multikey

| Rule id | Law |
|---|---|
| `arr.eq` | Array equality is ordered element-wise under §2 scalar rules; length must match |
| `arr.bag_pred` | “Element matches predicate” (any/all) freezes quantifier on the case: `any` (default for Mongo-like) vs `all` vs `exact_element` |
| `arr.nested` | Nested arrays are values; predicates do not implicitly flatten unless case freezes `flatten=1|deep` |
| `arr.dupes` | Duplicate elements are significant for equality and for multikey index semantics; dropping dupes is a **fail** unless `unique_elements=true` is frozen |
| `arr.multikey` | Multikey index / unnest: result multiplicity follows unnest bag rules frozen on the case (`unnest_multiplicity=bag|set`). Default primary: **bag** (duplicates preserved) |
| `arr.empty` | Empty array `[]` is present; distinct from missing and from null |
| `arr.null_el` | Null elements are ordinary elements; not equal to missing slots |
| `arr.order` | Array order is significant for equality; sorting arrays for compare is forbidden unless case freezes `array_order=sort` |

### 2.6 Cursor pagination and inter-page writes

| Rule id | Law |
|---|---|
| `cur.concat` | Under **frozen snapshot / declared consistency** with **no intervening writes**: `page_1 ++ page_2 ++ … == unpaged` (same ordered stream) |
| `cur.tiebreak` | Total order includes immutable document key (or frozen tie-break fields); unstable order is a **fail** |
| `cur.token` | Continuation token bytes need not match across engines; reconstructability of the stream does |
| `cur.offset` | Offset pagination is **out of profile** — refuse, not equivalence |
| `cur.writes.default` | Primary profile Gate-1 comparative cells are **`writes_between_pages=forbidden`**: inter-page writes during a pagination walk are **out of cell**. Harness must not inject writes between pages in default cells |
| `cur.writes.declared` | Cells that test concurrent writes must freeze **all** of: `consistency_mode`, `write_schedule`, `visibility_expectation` (`snapshot_stable` \| `read_your_writes` \| `engine_native_residual`), and may be `lane_local_only`. Divergent visibility without that freeze is a **fail**, not `type_incomparable` |
| `cur.snapshot.residiuum` | Residiuum self-differential under a declared read view / pin: pages must be stable for the view’s lifetime; violation is a product defect |

---

## 3. Per-family definitions

### 3.1 Selection (key / equality / range / compound / nested / array / bool)

| Concern | Rule |
|---|---|
| Result set | Same multiset of document identities (or projected rows) under §2 |
| Predicates | Same truth table on the fixture, including absent/null/type cases (§2.3, §2.5) |
| Index vs scan | Residiuum forced-scan path must equal every admitted index path (Q3) |
| Comparator | Same logical documents selected; expression shape may differ only under `predeclared_native_diff` |
| Strings / numbers | §2.1–§2.2 |
| Non-compare | Internal index structures; planner choices; explain text wording |

### 3.2 Projection (flat / nested / computed / conditional)

| Concern | Rule |
|---|---|
| Shape | Projected field set and nesting match the intention |
| Identity projection | `project` identity ≡ unprojected document values for selected keys |
| Computed | Equal after evaluation on the same logical inputs (§2) |
| Missing | `project_missing` freeze (§2.3) |
| Order of keys in JSON objects | Insignificant unless corpus freezes field order |
| Non-compare | Storage layout of unprojected fields |

### 3.3 Ordering / top-k / cursor pagination

| Concern | Rule |
|---|---|
| Total order | Same sequence including immutable key tie-break (§2.1 collation, §2.2 numbers, §2.3 nulls/missing) |
| Top-k | First k of that total order |
| Pagination | §2.6 — default no inter-page writes |
| Continuation tokens | Opaque; equivalence is reconstructability of the stream, not token bytes |
| Offset | Not supported; refuse — not an equivalence class |
| Inter-page writes | Only under §2.6 declared cells |

### 3.4 Enrichment / cardinality (`exactly_one` / `optional` / `many` / within)

| Concern | Rule |
|---|---|
| Cardinality | Violations surface as errors or declared incomplete — never silent wrong bag |
| Attach bags | Same multiset of attached documents per parent (order: as declared) |
| Nested within | Depth and bag mapping match intention |
| Missing parents/children | optional vs exactly_one differ by SPEC; cross-engine requires predeclared mapping, not post-hoc |
| Non-compare | Whether foreign lookup used index vs scan **for competitive timing only**; correctness still requires equal bags |

### 3.5 Grouping and aggregation

| Concern | Rule |
|---|---|
| Groups | Same partition keys (after §2 normalisation) |
| Accumulators | count/sum/min/max/avg equal on each group under §2.4 |
| Empty groups | §2.4 empty laws |
| Float avg | §2.4 `agg.avg.precision` — epsilon only if predeclared |
| Order of groups | Insignificant unless `order by` on groups declared |
| Status today | Tier A **blocker** for product completeness — laws still bind expected results |

### 3.6 Budgets / cancellation / consistency / coverage

| Concern | Rule |
|---|---|
| Budget exhaust | Stable error (`resource_limit` / budget codes); not partial silent success |
| Coverage incomplete | `coverage_incomplete` (or equivalent); never empty complete page for a hole |
| Consistency | Mode-specific visibility; cross-engine only where modes map and are frozen |
| Cancel/timeout | Cooperative stop; partial results only if explicitly allowed by mode |
| Damage | Surviving readable data still queryable; corrupt candidates never “verified” |

### 3.7 Explain / SQL subset

| Concern | Rule |
|---|---|
| Explain | Must describe the programme/physical strategy **actually executed** (hash/identity), not a marketing plan |
| SQL subset | Emitted RQL/QVM must match direct RQL intention; refuse codes stable outside subset |
| Frontend identity | Equivalent SQL / builder / RQL → identical canonical QVM bytes/hash (Q2/Q3) |
| Non-compare | Pretty-print whitespace of explain trees across engines |

---

## 4. Cross-engine (Mongo / CBL) scope

1. Only corpus cases marked `comparator_overlap=true` enter cross-engine equality.
2. Lane separation (embedded vs local c/s) is mandatory — see [RQL_Q0_LANES_EXCLUSIONS.md](./RQL_Q0_LANES_EXCLUSIONS.md).
3. Allowed document-native differences must be listed on the case **before** first
   measurement (`predeclared_native_diff`). Expression shape only — not answers.
4. Coverage/damage honesty is Residiuum-specific; comparators may omit those
   dimensions but must not be scored as “winning” by ignoring holes.
5. String collation / null-missing / numeric domain must follow §2; “close enough”
   is not a pass.

---

## 5. Digest / canonicalisation sketch (Q4 owns implementation)

```text
canonical_scalar = per §2 (strings binary; i64 exact; missing/null tags distinct)
canonical_row    = sorted object keys + canonical scalars + key identity
result_digest    = hash(algorithm, ordered_or_sorted_rows, multiplicity, coverage_flags)
stream_digest    = hash(concat pages) for ordered queries
refusal_digest   = hash(stable_code, family_id)
```

Exact hash algorithm and encoding land in the Q4 harness; this freeze requires
that digests cover the dimensions in §1 and honour §2 tags (missing ≠ null ≠ "").

---

## 6. Exit

### Q0.3 (first freeze)

- [x] Global dimensions named
- [x] Per-family tables present
- [x] Cross-engine overlap policy named

### Q0.A2 (this amendment)

- [x] String collation / case / normalisation / null ordering laws (`str.*`)
- [x] Integer domain and overflow laws (`int.*`)
- [x] Missing vs null three-way laws across engines (`mn.*`)
- [x] Aggregate empty / promotion / avg precision laws (`agg.*`)
- [x] Array matching / nested / dupes / multikey laws (`arr.*`)
- [x] Cursor inter-page write visibility laws (`cur.*`)
- [x] Closed admission set for non-compare; ban post-hoc exclusion
- [ ] Principal accept of amended freeze (package-level Q0 after remaining A* work)
