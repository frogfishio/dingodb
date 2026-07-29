# Heap qualification operator runbook (HP-010)

Status: operational draft for single-node `dingo-heap-v1` qualification evidence  
Normative anchors: [HEAP_SPEC.md](../HEAP_SPEC.md) §26–§27 / §39–§40 (HP-010),
matrix `spec/heap/qualification/hp010-matrix-v1.json`.

This runbook tells operators how to execute the drills that the qualification
matrix records, how to interpret `qualified=false`, and what the isolation claim
does **not** cover. It does not authorize advertising a qualified profile.

## 1. Claim honesty

Until every mandatory gate/drill in the HP-010 matrix is `accept` **and** Gate H6
passes external review, product language remains Level 1:

> DingoDB provides named heap namespaces; strong access-isolation qualification
> is in progress.

Machine check:

```bash
./scripts/verify-heap.sh
# asserts may_advertise_qualified() == false and matrix.qualified == false
```

Do **not** flip `QUALIFIED_CLAIM` or matrix `qualified` in a release branch without
complete evidence.

## 2. Pre-flight

1. Build with the same toolchain CI uses.
2. Confirm `spec/heap/qualification/hp010-matrix-v1.json` is present and
   `qualified` is `false`.
3. Run `./scripts/verify-heap.sh quick` — must exit 0.
4. Optionally run `./scripts/verify-heap.sh full` before a release candidate.

## 3. Mandatory drills (operator checklist)

| Drill | How to re-run | Pass means |
|-------|---------------|------------|
| Differential NI | `cargo test -p dingo-store --test hp010_qualification differential_ni_labelled_units` | Target-heap observation unchanged under other-heap mutation |
| Key-loss | `… key_loss_drill` | Destroyed key material is inert; second destroy fails closed |
| Restore payload-only | `… restore_drills_payload_and_retain_id` | Payload restore grants no access |
| DR retain-ID | same test | Ceremony fences old deployment; old credentials invalid |
| HeapCap lifecycle | `… heapcap_terminates_on_lifecycle` | Suspend/retire terminates prior caps |
| Single-owner admit | `… single_owner_admit` | Known owner only; mutations never admit under wrong heap |
| Derived paths | `… derived_path_indexes_streams_scoped` | Indexes/streams catalogs stay heap-scoped |
| SubjectV2 data plane | `cargo test -p dingo-sdk --test hp007_heap_isolation` | Same app keys on two heaps stay isolated; foreign SubjectV2 rejected |
| Remote connect_heap | `cargo test -p dingo-sdk --features dangerous-key-export --test hp007_connect_heap` | Welcome + ping/live/ready; wrong name → heap_unavailable |
| CPR-001 legacy opt-in | `cargo test -p dingo-sdk --test cpr001_legacy_opt_in` + `cargo check -p dingo-sdk --no-default-features` | Flat labelled non-qualified; heap-only profile builds |
| Query escape | `… query_escape_faulty_planner_confined` | Faulty unconstrained planner cannot escape bound heap |
| Load/latency | `… load_latency_budget` | Admit/decide/refresh meet recorded budget |
| Fuzz budget | `… fuzz_budget_structured_mutations` | Structured adversarial corpus rejects escape |
| Retention / incomplete purge | `… retention_and_incomplete_purge_still_retired` | Unavailable domain stays retired |

## 4. Day-2 operations tied to qualification

### 4.1 Key loss

1. Stop writers for the heap.
2. Invoke data-key destruction (in-process `destroy_data_key` or operator CLI when shipped).
3. Confirm destruction receipt fingerprint is non-zero and material is unreadable.
4. Attempt restore of ciphertext without the new key — must fail closed.

### 4.2 Payload restore vs DR retain-ID

- **Payload-only** restore always creates a **new** heap id and must not mint
  ordinary access.
- **Retain-ID** disaster recovery requires an explicit ceremony that fences the
  old `DeploymentId` and advances authority epoch. Refuse concurrent live
  authority without ceremony.

### 4.3 Incomplete purge

If any media domain (tier / replica) is unavailable, abort incomplete purge and
leave the heap **retired** with a permanent identity tombstone. Do not treat
partial wipe as purged.

### 4.4 Derived catalogs / indexes

