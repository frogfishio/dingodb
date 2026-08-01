import Residiuum.Identity

/-!
# Residiuum.Observation

Closed observation, outcome, publication, and atomic-decision constructors.
Normative: `FORMAL_KERNEL_MODEL_CONTRACT.md` §3, §9.

Note: Lean keyword `partial` is spelled `partialObs` / `kindPartial` in source.
Semantic meaning matches the contract's `partial`.
-/

namespace Residiuum

/-- Observation of an abstract value — constructors are closed and distinct. -/
inductive Observation (α : Type) where
  | complete (value : α) (evidence : EvidenceDigest)
  | absentProved (evidence : EvidenceDigest)
  | partialObs (evidence : EvidenceDigest)
  | damaged (evidence : EvidenceDigest)
  | unknown (evidence : EvidenceDigest)
  | unauthorized
  | unavailable (evidence : EvidenceDigest)
  deriving Repr

/-- Rejection reasons for failed operations. -/
inductive RejectReason where
  | unauthorized
  | invalidInput
  | invariantViolation
  | conflict
  | staleEpoch
  | blacklisted
  | forbiddenSurface
  | unsupported
  deriving DecidableEq, Repr, Inhabited

/-- Operation outcome preserving indeterminate/unavailable honesty. -/
inductive Outcome (α : Type) where
  | ok (value : α)
  | rejected (reason : RejectReason)
  | indeterminate (evidence : EvidenceDigest)
  | unavailable (evidence : EvidenceDigest)
  deriving Repr

/-- Publication state of a generation. -/
inductive Publication where
  | unpublished
  | prepared
  | committed
  | retired
  deriving DecidableEq, Repr, Inhabited

/-- Atomic multi-item decision. -/
inductive AtomicDecision where
  | undecided
  | commit
  | abort
  deriving DecidableEq, Repr, Inhabited

/-- Cluster node role. -/
inductive NodeRole where
  | follower
  | candidate
  | leader
  deriving DecidableEq, Repr, Inhabited

/-- Tag for observation constructor kind (for separation / collapse checks). -/
inductive ObservationKind where
  | complete
  | absentProved
  | kindPartial
  | damaged
  | unknown
  | unauthorized
  | unavailable
  deriving DecidableEq, Repr, Inhabited

/-- Extract constructor kind from an observation. -/
def Observation.kind {α : Type} : Observation α → ObservationKind
  | Observation.complete _ _ => ObservationKind.complete
  | Observation.absentProved _ => ObservationKind.absentProved
  | Observation.partialObs _ => ObservationKind.kindPartial
  | Observation.damaged _ => ObservationKind.damaged
  | Observation.unknown _ => ObservationKind.unknown
  | Observation.unauthorized => ObservationKind.unauthorized
  | Observation.unavailable _ => ObservationKind.unavailable

/--
FAS-FND-OBSERVATION-SEPARATION-001:
Constructor kinds are pairwise distinct; an observation has exactly one kind.
-/
theorem observation_kind_exhaustive {α : Type} (o : Observation α) :
    o.kind = ObservationKind.complete ∨
    o.kind = ObservationKind.absentProved ∨
    o.kind = ObservationKind.kindPartial ∨
    o.kind = ObservationKind.damaged ∨
    o.kind = ObservationKind.unknown ∨
    o.kind = ObservationKind.unauthorized ∨
    o.kind = ObservationKind.unavailable := by
  cases o <;> simp [Observation.kind]

theorem kind_complete_ne_absent :
    ObservationKind.complete ≠ ObservationKind.absentProved := by
  intro h; cases h

theorem kind_partial_ne_complete :
    ObservationKind.kindPartial ≠ ObservationKind.complete := by
  intro h; cases h

theorem kind_damaged_ne_absent :
    ObservationKind.damaged ≠ ObservationKind.absentProved := by
  intro h; cases h

theorem kind_unknown_ne_complete :
    ObservationKind.unknown ≠ ObservationKind.complete := by
  intro h; cases h

theorem complete_ne_absent {α : Type} (v : α) (e₁ e₂ : EvidenceDigest) :
    Observation.kind (Observation.complete v e₁ : Observation α) ≠
      Observation.kind (Observation.absentProved e₂ : Observation α) :=
  kind_complete_ne_absent

