import Residiuum.State

/-!
# Residiuum.WellFormed

Named well-formedness predicates and `init_well_formed`.
Normative: `FORMAL_KERNEL_MODEL_CONTRACT.md` §5–§6.
-/

namespace Residiuum

set_option linter.unusedVariables false

/-- Every collection's Heap exists. -/
def WF_CollectionsQualified (s : State) : Prop :=
  s.collections.all (fun qc => ListSet.mem s.heaps qc.heap) = true

/-- Generation ownership is by ItemId key construction. -/
def WF_GenerationOwnership (_s : State) : Prop := True

/-- `current(i)=g` implies `publication(i,g)=committed`. -/
def WF_CurrentCommitted (s : State) : Prop :=
  s.current.all (fun p =>
    let item := p.1
    let g := p.2
    decide (s.pubOf ⟨item, g⟩ = Publication.committed)) = true

/-- At most one generation is current per item (ListMap.get is functional). -/
def WF_CurrentUnique (_s : State) : Prop := True

/-- A committed complete value requires generation events. -/
def WF_ValueEvidence (s : State) : Prop :=
  s.values.all (fun p =>
    let gref := p.1
    if s.pubOf gref = Publication.committed then
      match s.generation_events.get gref with
      | none => false
      | some evs => !ListSet.empty? evs
    else true) = true

/-- Damage honesty placeholder (strengthened in FAS-4). -/
def WF_DamageHonesty (_s : State) : Prop := True

/-- Every credential is bound to one Heap present in state. -/
def WF_CredentialHeapBinding (s : State) : Prop :=
  s.credentials.all (fun p => ListSet.mem s.heaps p.2.heap) = true

/-- Delegated credentials stay in the same Heap as parent. -/
def WF_DelegationConfinement (s : State) : Prop :=
  s.credentials.all (fun p =>
    let cred := p.2
    match cred.parent with
    | none => true
    | some pid =>
        match s.credentials.get pid with
        | none => false
        | some parent => decide (parent.heap = cred.heap)) = true

/-- Atomic members are in the Atomic's Heap. -/
def WF_AtomicMemberQualification (s : State) : Prop :=
  s.atomics.all (fun p =>
    let ast := p.2
    ast.exact_members.all (fun item => decide (item.heap = ast.heap))) = true

/-- An Atomic cannot have both commit and abort (impossible by equality). -/
def WF_AtomicDecisionUnique (s : State) : Prop :=
  s.atomics.all (fun p =>
    let d := p.2.decision
    !(decide (d = AtomicDecision.commit) && decide (d = AtomicDecision.abort))) = true

/-- At most one log entry per (node, index) — structural via ListMap key. -/
def WF_LogIndexUnique (_s : State) : Prop := True

/-- Every active membership has at least one voter. -/
def WF_MembershipNonempty (s : State) : Prop :=
  s.membership.all (fun p =>
    let m := p.2
    !ListSet.empty? m.old_voters || !ListSet.empty? m.new_voters) = true

/-- Conjunction of named foundation well-formedness predicates. -/
def WellFormed (s : State) : Prop :=
  WF_CollectionsQualified s ∧
  WF_GenerationOwnership s ∧
  WF_CurrentCommitted s ∧
  WF_CurrentUnique s ∧
  WF_ValueEvidence s ∧
  WF_DamageHonesty s ∧
  WF_CredentialHeapBinding s ∧
  WF_DelegationConfinement s ∧
  WF_AtomicMemberQualification s ∧
  WF_AtomicDecisionUnique s ∧
  WF_LogIndexUnique s ∧
  WF_MembershipNonempty s

/-- FAS-2 MUST prove: `WellFormed Init`. Empty maps/lists make each conjunct `true = true`. -/
theorem init_well_formed : WellFormed Init :=
  ⟨rfl, trivial, rfl, trivial, rfl, trivial, rfl, rfl, rfl, rfl, trivial, rfl⟩

end Residiuum