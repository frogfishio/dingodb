# FAS precursor inventory (FA0-W0-T1)

Status: **Wave 0 inventory only**  
Date: 2026-08-01  
Package: does **not** claim FAS-0 accept or any theorem `machine_proved` / `implementation_connected`.  
Authority: FA0 Feature board + `FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md` §2 migration.  
Next task: **FA0-W0-T2** migration map (classify each row).

**status column values (this document):**

| Value | Meaning |
|-------|---------|
| `unconnected` | Exists in-tree; **not** registered under FAS theorem/claim registries (no `formal/registry/` yet) |
| `n/a` | Harness / script / oracle — not a theorem source |

Do **not** upgrade status here. Scoreboard FAS rows stay `not_started` until package exit.

---

## Summary

| Bucket | Count (rows) | Notes |
|--------|-------------:|-------|
| TLA+ heap models | 4 files | Bounded TLC when available; optional in verify-heap |
| Verus pure kernel | 1 source + scaffold crate | Pin target `0.2026.07.27.31579f0` via `setup_verus.sh` |
| Heap executable lemmas + Kani | `pure_proofs.rs` | Always cargo-test; Kani when installed |
| Gate scripts + CI | 4 scripts + 2 CI jobs | H6 / CPR-004 |
| CSQ oracles (register later) | store-model + reference-reader | Consistency physical/evidence path, not Lean |
| FAS registry tree | **absent** | Wave 0-T3 / FAS-0 |

---

## Inventory table

| path | tool | artifact_kind | lemma / module / harness names | status |
|------|------|---------------|--------------------------------|--------|
| `formal/heap/HeapAuthority.tla` | tla / tlc | temporal_model | MODULE `HeapAuthority`; `Init`, `Next`, `Spec`; `THEOREM Spec => []Inv` | unconnected |
| `formal/heap/HeapIsolation.tla` | tla / tlc | temporal_model | MODULE `HeapIsolation`; `Init`, `Next`, `Spec`; `THEOREM Spec => []Inv` | unconnected |
| `formal/heap/MCHeapAuthority.cfg` | tlc | model_check_config | SPECIFICATION `Spec`; INVARIANT `Inv`; CONSTANTS Gens/Certs/MaxGen | unconnected |
| `formal/heap/MCHeapIsolation.cfg` | tlc | model_check_config | SPECIFICATION `Spec`; INVARIANT `Inv`; CONSTANTS Heaps/Units/MaxUnits | unconnected |
| `verification/heap-verus/verus/pure_kernel.rs` | verus | pure_spec_and_proof | **spec:** `authority_binding_holds`, `generation_accepted`, `certificate_blacklisted`, `authority_admission_ok`; **proof:** `lemma_binding_rejects_foreign_heap`, `lemma_binding_accepts_match`, `lemma_generation_grace_window`, `lemma_blacklist_hits`, `lemma_non_serving_refuses_admission`, `lemma_isolation_foreign_unit`, `lemma_connected_pure_proof_bundle` | unconnected |
| `verification/heap-verus/src/lib.rs` | rust (scaffold) | connection_scaffold | `H6_PROOF_OBLIGATIONS`, `VERUS_TARGET_PREDICATES`, `KANI_HARNESSES_CONNECTED`; documents Verus↔executable link | unconnected |
| `verification/heap-verus/Cargo.toml` | cargo | crate_manifest | package scaffold for heap-verus | unconnected |
| `crates/residiuum-heap/src/pure_proofs.rs` | rust + kani | executable_lemmas | **lemmas:** `lemma_binding_rejects_foreign_heap`, `lemma_generation_grace_window`, `lemma_blacklist_hits_certificate_hash`, `lemma_non_serving_refuses_admission`, `lemma_isolation_model_inv_walk`, `lemma_authority_model_inv_walk`, `connected_pure_proof_bundle`; **kani:** `kani_binding_rejects_foreign_heap`, `kani_generation_grace_window`, `kani_blacklist_hits_certificate_hash`, `kani_non_serving_refuses_admission`, `kani_isolation_model_inv_walk`, `kani_authority_model_inv_walk`, `kani_connected_pure_proof_bundle` | unconnected |
| `crates/residiuum-heap/src/decide_obligations.rs` | rust | executable_obligations | H6 decide-path checks; `verus_connected_obligations_hold` test | unconnected |
| `crates/residiuum-heap/build.rs` | rustc | cfg_support | enables `cfg(kani)` for harnesses | n/a |
| `scripts/check_verus_heap.sh` | bash + verus | gate_script | requires pure_kernel lemma names present; runs Verus when binary available | n/a |
| `scripts/check_kani_heap.sh` | bash + kani | gate_script | requires Kani harness names in pure_proofs; cargo test always; cargo kani when available | n/a |
| `scripts/setup_verus.sh` | bash | toolchain_bootstrap | downloads pinned Verus `0.2026.07.27.31579f0` into `tools/verus` | n/a |
| `scripts/verify-heap.sh` | bash | umbrella_gate | quick/full: may invoke Kani, Verus, optional TLC on formal/heap | n/a |
| `scripts/check_heap_architecture.sh` | bash | architecture_check | heap layout checks (not a theorem prover) | n/a |
| `.github/workflows/ci.yml` job `kani-heap` | gha + kani | ci_job | runs `./scripts/check_kani_heap.sh` | n/a |
| `.github/workflows/ci.yml` job `verus-heap` | gha + verus | ci_job | `setup_verus.sh` then `./scripts/check_verus_heap.sh` | n/a |
| `crates/residiuum-store/tests/hp010_qualification.rs` (refs) | rust test | qualification_cross_check | asserts presence of Verus scaffold strings / TLA path names (existence, not FAS registry) | unconnected |
| `crates/residiuum-store-model/` (`src/{transition,scan_model,generator,history_api,false_harness,lib}.rs`) | rust | csq_independent_model | CSQ-1/4 sequential logical model; publication/history/scan; false harnesses | unconnected |
| `crates/residiuum-store-model/tests/csq4_state_machine.rs` | rust test | csq_suite | state-machine suite for CSQ-4 | unconnected |
| `tools/core-storage-reference-reader/` | rust CLI | csq_reference_reader | independent RESIDFRM forward scan (oracle firewall; no residiuum-store import) | unconnected |
| `scripts/verify-csq-oracle-firewall.sh` | bash | csq_gate | CSQ independent-oracle firewall | n/a |
| `scripts/verify-csq-state-machine.sh` | bash | csq_gate | CSQ state-machine verification entry | n/a |
| `fuzz/fuzz_targets/*` | cargo-fuzz | fuzz_targets | format/scan/heap_ownership fuzz (adversarial physical evidence class; not theorems) | unconnected |
| `formal/registry/**` | — | **missing** | FAS-0 target tree not created yet | n/a |
| `formal/lean/**` | lean4 | **missing** | FAS-2 target tree not created yet | n/a |
| FAS check scripts (`check-formal-*.sh`) | — | **missing** | FAS-0… package accept commands not present | n/a |

