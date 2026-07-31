# Formal Assurance registry and evidence contract

Status: **normative v1.0-draft — required by FAS-0**

This document removes implementation discretion from the registries, theorem
catalogue, proof results and profile evaluator required by the
[Formal Assurance Spine](FORMAL_ASSURANCE_SPEC.md).

## 1. Files and authority

FAS-0 SHALL create exactly these normative files:

```text
formal/registry/theorems-v1.json
formal/registry/assumptions-v1.json
formal/registry/tcb-v1.json
formal/registry/claims-v1.json
formal/registry/profiles-v1.json
formal/registry/operations-v1.json
formal/registry/negative-controls-v1.json
formal/registry/toolchain-lock-v1.json
formal/registry/schemas/theorems-v1.schema.json
formal/registry/schemas/assumptions-v1.schema.json
formal/registry/schemas/tcb-v1.schema.json
formal/registry/schemas/claims-v1.schema.json
formal/registry/schemas/profiles-v1.schema.json
formal/registry/schemas/operations-v1.schema.json
formal/registry/schemas/negative-controls-v1.schema.json
formal/registry/schemas/toolchain-lock-v1.schema.json
formal/registry/schemas/proof-result-v1.schema.json
formal/registry/schemas/package-report-v1.schema.json
formal/registry/schemas/proof-bundle-v1.schema.json
formal/registry/fixtures/accepted/
formal/registry/fixtures/rejected/
```

The JSON files are canonical for identifiers, dependency closure, statuses and
release evaluation. Lean, Verus and TLA+ sources are canonical for the formal
semantics they check. Prose cannot override either.

All schemas use JSON Schema draft 2020-12, UTF-8, sorted unique arrays where
order has no semantics, lowercase enum values, and `additionalProperties:
false` at every closed object.

## 2. Identifier grammar

Theorem IDs:

```text
FAS-(FND|CON|SEC|ATM|CLU)-[A-Z0-9-]+-[0-9]{3}
```

Other IDs:

```text
FAS-ASM-[A-Z0-9-]+-[0-9]{3}
FAS-TCB-[A-Z0-9-]+-[0-9]{3}
FAS-CLAIM-[A-Z0-9-]+-[0-9]{3}
FAS-TOOL-[A-Z0-9-]+-[0-9]{3}
FAS-OP-[A-Z0-9-]+-[0-9]{3}
FAS-NEG-[A-Z0-9-]+-[0-9]{3}
FAS-RESULT-[A-Z0-9-]+-[0-9]{3}
residiuum-formal-(foundation|consistency|security|atomics|cluster)-v[1-9][0-9]*
```

IDs are permanent. Renaming creates a new ID and records `supersedes`.
Deleting an ID referenced by a released bundle is forbidden.

## 3. Closed enums

```text
family:
  foundation | consistency | security | atomics | cluster

property_class:
  safety | liveness | refinement | preservation | complexity

proof_kind:
  theorem | bounded_model | bounded_rust | rust_refinement |
  executable_oracle | physical_qualification

tool:
  lean4 | verus | kani | tlaps | tlc | executable_model | csq

status:
  proposed | specified | model_checked_bounded | machine_proved |
  implementation_connected | physically_qualified | revoked

result:
  pass | fail | not_run | infrastructure_failure | revoked

assumption_class:
  mathematics | cryptography | compiler | language_runtime | filesystem |
  hardware | scheduler | network | time | operator | cross_tool_bridge

tcb_class:
  proof_kernel | prover | model_checker | compiler | generator | verifier |
  unsafe_boundary | ffi_boundary | crypto_primitive | persistence_boundary |
  artifact_identity
```

No implementation-specific alias is accepted.

## 4. Theorem registry object

`theorems-v1.json` has:

```json
{
  "schema": "residiuum-formal-theorems-v1",
  "profile_scope": "residiuum-formal-foundation-v1",
  "items": []
}
```

Every item requires:

```json
{
  "id": "FAS-CON-GENERATION-EXACT-001",
  "version": 1,
  "family": "consistency",
  "title": "Generation-exact reconstruction",
  "property_class": "safety",
  "statement_ref": "FORMAL_ASSURANCE_SPEC.md#92-generation-exactness",
  "formal_symbols": {
    "lean": ["Residiuum.Consistency.GenerationExact"],
    "verus": ["generation_exact_reassembly"],
    "tla": []
  },
  "scope": ["residiuum-core-storage-v1"],
  "assumptions": [],
  "excluded_cases": [],
  "depends_on": ["FAS-FND-OBSERVATION-SEPARATION-001"],
  "required_obligations": [
    {
      "proof_kind": "theorem",
      "tool": "lean4",
      "required_for_status": "machine_proved"
    },
    {
      "proof_kind": "rust_refinement",
      "tool": "verus",
      "required_for_status": "implementation_connected"
    },
    {
      "proof_kind": "bounded_rust",
      "tool": "kani",
      "required_for_status": "implementation_connected"
    },
    {
      "proof_kind": "physical_qualification",
      "tool": "csq",
      "required_for_status": "physically_qualified"
    }
  ],
  "rust_entrypoints": [],
  "abstraction_functions": [],
  "negative_controls": ["FAS-NEG-MIX-GENERATIONS-001"],
  "qualification_suites": ["CSQ-SUITE-CHK"],
  "status": "specified",
  "result_refs": [],
  "supersedes": null
}
```

Rules:

- `statement_ref` resolves to one unique normative statement.
- `formal_symbols` names real symbols; placeholder strings are rejected.
- every assumption ID exists;
- every dependency theorem exists and the graph is acyclic;
- every negative control exists in the claims/fixture registry;
- `implementation_connected` requires non-empty `rust_entrypoints` and
  `abstraction_functions` plus passing required obligations;
- `physically_qualified` requires applicable qualification suites with hashed
  passing evidence; and
- registry `status` is derived from results, never manually promoted.

## 5. Assumption object

Every assumption requires:

```json
{
  "id": "FAS-ASM-CROSS-TOOL-MAP-001",
  "version": 1,
  "class": "cross_tool_bridge",
  "statement": "The registered Lean and Verus representations encode the same abstract operation and observation semantics.",
  "rationale": "Lean and Verus do not prove each other's source languages.",
  "applies_to": ["FAS-CON-GENERATION-EXACT-001"],
  "evidence": [],
  "falsification": "A generated common vector or transition produces different normalized observations.",
  "owner": "formal-assurance",
  "status": "active",
  "supersedes": null
}
```

Assumption statuses are `active | discharged | falsified | retired`.
`falsified` immediately revokes dependent theorems. `discharged` requires a
theorem/result reference that removes the assumption.

The initial assumption catalogue MUST include:

| ID | Class | Required statement |
|---|---|---|
| `FAS-ASM-CLASSICAL-LOGIC-001` | mathematics | accepted logic foundations of selected proof kernels |
| `FAS-ASM-CROSS-TOOL-MAP-001` | cross_tool_bridge | Lean/TLA+/Verus representations correspond where claimed |
| `FAS-ASM-RUST-COMPILER-001` | compiler | verified source meaning is preserved by the named Rust toolchain outside proved compiler scope |
| `FAS-ASM-CRYPTO-PRIMITIVES-001` | cryptography | named primitives meet their standard security assumptions |
| `FAS-ASM-FILESYSTEM-DURABILITY-001` | filesystem | qualified sync/persistence contract holds |
| `FAS-ASM-ATOMIC-FAIR-RECOVERY-001` | scheduler | recovery is eventually scheduled for Atomic liveness only |
| `FAS-ASM-CLUSTER-EVENTUAL-SYNCHRONY-001` | network | network eventually satisfies the liveness timing model |
| `FAS-ASM-CLUSTER-FAILURE-BOUND-001` | hardware | failures remain within the declared quorum profile |

The exact cryptographic/filesystem algorithms and profiles are filled from the
accepted product profile, not guessed by FAS-0.

## 6. TCB object

Every TCB item requires:

```json
{
  "id": "FAS-TCB-LEAN-KERNEL-001",
  "version": 1,
  "class": "proof_kernel",
  "name": "Lean 4 kernel",
  "version_ref": "FAS-TOOL-LEAN4-001",
  "artifact_hash": "sha256:<hex>",
  "reason_trusted": "Checks Lean proof terms.",
  "scope": ["residiuum-formal-consistency-v1"],
  "reduction_plan": null
}
```

Empty hashes, “latest” versions and prose-only tool identities are rejected in
release bundles.

## 7. Toolchain lock

`toolchain-lock-v1.json` contains one item for each required tool:

```json
{
  "id": "FAS-TOOL-VERUS-001",
  "tool": "verus",
  "version": "0.2026.07.27.31579f0",
  "source": "https://github.com/verus-lang/verus/releases/...",
  "sha256_by_platform": {
    "macos-aarch64": "<64 lowercase hex>",
    "linux-x86_64": "<64 lowercase hex>"
  },
  "bootstrap": "scripts/setup-formal-tools.sh --tool verus",
  "verify": "tools/verus/verus --version",
  "approved_by": "<review identity>",
  "approved_at": "<RFC3339>"
}
```

The existing Verus pin `0.2026.07.27.31579f0` is retained initially. FAS-1
selects exact Lean, Kani, TLA+/TLC and TLAPS versions through one compatibility
run, records their archive hashes, and requires principal review of this lock.
No implementer may substitute “latest” or float a CI installer.

Changing any pin invalidates the affected result closure.

## 8. Proof result object

Every tool invocation emits one `residiuum-formal-proof-result-v1` object:

```json
{
  "schema": "residiuum-formal-proof-result-v1",
  "result_id": "FAS-RESULT-CON-GENERATION-LEAN-001",
  "theorem_id": "FAS-CON-GENERATION-EXACT-001",
  "obligation_index": 0,
  "tool_id": "FAS-TOOL-LEAN4-001",
  "proof_kind": "theorem",
  "result": "pass",
  "source_revision": "<40 lowercase hex>",
  "dirty_tree_hash": null,
  "input_hashes": {},
  "tool_artifact_hash": "sha256:<hex>",
  "command": ["lake", "build", "Residiuum.Consistency.GenerationExact"],
  "started_at": "<RFC3339>",
  "duration_ms": 0,
  "bounds": null,
  "assumptions_used": [],
  "stdout_hash": "sha256:<hex>",
  "stderr_hash": "sha256:<hex>",
  "counterexample_ref": null,
  "environment_ref": "environment.json"
}
```

Rules:

- release `pass` requires a clean source revision;
- `not_run` and `infrastructure_failure` never satisfy an obligation;
- bounded tools require non-null `bounds` with state/unwind/search limits;
- counterexample-producing failure retains the original counterexample;
- commands are arrays, never shell strings;
- output content is an attachment addressed by its hash; and
- timestamps do not establish proof identity.

### 8.1 Common field constraints

All registry roots require exactly `schema`, `version` and `items`. `version`
is integer `1`; `items` is an array unique by `id`. Unless a field below is
explicitly nullable, it is required and non-null.

Across every object:

- `version` is a positive integer;
- SHA-256 values match `^sha256:[0-9a-f]{64}$`;
- source revisions match `^[0-9a-f]{40}$`;
- timestamps are RFC 3339 UTC values;
- paths are normalized repository-relative POSIX paths, contain no `..`, and
  are not absolute;
- identifier arrays contain unique known IDs;
- strings are nonempty after trimming; and
- maps have lexically sorted keys in canonical bundle encoding.

`null` is permitted only for `supersedes`, `dirty_tree_hash`, `bounds`,
`counterexample_ref` and explicitly optional feature fields.

## 9. Claim, profile and control objects

### 9.1 Public claim

`claims-v1.json` contains claim objects:

```json
{
  "id": "FAS-CLAIM-CONSISTENCY-GENERATION-001",
  "version": 1,
  "public_text": "Generation-exact reconstruction is implementation-connected under the listed assumptions.",
  "theorem_ids": ["FAS-CON-GENERATION-EXACT-001"],
  "minimum_status": "implementation_connected",
  "profile_ids": ["residiuum-formal-consistency-v1"],
  "allowed_channels": ["cli", "documentation", "release_manifest"],
  "forbidden_short_forms": ["the whole database is formally verified"],
  "supersedes": null
}
```

`allowed_channels` is closed to `cli | documentation | website |
release_manifest`. Every theorem ID exists. A claim is publishable only when
all named theorems meet `minimum_status` and none is revoked. Generated public
text MUST include boundedness and material assumptions when applicable.

### 9.2 Profile

`profiles-v1.json` contains:

```json
{
  "id": "residiuum-formal-consistency-v1",
  "version": 1,
  "mandatory_theorem_ids": ["FAS-FND-OBSERVATION-SEPARATION-001"],
  "minimum_status_by_theorem": {
    "FAS-FND-OBSERVATION-SEPARATION-001": "machine_proved"
  },
  "required_claim_ids": [],
  "required_tcb_ids": [],
  "required_tool_ids": [],
  "required_qualification_suites": [],
  "release_identity_required": true,
  "supersedes": null
}
```

