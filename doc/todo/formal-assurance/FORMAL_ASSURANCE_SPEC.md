# Residiuum Formal Assurance Spine specification

Status: **normative design v1.1-draft — implementation-ready after CSQ-12**

Program: `FAS`

Profiles:

```text
residiuum-formal-foundation-v1
residiuum-formal-consistency-v1
residiuum-formal-security-v1
residiuum-formal-atomics-v1
residiuum-formal-cluster-v1
```

Normative companions:

- [Registry and evidence contract](FORMAL_ASSURANCE_REGISTRY_CONTRACT.md)
- [Formal kernel model contract](FORMAL_KERNEL_MODEL_CONTRACT.md)
- [Core Storage Qualification](../core-storage/CORE_STORAGE_QUALIFICATION_SPEC.md)
- [Heap specification](../../wip/heap/HEAP_SPEC.md)
- [Atomics specification](../atomics/ATOMICS_SPEC.md)
- [Cluster specification](../cluster/CLUSTER_SPEC.md)
- [Testing strategy](../../reference/engineering/TESTING_STRATEGY.md)
- [Verification implementation plan](../verification/VERIFICATION_IMPLEMENTATION_PLAN.md)

Implementation order:
[FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md](FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md).

## 1. Decision

Residiuum SHALL maintain a public, reproducible Formal Assurance Spine.

For selected product claims it SHALL publish:

1. a precise mathematical statement;
2. a closed set of assumptions;
3. a machine-checkable proof or bounded model result;
4. the connection, if any, to production Rust;
5. adversarial implementation evidence;
6. the exact release/source identity;
7. deliberately false negative controls; and
8. every excluded or unproved obligation.

The product principle is:

> Do not trust the claim. Read the theorem, inspect its assumptions, run the
> prover, and torture the connected implementation.

## 2. Purpose

The spine provides formal assurance for the areas where ordinary testing is
least capable of establishing universal confidence:

- database consistency;
- security and Heap noninterference;
- Atomic safety and isolation; and
- clustered agreement, fencing and convergence.

It also provides the method by which future theorem-bearing features enter the
system. A feature does not receive a strong formal claim merely because its
documentation contains equations.

## 3. Non-goals and honest boundary

The spine does not:

- prove that all possible hardware behaves correctly;
- prove the absence of every implementation defect;
- prove undeclared product behavior;
- treat successful model checking as an unbounded theorem;
- treat test coverage as mathematical proof;
- treat a mathematical theorem as connected to Rust without a refinement
  artifact;
- prove a cryptographic primitive rather than state the primitive assumption;
- prove availability without explicit scheduling/network assumptions;
- prove performance except for separately stated complexity/resource bounds;
- replace CSQ, fuzzing, mutation, crash or real-filesystem campaigns; or
- permit the unqualified phrase “formally verified database.”

The strongest general statement allowed is:

> Residiuum has a formally specified semantic kernel. The named invariants
> listed in its release proof manifest are machine checked under disclosed
> assumptions and, where marked implementation-connected, are refined by the
> released Rust kernel.

## 4. Assurance vocabulary

Every theorem obligation has exactly one highest achieved status:

```text
proposed
specified
model_checked_bounded
machine_proved
implementation_connected
physically_qualified
revoked
```

| Status | Meaning |
|---|---|
| `proposed` | informal intended property; no formal claim |
| `specified` | well-typed formal statement with assumptions; not proved |
| `model_checked_bounded` | every state in a disclosed finite model was explored |
| `machine_proved` | a proof assistant accepted an unbounded or explicitly bounded theorem |
| `implementation_connected` | proof/refinement is connected to the released Rust path |
| `physically_qualified` | connected proof plus applicable CSQ/adversarial physical evidence |
| `revoked` | evidence invalidated by defect, drift, tool failure or changed assumptions |

Statuses are monotone only within one unchanged theorem, assumption, source and
toolchain identity. A changed identity creates a new obligation version.

Forbidden claims include:

- “proved” for `specified`;
- “mathematically proved” for test-only evidence;
- “formally verified implementation” for an abstract-only theorem;
- “unbounded proof” for bounded model checking;
- “available” when only safety was proved; and
- “secure” without the proved security property and threat assumptions.

## 5. Formal toolchain and authority

### 5.1 Verus

Verus is authoritative for Rust-connected pure safety properties and
refinement obligations over the production Rust kernel.

