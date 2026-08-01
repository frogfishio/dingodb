# FAS migration map (FA0-W0-T2)

Status: **Wave 0 classification only**  
Date: 2026-08-01  
Source inventory: [FAS_PRECURSOR_INVENTORY.md](./FAS_PRECURSOR_INVENTORY.md)  
Package: does **not** claim FAS-0 accept. Seeds **FAS-0-T1** ownership map.

## Disposition vocabulary

| Disposition | Meaning for FAS-0 |
|-------------|-------------------|
| `register_in_FAS0` | Must appear in FAS-0 registries (artifact / proof-source / evidence ownership). **May not be omitted.** |
| `historical` | Keep in tree; record as historical / superseded path only if a replacement is named. None used this wave without a replacement. |
| `unconnected` | Explicitly left outside theorem proof status; still listed so FAS-0 exit “every artifact classified” holds. Not silently dropped. |
| `tooling` | Bootstrap/CI/gate machinery — register under **toolchain / operations**, not as theorems. |

**Hard rule:** Heap Verus pure_kernel lemmas and TLA `HeapAuthority` / `HeapIsolation` are **`register_in_FAS0`**. Silent drop is a defect.

Suggested initial proof **status** when FAS-0 writes rows (not applied yet):

| Class | Suggested registry status at first write |
|-------|------------------------------------------|
| Verus pure_kernel proofs | `specified` or `machine_proved` only after FAS-1 re-run produces result hashes — Wave 0 leaves as **register stub**, not auto-promoted |
| TLA + TLC configs | `model_checked_bounded` only with disclosed bounds + result hash; until then **register as specified / bounded model source** |
| Executable pure_proofs / Kani | evidence + bounded_rust path; not Lean theorem |
| CSQ model / reference-reader | physical / oracle evidence ownership for CON family |
| Fuzz targets | adversarial physical evidence class |

---

## Migration table

| path | disposition | intended FAS package | registry role (FAS-0-T1) | notes |
|------|-------------|----------------------|-------------------------|-------|
| `formal/heap/HeapAuthority.tla` | **register_in_FAS0** | FAS-0 registry + **FAS-5** security temporal | proof_source (tlc/tlaps family security) | Map toward `FAS-SEC-*` (epoch/authority). Keep module; migrate under `formal/tla/security/` later optional. |
| `formal/heap/HeapIsolation.tla` | **register_in_FAS0** | FAS-0 + **FAS-5** | proof_source | Noninterference / isolation sketch → `FAS-SEC-HEAP-NONINTERFERENCE-001` lineage. |
| `formal/heap/MCHeapAuthority.cfg` | **register_in_FAS0** | FAS-0 + FAS-5 | model_check_config | Bounds must be disclosed when any TLC result is published. |
| `formal/heap/MCHeapIsolation.cfg` | **register_in_FAS0** | FAS-0 + FAS-5 | model_check_config | Same. |
| `verification/heap-verus/verus/pure_kernel.rs` | **register_in_FAS0** | FAS-0 + **FAS-1** tool pin + **FAS-3/5** | proof_source (verus) | **Must keep all named lemmas/specs.** Primary Heap pure-kernel precursor. |
| `verification/heap-verus/src/lib.rs` | **register_in_FAS0** | FAS-0 + FAS-3 | connection_scaffold | Documents Verus↔executable obligation names; not a proof alone. |
| `verification/heap-verus/Cargo.toml` | **register_in_FAS0** | FAS-0 | crate_manifest | Package identity for scaffold crate. |
| `crates/residiuum-heap/src/pure_proofs.rs` | **register_in_FAS0** | FAS-0 + FAS-3 + FAS-5 | executable_oracle / bounded_rust | Lemmas + Kani harnesses; production-adjacent executable stand-ins. |
| `crates/residiuum-heap/src/decide_obligations.rs` | **register_in_FAS0** | FAS-0 + FAS-5 | executable_obligations | H6 decide-path; link in ownership map. |
| `crates/residiuum-heap/build.rs` | tooling | FAS-1 (cfg) | tooling | `cfg(kani)` support only. |
| `scripts/check_verus_heap.sh` | tooling | **FAS-1** (absorb / wrap) | gate_script | Eventually superseded or wrapped by `check-formal-security` / toolchain checks; keep until then. |
| `scripts/check_kani_heap.sh` | tooling | **FAS-1** | gate_script | Same. |
| `scripts/setup_verus.sh` | tooling | **FAS-1** | toolchain_bootstrap | Pin `0.2026.07.27.31579f0` → `toolchain-lock-v1.json`. |
| `scripts/verify-heap.sh` | tooling | FAS-1 residual / Heap HAR | umbrella_gate | Product Heap gate; not FAS package accept. |
| `scripts/check_heap_architecture.sh` | tooling | HAR residual | architecture_check | Not theorem authority. |
| `.github/workflows/ci.yml` (`kani-heap`) | tooling | **FAS-1** CI lanes | ci_job | Register as CI surface for Kani; expand to formal lanes later. |
| `.github/workflows/ci.yml` (`verus-heap`) | tooling | **FAS-1** CI lanes | ci_job | Same for Verus. |
| `crates/residiuum-store/tests/hp010_qualification.rs` (formal refs) | **unconnected** | FAS-0 note only | existence_cross_check | Asserts file strings exist; **not** a proof. Do not promote to theorem status. Keep test. |
| `crates/residiuum-store-model/**` | **register_in_FAS0** | FAS-0 ownership + **FAS-4** evidence | csq_independent_model | Physical/oracle for CON theorems; not Lean. |
| `crates/residiuum-store-model/tests/csq4_state_machine.rs` | **register_in_FAS0** | FAS-0 + FAS-4 | csq_suite | Transition coverage evidence. |
| `tools/core-storage-reference-reader/**` | **register_in_FAS0** | FAS-0 + FAS-4 | csq_reference_reader | Oracle firewall reader. |
| `scripts/verify-csq-oracle-firewall.sh` | tooling | FAS-4 evidence runner | csq_gate | Keep; link from CON physical evidence. |
| `scripts/verify-csq-state-machine.sh` | tooling | FAS-4 evidence runner | csq_gate | Keep. |
| `fuzz/fuzz_targets/*` | **unconnected** | FAS-0 catalogue note + CSQ/MUT | fuzz_targets | Adversarial class; register as evidence class later if CON/CSQ map requires; not theorems. |
| `formal/registry/**` (missing) | n/a — create | **FAS-0** / Wave 0-T3 | registry_tree | Must be created; not a precursor to migrate. |
| `formal/lean/**` (missing) | n/a — create | **FAS-2** | lean_kernel | Greenfield. |
| FAS `check-formal-*.sh` (missing) | n/a — create | FAS-0…4 | accept_commands | Greenfield per implementation plan §3.1. |