Derived indexes live under `indexes/{heap_hex}/`. Wiping rebuildable catalogs
must not mix objects across heaps. After wipe, rebuild from descriptor chains
only (`rebuild_and_persist_all_catalogs`).

## 5. Load / latency and fuzz budgets

Recorded budgets (single-node CI, debug-friendly):

| Surface | Budget |
|---------|--------|
| 2 000 `admit_frame_to_heap` calls | ≤ 2 000 ms wall |
| 2 000 `decide` + `refresh_capability_or_terminate` | ≤ 2 000 ms wall |
| Structured fuzz corpus (envelope + stream/collection bind + concat) | 100% reject or same-heap Known only |

Raise budgets only by matrix amendment — never by silently skipping the drill.

LibFuzzer target `fuzz/fuzz_targets/heap_ownership.rs` supplements CI; overnight
fuzz is recommended before claiming H6.

## 5.1 Lifecycle crash cells

`crates/dingo-store/crash_matrix.v1.json` operation `heap_lifecycle` records
failpoints:

- `heap_lifecycle.after_state_store`
- `heap_lifecycle.after_transition_receipt`
- `heap_lifecycle.after_purge_plan`
- `heap_lifecycle.after_coverage_destroy`

CI Accept: `hp010_qualification::lifecycle_crash_matrix_peer_heaps_unaffected`
(peer heap labelled units remain readable; caps on the crashed heap terminate).

## 5.2 Health detail and support bundles

Public probes (`health_live` / `health_ready`) may observe only `live` and
`ready` from the closed §13.2 registry.

Authenticated heap-bound health detail may add draining + bound-heap usage and
must strip physical store paths, global live counts, node topology, and any
foreign-heap usage.

Support bundles drop foreign-heap artifacts, refuse secret-bearing entries, and
deny undeclared deployment-wide kinds.

CI Accept: `hp010_qualification::support_bundle_health_detail_scoped`.

## 5.3 Named isolation profiles

Machine-readable registry: `spec/heap/isolation-profiles-v1.json`.

| Profile | HP-010 role |
|---------|-------------|
| `heap-data-isolated` | Reference / H6 minimum |
| `heap-metadata-hardened` | Closed registry + coarsened timing / no aggregate load; operational confinement via `confine_operational_observation_under` |
| `heap-resource-isolated` | Declared, not qualified in this package |
| `heap-physical-isolated` | Declared, not qualified in this package |

SHA-256 of the exact JSON bytes is recorded by `IsolationProfileRegistry` for
qualification evidence. Deployment extensions start empty (`version: 0`).

CI Accept:
- `hp010_qualification::isolation_profile_registry_closed`
- `hp010_qualification::metadata_hardened_operational_confinement`

## 5.4 Connected H6 models (partial — includes `authority_admission_ok` / grace mint)

| Artifact | Connected Rust stand-in |
|----------|-------------------------|
| `formal/heap/HeapIsolation.tla` | `dingo_heap::IsolationModel` |
| `formal/heap/HeapAuthority.tla` | `dingo_heap::AuthorityModel` |
| §39 GenOK / blacklist | `generation_accepted` / `certificate_blacklisted` |

These are CI-connected sketches, **not** full Verus proofs. Claim stays Level 1.

## 5.5 Complete-path review and external review pack

| Artifact | Role |
|----------|------|
| [HEAP_COMPLETE_PATH_REVIEW.md](HEAP_COMPLETE_PATH_REVIEW.md) | Complete-path inventory; CPR-001…CPR-006 open findings |
| [HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md](HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md) | Engagement pack for independent review (report still open) |
| `dingo_heap::connected_pure_proof_bundle` | Executable pure lemmas (not machine-checked Verus) |

These advance H6 **evidence packaging**. They do **not** by themselves authorize
`qualified=true`.

## 6. Gate H6 limitations (published)

Logical heap isolation does **not** protect against:

- privileged administrators with local recovery / authority ceremony access;
- physical access to durable media or process memory;
- shared-resource side channels (timing, cache, scheduler contention);
- compromise of the serving-process TCB.

External security review and connected Verus/TLA evidence remain required before
any Level-2 claim language.

## 7. When something fails

1. Leave `qualified=false`.
2. File the failing drill name + matrix path in the release notes.
3. Do not ship marketing language that implies cryptographic isolation.
4. Re-run `./scripts/verify-heap.sh` after the fix; update matrix evidence paths
   only when the Accept test is green.