The real consistency profile includes every foundation and consistency ID in
§12. The key set of `minimum_status_by_theorem` equals
`mandatory_theorem_ids`. Unknown, duplicate, omitted or extra theorem IDs
fail. Profile inheritance is forbidden in v1; the full closed set is
materialized so a verifier does not need implicit policy.

### 9.3 Negative control

Negative controls are items in `formal/registry/negative-controls-v1.json`:

```json
{
  "id": "FAS-NEG-MIX-GENERATIONS-001",
  "version": 1,
  "theorem_ids": ["FAS-CON-GENERATION-EXACT-001"],
  "fixture_path": "formal/registry/fixtures/rejected/mix-generations.json",
  "expected_failure_code": "generation_mismatch",
  "expected_stage": "proof_or_model",
  "fixture_hash": "sha256:<hex>",
  "mutant_source_ref": null
}
```

`expected_stage` is `schema | registry | proof_or_model | refinement |
qualification | bundle_verification`. A negative control passes only when the
declared stage rejects it with the exact stable code. A crash, timeout or
different rejection is failure.

### 9.4 Operation contract

`operations-v1.json` realizes every operation required by the formal kernel:

```json
{
  "id": "FAS-OP-PUT-001",
  "version": 1,
  "input_type": "Residiuum.Input.put",
  "precondition_symbol": "Residiuum.Operation.Put.precondition",
  "transition_symbol": "Residiuum.Operation.Put.step",
  "outcome_symbol": "Residiuum.Operation.Put.outcome",
  "mode": "write",
  "semantics": "relational",
  "preserved_invariant_ids": ["WF_CollectionsQualified"],
  "authorization_symbol": "Residiuum.Authority.canPut",
  "crash_point_ids": [],
  "rust_entrypoints": [],
  "verus_symbols": [],
  "tla_actions": [],
  "supersedes": null
}
```

`mode` is `read | write`; `semantics` is `deterministic | relational`. The
initial registry has one or more entries covering every closed `Input`
constructor in the kernel contract. A state-changing entry requires at least
one preserved invariant. Every entry requires an authorization symbol. FAS-3
fills Rust/Verus bindings; until then their arrays are empty and no connection
can be claimed.

### 9.5 Package report

Each acceptance command emits one
`residiuum-formal-package-report-v1` object:

```json
{
  "schema": "residiuum-formal-package-report-v1",
  "package": "FAS-2",
  "source_tree_hash": "sha256:<hex>",
  "toolchain_lock_hash": "sha256:<hex>",
  "started_at": "<RFC3339>",
  "finished_at": "<RFC3339>",
  "result_refs": [],
  "negative_control_result_refs": [],
  "artifact_hashes": {},
  "result": "pass"
}
```

`package` is `FAS-0` through `FAS-9`. `result` uses the closed result enum.
The report aggregates proof-result objects; it does not replace them. A
declared `pass` is recomputed from all package-mandatory obligations.

## 10. Proof bundle object

`proof-bundle-v1.schema.json` requires a manifest with:

```json
{
  "schema": "residiuum-formal-proof-bundle-v1",
  "bundle_version": 1,
  "profile_ids": ["residiuum-formal-foundation-v1"],
  "source_revision": "<40 lowercase hex>",
  "source_tree_hash": "sha256:<hex>",
  "binary_hashes": {},
  "registry_hashes": {},
  "toolchain_lock_hash": "sha256:<hex>",
  "result_paths": [],
  "negative_control_result_paths": [],
  "qualification_evidence_paths": [],
  "attachment_hashes": {},
  "created_at": "<RFC3339>",
  "builder_identity": "residiuum-formal-bundle/v1",
  "declared_result": "pass"
}
```

Every listed path is content-addressed by a manifest hash. No unlisted file
affects evaluation. The independent verifier ignores `declared_result`,
recomputes it, rejects duplicate paths, symlinks, path traversal, hash
mismatch, missing/extra required results, dirty release source and profile
closure mismatch.

## 11. Status derivation

The evaluator derives status in this exact order:

```text
revoked
  if a dependency/assumption/result is revoked or falsified

physically_qualified
  if implementation_connected and every required physical_qualification passes

implementation_connected
  if machine_proved or model_checked_bounded as required, and every mandatory
  rust_refinement/bounded_rust obligation passes with valid entrypoint binding

machine_proved
  if every mandatory theorem obligation passes

model_checked_bounded
  if every mandatory bounded_model obligation passes and no theorem obligation
  is required for this level

specified
  if formal symbols type-check but proof obligations are incomplete

proposed
  otherwise
```