---

## Must-keep set (non-negotiable)

These rows are **`register_in_FAS0`**. FAS-0-T1 fails if any is omitted without an explicit `historical` replacement ID:

1. `formal/heap/HeapAuthority.tla` + `MCHeapAuthority.cfg`
2. `formal/heap/HeapIsolation.tla` + `MCHeapIsolation.cfg`
3. `verification/heap-verus/verus/pure_kernel.rs` — every lemma/spec named in inventory
4. `crates/residiuum-heap/src/pure_proofs.rs` — every lemma and Kani harness named in inventory
5. CSQ independent model + reference-reader (CON physical path)

**Zero `historical` drops** in this wave — no precursor is deleted or demoted to historical without a successor artifact ID.

---

## Suggested theorem ownership seeds (for FAS-0-T1)

Not binding until registries land; orientation only:

| Precursor surface | Candidate theorem IDs (REGISTRY §12) | Family |
|-------------------|--------------------------------------|--------|
| pure_kernel + pure_proofs binding/gen/blacklist/admission | `FAS-SEC-AUTHORITY-CONFINEMENT-001`, `FAS-SEC-EPOCH-REVOCATION-001`, `FAS-SEC-BLACKLIST-SOUND-001`, `FAS-SEC-MASTER-NONSERVING-001` | security |
| isolation models (Verus/TLA) | `FAS-SEC-HEAP-NONINTERFERENCE-001` | security |
| store-model publication / false harnesses | `FAS-CON-PUBLICATION-NONHYBRID-001`, `FAS-CON-DAMAGE-HONESTY-001`, … | consistency |
| reference-reader | CON physical qualification evidence | consistency |

ATM / CLU catalogue IDs: **register as `proposed`/`specified` stubs only** in FAS-0 — no precursor proof sources in this inventory.

---

## Directory migration (later, not Wave 0)

| Current | Eventual shape (impl plan §2) | Action now |
|---------|------------------------------|------------|
| `formal/heap/*.tla` | `formal/tla/security/` (or keep + registry path) | **Keep paths**; registry points at current paths |
| `verification/heap-verus/` | `formal/verus/security/` or remain + register | **Keep**; no move in Wave 0 |
| (none) | `formal/registry/*` | Create in **W0-T3 / FAS-0** |
| (none) | `formal/lean/` | Create in **FAS-2** |

Moving files is **out of scope** for Wave 0; classification is enough.

---

## Exit FA0-W0-T2

- [x] Every inventory row classified
- [x] Verus + TLA must-keep enforced
- [x] Intended FAS package named per row
- [x] No package accept; no scoreboard FAS accept
- [x] FA0-W0-T3 registry scaffold (fail-closed `scripts/check-formal-registry.sh`)