Verus proofs MUST import, call, refine, or be structurally tied to the code path
they claim. A duplicate demonstration implementation is not connected.

### 5.2 Lean 4

Lean 4 is the independent high-level theorem environment for abstract state,
operation, invariant-preservation and compositional proofs.

A Lean theorem is not implementation-connected until a registered refinement
bridge links its types, operations and observations to Verus/Rust.

### 5.3 TLA+, TLC and TLAPS

TLA+ is authoritative for temporal protocol specifications:

- crash/publication ordering;
- concurrency and isolation;
- authority epochs;
- Atomic decisions and recovery;
- leader terms, quorum decisions and membership change; and
- safety/liveness separation.

TLC or another approved checker provides bounded exploration. TLAPS or an
approved proof backend provides theorem proof. Reports MUST distinguish them.

### 5.4 Kani

Kani is authoritative for bounded concrete Rust obligations such as arithmetic
and parser bounds, finite state combinations, generation/chunk arrangements
and control-state reachability. Bounds and unwinding assumptions are part of
the result.

### 5.5 Independent executable models

Executable models and reference readers are testing oracles, not proof
assistants. They provide differential evidence and counterexample replay.

### 5.6 No silent substitution

Replacing a proof tool requires theorem-by-theorem capability mapping,
proof-status review, regenerated evidence, negative controls, clean
reproduction and a profile-version decision.

## 6. Trusted computing base

Every profile declares its trusted computing base (`TCB`), including:

- proof-assistant kernel and pinned prover/model-checker binaries;
- theorem sources, build scripts and registry evaluator;
- compiler/standard-library assumptions relevant to refinement;
- unsafe Rust and foreign-function boundaries in the connected path;
- cryptographic primitive assumptions;
- filesystem/persistence assumptions;
- scheduler/network fairness assumptions for liveness; and
- artifact hashing/signing implementation.

The TCB is minimized but never hidden. Each item has a version, hash and reason
it is trusted.

Any `unsafe` block reachable from a connected claim is eliminated, proved,
wrapped behind a proved contract plus assumption, or declared as a TCB boundary
that limits the claim.

## 7. Canonical mathematical universe

The exact primitive types, closed constructors, state record, `WellFormed`,
`Init`, operation vocabulary, transition relation and observation law are
normative in
[FORMAL_KERNEL_MODEL_CONTRACT.md](FORMAL_KERNEL_MODEL_CONTRACT.md). This
section is its compact mathematical overview, not an alternative model.

The abstract state is:

\[
\Sigma=(H,C,K,G,V,A,D,\Gamma,E,R,N)
\]

where:

- \(H\): Heap identities;
- \(C\): collection identities qualified by Heap;
- \(K\): document keys;
- \(G\): generation identities and order;
- \(V\): logical values;
- \(A\): authority and credential state;
- \(D\): durability/publication state;
- \(\Gamma\): coverage, damage and uncertainty state;
- \(E\): durable decision/evidence state;
- \(R\): active rule and relationship invariants; and
- \(N\): optional node/partition/term state.

An operation is a typed relation:

\[
Op:\Sigma\times Input\rightarrow\Sigma\times Outcome
\]

It is a function only where determinism is declared. Crash and incomplete
evidence may yield a set of permitted abstract outcomes.

### 7.1 Observations

\[
Observe:Principal\times Scope\times\Sigma\rightarrow Observation
\]

Observation distinguishes:

```text
complete(v, evidence)
absent_proved(evidence)
partial(evidence)
damaged(evidence)
unknown(evidence)
unauthorized
unavailable(evidence)
```

These constructors never collapse without a registered safe projection.
For readability, later formulae use `Complete(v)`, `Absent`, `Partial`,
`Damaged`, `Unknown`, `Unauthorized` and `Unavailable` as notation for pattern
matching the corresponding exact constructors above; `Absent` always means
`absent_proved(evidence)`, never failed discovery.

### 7.2 Abstract/concrete connection

For concrete Rust state \(s\):

\[
\alpha:ConcreteState\rightarrow\Sigma
\]

For deterministic concrete operation \(cOp\) and abstract operation \(aOp\):

\[
\alpha(cOp(s,x))=aOp(\alpha(s),x)
\]

For nondeterministic crash semantics:

\[
\alpha(cOp(s,x))\in aOp(\alpha(s),x)
\]

Observation refinement requires:

\[
Observe_c(p,s)=Observe_a(p,\alpha(s))
\]

or an explicitly registered conservative projection.

The implementation MUST NOT turn partial into absent, unknown into success,
damaged into valid, unauthorized into absent, old generation into current,
prepared into committed, or minority evidence into quorum commitment.

### 7.3 Compositional preservation

Every theorem-bearing feature \(F\) declares the earlier invariant set
\(\mathcal I\) it must preserve:

\[
\left(\bigwedge_{I\in\mathcal I}I(\Sigma)\right)
\land Pre_F(\Sigma,x)
\implies
\bigwedge_{I\in\mathcal I}I(F(\Sigma,x))
\]

This is how RRE, relationships, Atomics and clustering compose without
invalidating the storage/security foundation.

## 8. Theorem registry

The exact schemas, identifiers, mandatory theorem catalogue, status evaluator,
profile gates and fixture rules are normative in
[FORMAL_ASSURANCE_REGISTRY_CONTRACT.md](FORMAL_ASSURANCE_REGISTRY_CONTRACT.md).

Each theorem is versioned. The following is a conceptual summary; developers
MUST implement the closed schema from the registry contract rather than infer
a schema from this list:

```text
theorem_id
version
family
claim
formal_statement
scope
assumptions
excluded_cases
safety_or_liveness
proof_kind
tool
toolchain_hash
source_paths/source_hashes
rust_entrypoints
abstraction_function
refinement_obligations
bounds
negative_controls
qualification_suites
status/result_hashes
supersedes
```

The registry is closed. Every public formal claim maps to theorem IDs. No
theorem ID means no formal product claim.

## 9. Consistency theorem family

Profile: `residiuum-formal-consistency-v1`

### 9.1 No fabricated value

\[
Observe(p,k,\Sigma)=Complete(v)
\implies
\exists g.\ Committed(\Sigma,k,g,v)
\]

### 9.2 Generation exactness

\[
Reassemble(m_g,P)=Complete(v)
\implies
\forall p\in Used(P).\ EventId(p)\in EventIds(m_g)
\]

### 9.3 Publication atomicity

\[
Recover(Crash(Publish(op,\Sigma)))
\in\{\Sigma,Commit(op,\Sigma),Unknown(op,\Sigma)\}
\]

`Unknown` is allowed only where evidence is insufficient. A hybrid committed
state is never allowed.

### 9.4 Durable acknowledgement

\[
AckDurable(op,\Sigma)\land Assumptions_{fs}
\implies Observe(Recover(Crash(\Sigma)),op)=Committed
\]

Filesystem and device assumptions are theorem parameters, not footnotes.

### 9.5 Recovery idempotence

\[
Recover(Recover(\Sigma))=Recover(\Sigma)
\]

### 9.6 Derived-state non-authority

\[
DerivedOnly(k,\Sigma)\implies Observe(p,k,\Sigma)\neq Complete(v)
\]

unless authoritative evidence independently establishes \(v\).

### 9.7 Damage honesty

\[
InsufficientEvidence(k,\Sigma)
\implies Observe(p,k,\Sigma)\notin\{Absent,Complete(v)\}
\]

### 9.8 Healthy-island locality

\[
Valid(f,\Sigma)\land Damage(\Sigma)\cap Range(f)=\varnothing
\implies f\in Discover(Scan(Damage(\Sigma)))
\]

within the declared scanner and corruption assumptions.

## 10. Security theorem family

Profile: `residiuum-formal-security-v1`

### 10.1 Heap noninterference

\[
h_1\neq h_2
\implies Observe(p,h_1,Apply(op_{h_2},\Sigma))
=Observe(p,h_1,\Sigma)
\]

except declared global operational observations proved not to contain Heap
information.

### 10.2 Authority confinement

\[
Authorize(c,op)
\implies Heap(op)=Heap(c)\land Rights(op)\subseteq Rights(c)
\]

### 10.3 Delegation monotonicity

\[
Issue(parent,child)\implies Rights(child)\subseteq Rights(parent)
\]

Every parent restriction remains true or becomes stricter.

### 10.4 Epoch revocation

\[
Epoch(c)<CurrentEpoch(Heap(c))\land\neg GraceAccept(c)
\implies\neg Authorize(c,op)
\]

### 10.5 Blacklist soundness

\[
Hash(c)\in Blacklist\implies\neg Authorize(c,op)
\]