theorem partial_ne_complete {α : Type} (v : α) (e₁ e₂ : EvidenceDigest) :
    Observation.kind (Observation.partialObs e₁ : Observation α) ≠
      Observation.kind (Observation.complete v e₂ : Observation α) :=
  kind_partial_ne_complete

theorem damaged_ne_absent {α : Type} (e₁ e₂ : EvidenceDigest) :
    Observation.kind (Observation.damaged e₁ : Observation α) ≠
      Observation.kind (Observation.absentProved e₂ : Observation α) :=
  kind_damaged_ne_absent

theorem unknown_ne_complete {α : Type} (v : α) (e₁ e₂ : EvidenceDigest) :
    Observation.kind (Observation.unknown e₁ : Observation α) ≠
      Observation.kind (Observation.complete v e₂ : Observation α) :=
  kind_unknown_ne_complete

/--
Forbidden collapse pairs from KERNEL_MODEL §9.
A public projection must not map `from` onto `to`.
-/
inductive ForbiddenCollapse : ObservationKind → ObservationKind → Prop where
  | partial_absent :
      ForbiddenCollapse ObservationKind.kindPartial ObservationKind.absentProved
  | partial_complete :
      ForbiddenCollapse ObservationKind.kindPartial ObservationKind.complete
  | damaged_absent :
      ForbiddenCollapse ObservationKind.damaged ObservationKind.absentProved
  | damaged_complete :
      ForbiddenCollapse ObservationKind.damaged ObservationKind.complete
  | unknown_absent :
      ForbiddenCollapse ObservationKind.unknown ObservationKind.absentProved
  | unknown_complete :
      ForbiddenCollapse ObservationKind.unknown ObservationKind.complete
  | unauthorized_absent :
      ForbiddenCollapse ObservationKind.unauthorized ObservationKind.absentProved
  | unavailable_absent :
      ForbiddenCollapse ObservationKind.unavailable ObservationKind.absentProved

/-- Identity projection on observation kinds — never collapses. -/
def idProjection (k : ObservationKind) : ObservationKind := k

/-- FAS-FND-FORBIDDEN-COLLAPSE-001: identity projection never performs a forbidden collapse. -/
theorem id_projection_no_forbidden_collapse (src : ObservationKind) :
    ¬ ForbiddenCollapse src (idProjection src) := by
  intro h
  cases h

/--
Coarse public error kind: failures may coarsen among failure states,
but must not promote failure into complete/absentProved.
-/
inductive PublicError where
  | okComplete
  | okAbsent
  | error
  deriving DecidableEq, Repr

/-- Allowed public coarsening. -/
def toPublicError (k : ObservationKind) : PublicError :=
  match k with
  | ObservationKind.complete => PublicError.okComplete
  | ObservationKind.absentProved => PublicError.okAbsent
  | ObservationKind.kindPartial
  | ObservationKind.damaged
  | ObservationKind.unknown
  | ObservationKind.unauthorized
  | ObservationKind.unavailable => PublicError.error

/-- Public coarsening does not map failure kinds to complete. -/
theorem public_error_no_complete_collapse (k : ObservationKind)
    (h : k = ObservationKind.kindPartial ∨ k = ObservationKind.damaged ∨
         k = ObservationKind.unknown ∨ k = ObservationKind.unauthorized ∨
         k = ObservationKind.unavailable) :
    toPublicError k ≠ PublicError.okComplete := by
  rcases h with h | h | h | h | h <;> subst h <;> simp [toPublicError]

/-- Public coarsening does not map failure kinds to absent. -/
theorem public_error_no_absent_collapse (k : ObservationKind)
    (h : k = ObservationKind.kindPartial ∨ k = ObservationKind.damaged ∨
         k = ObservationKind.unknown ∨ k = ObservationKind.unauthorized ∨
         k = ObservationKind.unavailable) :
    toPublicError k ≠ PublicError.okAbsent := by
  rcases h with h | h | h | h | h <;> subst h <;> simp [toPublicError]

end Residiuum