---

## Detail notes (for W0-T2)

### Heap formal precursors (primary FAS-5 security fodder)

- Verus file models the same obligations as `residiuum_heap::pure_proofs` with integer stand-ins (no I/O/crypto).
- Scoreboard already notes: “Verus pure_kernel 8 verified” / Heap Kani harnesses — **precursor evidence**, not FAS package accept.
- TLA models are small finite-state sketches; TLC optional and non-fatal in `verify-heap.sh` (`|| true` paths historically).

### CSQ oracles (FAS-4 consistency physical/evidence links)

- Not mathematical proofs. They are the **independent model / reference reader** path CSQ and FAS-4 will cite under physical qualification.
- Register under FAS as evidence/ownership map targets, not as Lean theorems.

### Explicit non-inventory (do not confuse with FAS)

- Product docs with equations (`ATOMICS_SPEC`, cluster, etc.) without machine-checked sources under `formal/` or `verification/`.
- `tools/verus/` binary install dir (gitignored) — toolchain, not product evidence.
- Performance / PQH crates — different program.

---

## Honesty bar (this slice)

- Inventory **does not** change `NEXT_BUILD_STATUS.md` FAS rows.
- Inventory **does not** assert `machine_proved` or public formal claims.
- Existing CI green on Verus/Kani is **Heap gate / CPR-004** evidence, not FAS spine accept.

---

## Exit for FA0-W0-T1

- [x] Paths under `formal/heap`, `verification/heap-verus`, heap check scripts, related CI
- [x] CSQ-linked model/reader paths named for later registration
- [x] All theorem-like rows marked `unconnected` or harness rows `n/a`
- [ ] FA0-W0-T2 migration classification (next)