within the authenticated blacklist-snapshot model.

### 10.6 Master-key non-serving

\[
CredentialKind(c)=Master\implies\neg NetworkAuthorize(c,op)
\]

### 10.7 Scope-guard noninterference

Ordinary scoped operations observe only their scope; wildcard observers have
the defined RUD/no-create authority. No scope projection grants absent rights.

Cryptographic unforgeability is an assumption over named algorithms and key
sizes. Binding, audience, epoch, rights and transition logic are proved.

## 11. Atomics and isolation theorem family

Profile: `residiuum-formal-atomics-v1`

The family uses Residiuum’s declared semantics, not an assumed SQL transaction
model.

### 11.1 All-or-none visibility

\[
VisibleMembers(a,\Sigma)\in\{\varnothing,Members(a)\}
\]

### 11.2 Prepare completeness

\[
Decision(a)=Commit\implies\forall m\in Members(a).\ Prepared(m,a)
\]

### 11.3 Decision uniqueness

\[
\neg(Decision(a)=Commit\land Decision(a)=Abort)
\]

### 11.4 Prepared-state invisibility

\[
Prepared(m,a)\land\neg Committed(a)
\implies m\notin OrdinaryObservation(\Sigma)
\]

### 11.5 Retry idempotence

\[
Apply(i,Apply(i,\Sigma))=Apply(i,\Sigma)
\]

for one canonical idempotency identity and operation.

### 11.6 Invariant preservation