A theorem cannot skip a required level. Independent Lean and Verus results may
both be presented, but one does not imply the other.

## 12. Initial theorem catalogue

FAS-0 MUST register these IDs. FAS-0 may add supporting lemmas, but may not
rename or omit the mandatory claims.

### 12.1 Foundation

| ID | Required property |
|---|---|
| `FAS-FND-OBSERVATION-SEPARATION-001` | complete, absent-proved, partial, damaged, unknown, unauthorized and unavailable are distinct |
| `FAS-FND-FORBIDDEN-COLLAPSE-001` | no registered public projection performs a forbidden collapse |
| `FAS-FND-REFINEMENT-COMPOSITION-001` | valid refinements compose while preserving named invariants |

### 12.2 Consistency

| ID | Required property |
|---|---|
| `FAS-CON-NO-FABRICATED-VALUE-001` | complete observations have authoritative committed provenance |
| `FAS-CON-GENERATION-EXACT-001` | reconstruction uses only the manifest generation |
| `FAS-CON-PUBLICATION-NONHYBRID-001` | crash recovery selects old/new/unknown, never hybrid |
| `FAS-CON-DURABLE-ACK-001` | durable acknowledgement survives under declared persistence assumptions |
| `FAS-CON-RECOVERY-IDEMPOTENT-001` | recovery reaches a fixed point after one application |
| `FAS-CON-DERIVED-NONAUTHORITY-001` | derived state cannot establish authoritative truth |
| `FAS-CON-DAMAGE-HONESTY-001` | insufficient evidence cannot become absence or completeness |
| `FAS-CON-HEALTHY-ISLAND-001` | unaffected valid frames remain discoverable within scanner assumptions |

### 12.3 Security

| ID | Required property |
|---|---|
| `FAS-SEC-HEAP-NONINTERFERENCE-001` | one Heap operation cannot change another Heap observation |
| `FAS-SEC-AUTHORITY-CONFINEMENT-001` | authorized operation remains within Heap and rights |
| `FAS-SEC-DELEGATION-MONOTONE-001` | child authority cannot exceed parent authority |
| `FAS-SEC-EPOCH-REVOCATION-001` | stale epochs fail outside declared grace |
| `FAS-SEC-BLACKLIST-SOUND-001` | blacklisted certificate identities fail authorization |
| `FAS-SEC-MASTER-NONSERVING-001` | master credentials cannot authorize network operations |
| `FAS-SEC-SCOPE-GUARD-001` | scoped/wildcard RUD-no-create semantics preserve authority and noninterference |

### 12.4 Atomics/isolation

| ID | Required property |
|---|---|
| `FAS-ATM-ALL-OR-NONE-001` | member visibility is empty or complete |
| `FAS-ATM-PREPARE-COMPLETE-001` | commit requires all exact members prepared |
| `FAS-ATM-DECISION-UNIQUE-001` | commit and abort cannot both be authoritative |
| `FAS-ATM-PREPARED-INVISIBLE-001` | ordinary readers cannot see uncommitted prepared state |
| `FAS-ATM-RETRY-IDEMPOTENT-001` | canonical retry identity cannot duplicate effects |
| `FAS-ATM-INVARIANT-PRESERVATION-001` | Atomic commit preserves active RRE/relationship invariants |
| `FAS-ATM-ISOLATION-HISTORY-001` | concrete concurrent history refines the named isolation profile |
| `FAS-ATM-RECOVERY-CONVERGENCE-001` | recovery is safe and conditionally reaches one final decision |

### 12.5 Cluster

| ID | Required property |
|---|---|
| `FAS-CLU-QUORUM-INTERSECTION-001` | all permitted quorums intersect |
| `FAS-CLU-TERM-AUTHORITY-001` | one leader authority exists per partition term |
| `FAS-CLU-LEADER-FENCING-001` | stale terms cannot commit |
| `FAS-CLU-AGREEMENT-001` | one command occupies a committed log position |
| `FAS-CLU-ACK-SURVIVAL-001` | quorum acknowledgement remains recoverable within failure bounds |
| `FAS-CLU-PARTITION-HONESTY-001` | missing quorum evidence cannot become committed/current success |
| `FAS-CLU-REPLICA-CONVERGENCE-001` | live replicas converge under explicit liveness assumptions |
| `FAS-CLU-HEAP-CONFINEMENT-001` | distribution preserves Heap noninterference |
| `FAS-CLU-MEMBERSHIP-SAFETY-001` | membership transitions preserve quorum safety and fencing |

