# RQL-Q0 — Equivalent-result definitions

Status: **labor complete · principal freeze pending**

Package: RQL-Q0 deliverable 3
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §1.3, §3, §6
Board task: Q0.3
Effective: 2026-08-07

These definitions freeze **what “same answer” means** for Q3 differential tests
and Q4/Q5 cross-engine cells. Harness code may refine serialisation detail in Q4
but must not weaken these dimensions without principal amendment.

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

1. **JSON numbers:** compare as IEEE-754 decimal-safe canonical form for integers
   in ±2^53; outside that range, compare stringified integer if both engines
   preserve integer identity, else mark cell `type_incomparable`.
2. **Floats:** exact bit equality not required across engines; use ULPs policy
   declared per corpus case (default: exact for fixtures that use only integers).
3. **Missing vs null:** Residiuum total semantics apply on Residiuum; Mongo/CBL
   null/missing differences are `document-native` only when the corpus case
   declares a mapping note. Silent collapse of missing→null is a **defect** for
   Residiuum self-differential (oracle vs product).
4. **Key order in objects:** insignificant unless projection demands field order.
5. **Binary / special types:** only when the corpus case includes them; otherwise
   out of cell.

---

## 3. Per-family definitions

### 3.1 Selection (key / equality / range / compound / nested / array / bool)

| Concern | Rule |
|---|---|
| Result set | Same multiset of document identities (or projected rows) |
| Predicates | Same truth table on the fixture, including absent/null/type cases |
| Index vs scan | Residiuum forced-scan path must equal every admitted index path (Q3) |
| Comparator | Same logical documents selected; expression shape may differ (`document-native-equivalent`) |
| Non-compare | Internal index structures; planner choices; explain text wording |

### 3.2 Projection (flat / nested / computed / conditional)

| Concern | Rule |
|---|---|
| Shape | Projected field set and nesting match the intention |
| Identity projection | `project` identity ≡ unprojected document values for selected keys |
| Computed | Equal after evaluation on the same logical inputs |
| Order of keys in JSON objects | Insignificant unless corpus freezes field order |
| Non-compare | Storage layout of unprojected fields |

### 3.3 Ordering / top-k / cursor pagination

| Concern | Rule |
|---|---|
| Total order | Same sequence including immutable key tie-break |
| Top-k | First k of that total order |
| Pagination | `page_1 ++ page_2 ++ … == unpaged` under declared consistency |
| Continuation tokens | Opaque; equivalence is reconstructability of the stream, not token bytes |
| Offset | Not supported; refuse — not an equivalence class |
| Inter-page writes | Only under corpus-declared consistency mode; otherwise out of cell |

### 3.4 Enrichment / cardinality (`exactly_one` / `optional` / `many` / within)

| Concern | Rule |
|---|---|
| Cardinality | Violations surface as errors or declared incomplete — never silent wrong bag |
| Attach bags | Same multiset of attached documents per parent (order: as declared) |
| Nested within | Depth and bag mapping match intention |
| Missing parents/children | optional vs exactly_one differ by SPEC; cross-engine mapping notes required |
| Non-compare | Whether foreign lookup used index vs scan **for competitive timing only**; correctness still requires equal bags |

### 3.5 Grouping and aggregation

| Concern | Rule |
|---|---|
| Groups | Same partition keys (after normalisation) |
| Accumulators | count/sum/min/max/avg equal on each group; empty-group policy per corpus case |
| Float avg | Corpus declares exact vs epsilon |
| Order of groups | Insignificant unless `order by` on groups declared |
| Status today | Tier A **blocker** — definitions apply once implemented |

### 3.6 Budgets / cancellation / consistency / coverage

| Concern | Rule |
|---|---|
| Budget exhaust | Stable error (`resource_limit` / budget codes); not partial silent success |
| Coverage incomplete | `coverage_incomplete` (or equivalent); never empty complete page for a hole |
| Consistency | Mode-specific visibility; cross-engine only where modes map |
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
2. Lane separation (embedded vs local c/s) is mandatory — see lanes freeze doc.
3. Allowed document-native differences must be listed on the case (e.g. field
   naming, `$` vs named params).
4. Coverage/damage honesty is Residiuum-specific; comparators may omit those
   dimensions but must not be scored as “winning” by ignoring holes.

---

## 5. Digest / canonicalisation sketch (Q4 owns implementation)

```text
canonical_row  = sorted object keys + normalised scalars + key identity
result_digest  = hash(algorithm, ordered_or_sorted_rows, multiplicity, coverage_flags)
stream_digest  = hash(concat pages) for ordered queries
refusal_digest = hash(stable_code, family_id)
```

Exact hash algorithm and encoding land in the Q4 harness; this freeze only
requires that digests cover the dimensions in §1.

---

## 6. Exit (Q0.3)

- [x] Global dimensions named
- [x] Per-family rules for selection, projection, order/page, enrich, agg, budgets, explain/SQL
- [x] Explicit non-compare lists
- [x] Cross-engine overlap policy
- [ ] Principal accept