\[
Invariant_R(\Sigma)\land AtomicCommit(a,\Sigma)=\Sigma'
\implies Invariant_R(\Sigma')
\]

for active RRE and relationship invariants in the coordination scope.

### 11.7 Isolation

Every admitted concurrent history refines a named legal abstract history:

\[
\alpha(H_{concrete})\in Legal
\]

“Serializable” or “linearizable” is forbidden unless its exact history theorem
is proved.

### 11.8 Recovery convergence and liveness

Safety:

\[
\Box\neg HybridAtomicVisibility
\]

Conditional liveness:

\[
Prepared(a)\land FairRecovery\land StorageAvailable
\implies\Diamond Final(a)
\]

Safety remains valid when liveness assumptions fail.

## 12. Cluster theorem family

Profile: `residiuum-formal-cluster-v1`

The cluster MUST use a named, proved consensus protocol or produce equivalent
proof obligations. “Raft-like” is not a protocol.

### 12.1 Quorum intersection

\[
\forall q_1,q_2\in Q.\ q_1\cap q_2\neq\varnothing
\]

including every permitted joint-consensus membership transition.

### 12.2 Unique term authority

\[
Leader(n_1,t,p)\land Leader(n_2,t,p)\implies n_1=n_2
\]

### 12.3 Leader fencing

\[
term(op)<CurrentTerm(partition)\implies\neg Commit(op)
\]

### 12.4 Agreement

\[
\Box\neg\exists a,b.\ Committed(a,i)\land Committed(b,i)\land a\neq b
\]

### 12.5 Acknowledgement survival

\[
Ack_{quorum}(op)\land Failures\le f
\implies\Diamond RecoverableCommitted(op)
\]

under the declared persistence and membership assumptions.

### 12.6 Partition honesty

\[
\neg QuorumEvidence(op)
\implies Outcome(op)\notin\{Committed,LinearizableRead\}
\]

### 12.7 Replica convergence

\[
FairNetwork\land StableMembership\land QuorumAvailable
\implies\Diamond\forall r\in LiveReplicas.\ PrefixAgree(r)
\]

### 12.8 Distributed Heap confinement

Routing, replication, repair, rebalancing and recovery preserve security
noninterference. A node/partition ID never substitutes for a Heap ID.

### 12.9 Membership safety

Membership change preserves quorum intersection and prevents removed or stale
members from authorizing new commitments.

Safety proofs are mandatory before implementation admission. Liveness depends
on explicit eventual-synchrony/fairness assumptions and never weakens safety.

## 13. Proof connection

A theorem is `implementation_connected` only when:

1. concrete Rust entrypoints are named;
2. concrete and abstract types are related;
3. initial concrete state refines initial abstract state;
4. each concrete transition simulates an allowed abstract transition;
5. observations and errors refine without forbidden collapse;
6. crash/recovery transitions are included;
7. unsafe/FFI boundaries are discharged or declared;
8. feature flags and profiles match the release build;
9. proof source hashes match the code revision; and
10. CI executes the proof with a pinned toolchain.

Golden vectors and differential tests strengthen a bridge but do not prove
semantic equivalence between Lean, TLA+ and Verus. Unproved cross-tool bridges
remain explicit.

## 14. Negative controls and proof mutation

Every family includes false companions which MUST fail, including:

- allow cross-Heap observation or rights escalation;
- accept a stale epoch;
- mix chunk generations;
- select a hybrid crash state;
- treat partial as absent;
- expose one prepared Atomic member;
- permit conflicting decisions or two leaders in one term;
- acknowledge without quorum;
- break joint-consensus quorum intersection; and
- remove a required liveness assumption.

CI fails if a false theorem is accepted, a counterexample is missing, proof
discovery skips an obligation, a connected Rust entrypoint disappears, an
assumption changes without versioning, or registry/artifacts disagree.

## 15. Versioning and invalidation

The following invalidate affected results:

- theorem, assumption or abstract operation change;
- connected Rust/reachable implementation change;
- feature/profile or unsafe/FFI boundary change;
- compiler/prover/model-checker change outside the pinned lock;
- cryptographic, filesystem or network model change; or
- a counterexample or production defect.

Dependency analysis revokes the smallest affected theorem closure.

Every defect violating a theorem immediately revokes its claim, retains the
counterexample, adds a negative control/mutation, repairs specification or
implementation, and reruns the dependent proof/qualification closure.

## 16. Proof evidence bundle

Profile: `residiuum-proof-bundle-v1`

```text
proof-bundle/
  manifest.json
  theorem-registry.json
  assumption-ledger.json
  tcb.json
  toolchain-lock.json
  claims.json
  results/
  counterexamples/
  negative-controls/
  qualification-links/
  hashes.json
  verify
```

The manifest binds the Residiuum/source/binary identity, theorem profiles,
source and result hashes, tools, assumptions, TCB, model bounds, statuses,
exclusions, negative controls, Rust connections and CSQ evidence.

The verifier operates without trusting a running Residiuum database.

## 17. Public examination surface

```text
residiuum claims
residiuum theorem <THEOREM-ID>
residiuum assumptions [--theorem <THEOREM-ID>]
residiuum verify-proofs [--profile <PROFILE>] [--bundle <PATH>]
residiuum verify --profile core-storage
residiuum torture --profile release
```

The CLI never converts bounded model checking into theorem proof or abstract
proof into implementation connection.

## 18. CI and reproducibility

Pull-request CI validates registries, changed theorem closures, fast proofs,
TLA+ bounded smoke, negative controls, Rust connections and bundle verification.

Dedicated proof CI runs complete Lean, Verus, Kani, TLC/TLAPS, proof mutation
and clean-room reproduction. Release CI reruns mandatory theorems from a clean,
pinned environment. Cached results require exact input hashes.

The public verification path SHOULD be one command and MUST require no
commercial service.

## 19. Staged profiles

### 19.1 Foundation

Requires closed registries, pinned tools, evidence bundle/verifier, negative
controls, refinement vocabulary and claim-language enforcement. It makes no
database theorem claim by itself.

### 19.2 Consistency

Requires §9 theorems, implementation connections and applicable CSQ evidence.

### 19.3 Security

Requires §10, consistency preservation, connected Heap admission/observation
paths and applicable adversarial tests.

### 19.4 Atomics

Requires §11 and preservation of consistency, security, RRE and relationship
invariants.

### 19.5 Cluster

Requires §12 and preservation of consistency, security and applicable Atomic
invariants under the declared distributed profile.

Profiles compose only through proved preservation theorems.

## 20. Acceptance standard

A formal profile is accepted only when:

- every mandatory theorem is registered;
- assumptions and TCB entries are closed and versioned;
- required machine proofs pass;
- bounded results disclose complete bounds;
- connected claims have valid refinement bridges;
- negative controls fail for the intended reason;
- applicable CSQ/adversarial qualification passes;
- the proof bundle independently verifies;
- a clean environment reproduces the result;
- no skipped, stale, orphaned or ambiguous proof exists; and
- public wording exactly matches achieved statuses.

The release proposition is:

> Mathematically specified. Machine checked. Connected to the released Rust
> kernel where stated. Physically tortured. Independently reproducible.