## 13. Required proof authority by family

| Family | Primary theorem authority | Temporal authority | Rust connection | Physical evidence |
|---|---|---|---|---|
| foundation | Lean 4 | — | Verus where projection code exists | contract tests |
| consistency | Lean 4 | TLA+/TLAPS for publication/recovery | Verus + Kani | CSQ |
| security | Lean 4 | TLA+/TLAPS for epochs/rotation | Verus + Kani | Heap qualification/security tests |
| Atomics | Lean 4 | TLA+/TLAPS mandatory | Verus + Kani + concurrency kernels | Atomic crash/history qualification |
| cluster | Lean 4 supporting algebra | TLA+/TLAPS mandatory | Verus/Kani on protocol state and wire admission | deterministic simulation + fault campaign |

TLC results are bounded model evidence. TLAPS results are theorem evidence.
Neither automatically establishes Rust connection.

## 14. Cross-tool semantic bridge

Lean, TLA+ and Verus do not verify one another. V1 therefore treats their
semantic correspondence as an explicit TCB/assumption boundary rather than
pretending an automatic end-to-end proof exists.

For each connected theorem, FAS-3 SHALL provide:

```text
bridge_id
theorem_id
canonical operation/outcome names
Lean symbols
TLA+ actions (when applicable)
Verus spec functions
Rust entrypoints
normalization function
common generated vectors
cross-tool differential result
semantic-map assumption ID
```

The Rust connection claim is grounded in the Verus proof over the production
path. Lean provides independent abstract proof. TLA+/TLAPS provides temporal
proof. Their joint presentation remains subject to
`FAS-ASM-CROSS-TOOL-MAP-001` until a future verified common-spec generator
discharges it.

## 15. Profiles and exact gates

| Profile | Mandatory theorem IDs | Minimum status |
|---|---|---|
| `residiuum-formal-foundation-v1` | all `FAS-FND-*` | `machine_proved`; connected where code exists |
| `residiuum-formal-consistency-v1` | foundation + all `FAS-CON-*` | `physically_qualified` |
| `residiuum-formal-security-v1` | foundation + consistency + all `FAS-SEC-*` | `physically_qualified` |
| `residiuum-formal-atomics-v1` | foundation + consistency + security + all `FAS-ATM-*` | `physically_qualified` |
| `residiuum-formal-cluster-v1` | foundation + consistency + security + applicable Atomics + all `FAS-CLU-*` | `physically_qualified` |

The foundation profile may accept an abstract theorem without a Rust
entrypoint only when its theorem registry declares that no implementation
surface exists. Every product-behavior theorem requires connection.

## 16. Evaluator and verifier separation

The bundle builder and verifier MUST be separate implementations:

- builder: Rust crate/binary `residiuum-formal-bundle`;
- independent verifier: Rust crate/binary `residiuum-formal-verify` with no
  dependency on builder evaluation code;
- shared dependency permitted only for passive data structs and canonical
  decoding, not status derivation or gate evaluation; and
- negative fixtures are evaluated by both, with deliberately disagreeing
  mutant evaluators proving the firewall.

The verifier re-derives file hashes, dependency closure, status, profile gates
and release identity. It never trusts a declared overall result.

## 17. Minimum fixture catalogue

Rejected fixtures MUST include:

```text
unknown theorem/assumption/tool/status
duplicate or malformed ID
cyclic theorem dependency
missing formal symbol
claimed proof with not_run result
bounded proof without bounds
connected claim without Rust entrypoint
connected claim with changed entrypoint hash
physical qualification without CSQ evidence
falsified assumption
passing false theorem/negative control
stale toolchain/source/binary hash
cross-tool disagreement hidden
revoked theorem included in passing profile
builder-pass/verifier-fail mutant
partial bundle and path traversal
```

Accepted fixtures cover each assurance level independently and one complete
foundation profile.

## 18. FAS-0 acceptance command

FAS-0 is accepted only when this command exists and passes:

```text
bash scripts/check-formal-registry.sh
```

It SHALL:

1. validate every registry against its schema;
2. validate IDs, references and acyclic dependency closure;
3. confirm the mandatory theorem catalogue;
4. derive statuses from fixture results;
5. run every rejected fixture and require rejection;
6. compare Rust closed enums with registry enums; and
7. emit `target/formal-assurance/fas0-registry-report.json`.

File existence, text search and item counts alone cannot satisfy FAS-0.
