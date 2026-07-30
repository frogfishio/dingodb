# DingoDB verification status

Status: living evidence-gap snapshot

Updated: 2026-07-30

Normative strategy:
[TESTING_STRATEGY.md](../TESTING_STRATEGY.md)

Implementation plan:
[VERIFICATION_IMPLEMENTATION_PLAN.md](VERIFICATION_IMPLEMENTATION_PLAN.md)

This document records current evidence. It does not upgrade capability claims.

## 1. Current verdict

DingoDB has a substantial test foundation, but no profile currently has the
complete claim/invariant/oracle/evidence bundle required by the new strategy.

The repository MUST NOT describe whole-system testing as exhaustive.

## 2. Static inventory snapshot

Observed on 2026-07-30:

| Item | Count / state |
|---|---:|
| Rust `#[test]` annotations under `crates/` and `verification/` | 1,157 |
| integration-test files under `crates/*/tests/` | 82 |
| files containing tests/properties/proofs | 189 |
| cargo-fuzz targets | 5 |
| fuzz targets invoked by nightly | 4 |
| visible Kani proof harness annotations | 7 |
| visible Verus `proof fn` declarations | 7 |
| CI operating systems | Ubuntu, macOS |
| MSRV job | Ubuntu |
| Windows CI | absent |
| published line/branch coverage | absent |
| sanitizers/Miri | absent |
| controlled-schedule concurrency lane | absent |
| multi-process Jepsen-style lane | absent |

Counts are diagnostics. `VFY-2` will replace manual counts with
registry-derived status.

## 3. Current strengths

| Surface | Evidence present | Current limit |
|---|---|---|
| SDA | broad conformance and ENR tests | no unified claim registry |
| format | frame/segment/scan corpus and properties | fuzz surface incomplete |
| salvage | corruption, holes, forward/reverse behavior | no every-byte campaign report |
| store | API, indexes, history, chunks, tiering | no independent generated state model |
| crash | failpoints, abort, ENOSPC, permission, short-write | conditional full matrix; no real power-cut lab |
| Heap | isolation, Kani, Verus, TLA+ sketches | qualification false; external review open |
| server | protocol, TLS, authz, admission, bounds | no long concurrent soak evidence |
| cluster | deterministic simulation and lincheck | no multi-process partition histories |
| operations | backup, restore, scrub, migration | historical compatibility matrix absent |
| scale | testrig and manual campaigns | not a scheduled qualification lane |

## 4. Current CI lanes

### Pull request / push

Configured:

- format;
- strict clippy;
- architecture contract;
- build all targets;
- workspace tests;
- documentation;
- release-content checks;
- dependency/license checks;
- Ubuntu and macOS;
- Ubuntu MSRV;
- Kani Heap job; and
- Verus Heap job.

This configuration does not prove that the latest remote run is green without
importing its run artifact.

### Nightly

Configured:

- workspace tests;
- format destructive corpus;
- store suite;
- full crash-matrix environment;
- CLI suite;
- packaging; and
- four fuzz targets for 30 seconds each.

Not currently invoked:

- `heap_ownership` fuzz target;
- testrig smoke/large campaigns;
- Miri/sanitizers;
- mutation testing;
- compatibility matrix;
- multi-process network histories; and
- long soak.

## 5. Latest local full-suite attempt

Command:

```text
cargo test --workspace
```

Result:

```text
infrastructure_failure
reason: ENOSPC while compiling/linking test targets
available disk at observation: approximately 200 MiB
repository target directory: approximately 7.2 GiB
tests reached: compilation only; complete suite result unavailable
```

This is not a DingoDB functional failure. It is also not a pass.

Requirements created for `VFY-1`:

- preflight disk/inode budget;
- isolated artifact root;
- partial result manifest;
- infrastructure-failure classification; and
- no ambiguous “suite failed” summary.

No build artifacts were deleted automatically.

## 6. Gap matrix

Legend:

```text
green    meaningful evidence exists
amber    partial or implementation-shaped evidence
red      required lane absent
```

| Verification dimension | V0 pure | V1 format | V2 store | V3 Heap | V4 server | V5 cluster | V6 archive |
|---|---|---|---|---|---|---|---|
| normative corpus | green | green | amber | amber | amber | amber | red |
| independent oracle | green/amber | amber | red | amber | amber | amber | red |
| property generation | amber | amber | red | amber | red | amber | red |
| bounded proof | amber | amber | red | green/amber | red | amber | red |
| crash enumeration | n/a | amber | amber | amber | amber | amber | red |
| corruption campaign | n/a | green/amber | amber | amber | amber | amber | red |
| continuous fuzz | red | amber | red | red | red | red | red |
| concurrency exploration | n/a | n/a | red | red | red | red | red |
| packaged journey | n/a | amber | amber | red | amber | red | red |
| historical compatibility | red | red | red | red | red | red | red |
| scale/soak | n/a | amber | amber | amber | red | amber | red |
| multi-process faults | n/a | n/a | n/a | n/a | red | red | red |
| release evidence bundle | red | red | red | red | red | red | red |

`green/amber` means a strong kernel exists but its bounds or claim mapping are
not connected to the proposed registry.

## 7. Highest-risk unknowns

Ordered:

1. absence/coverage truth through query/index/remote combinations;
2. acknowledged writes across every persistence boundary;
3. Heap complete-path isolation beyond the pure kernel;
4. released-artifact install/upgrade/restore compatibility;
5. all untrusted parser/resource-denial surfaces;
6. sustained concurrent server behavior;
7. multi-process network consensus and repair;
8. encryption/key-provider failure across all artifacts;
9. maintenance under foreground load; and
10. archive/native remote-media behavior.

## 8. Production gates

`DEFECTS.md` §16 remains authoritative.

At this snapshot:

- most data-safety and single-node gates are unchecked;
- distributed gates are predominantly unchecked;
- TLS/authz/admission have meaningful evidence;
- continuation-token authentication remains open;
- fuzz coverage is incomplete; and
- operational/compatibility gates remain unchecked.

No test-count claim overrides those gates.

## 9. Immediate actions

| Order | Package | Action |
|---:|---|---|
| 1 | `VFY-0` | create claim/suite/profile/report registries |
| 2 | `VFY-2` | map tests/proofs/fuzzers to claims and oracles |
| 3 | `VFY-1` | implement preflight and evidence-producing runner |
| 4 | `M0-1` | use whole-database inventory to correct program status |
| 5 | `VFY-6` preparation | connect omitted Heap fuzz target and claims |

## 10. Status rule

Future updates MUST link evidence artifacts or source revisions. “Well tested,”
“full coverage,” and “exhaustive” are not states.
