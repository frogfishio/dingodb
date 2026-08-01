import Residiuum.Observe

/-!
# Residiuum.Vectors

Accepted / rejected finite model vectors for FAS-2 foundation gate.
-/

namespace Residiuum.Vectors

open Residiuum

/-- Vector: Init is well-formed. -/
theorem vec_init_wf : WellFormed Init := init_well_formed

/-- Vector: create_heap then get missing collection → unknown, not absent. -/
def vec_heap_then_get_unknown : ObservationKind :=
  let h : HeapId := ⟨1⟩
  let s := afterCreateHeap h
  let item : ItemId := ⟨h, ⟨1⟩, ⟨1⟩⟩
  (Observe ⟨0⟩ (Scope.item item) s).kind

theorem vec_heap_then_get_unknown_kind :
    vec_heap_then_get_unknown = ObservationKind.unknown := by
  native_decide

/-- Vector: put then get yields complete. -/
def vec_put_get_complete : ObservationKind :=
  let h : HeapId := ⟨1⟩
  let c : CollectionId := ⟨1⟩
  let k : Key := ⟨1⟩
  let item : ItemId := ⟨h, c, k⟩
  let gen : Generation := ⟨1⟩
  let val : Value := ⟨42⟩
  let ev : EventId := ⟨7⟩
  let s0 := afterCreateHeap h
  let s1 := (stepFoundation s0 (Input.createCollection ⟨h, c⟩)).1
  let s2 := (stepFoundation s1 (Input.put item gen val ev)).1
  (Observe ⟨0⟩ (Scope.item item) s2).kind

theorem vec_put_get_complete_kind :
    vec_put_get_complete = ObservationKind.complete := by
  native_decide

/-- Negative vector: partial must not be treated as complete by public projection. -/
theorem vec_partial_not_public_complete :
    toPublicError ObservationKind.kindPartial ≠ PublicError.okComplete := by
  intro h; cases h

/-- Negative vector: damaged must not project to absent. -/
theorem vec_damaged_not_public_absent :
    toPublicError ObservationKind.damaged ≠ PublicError.okAbsent := by
  intro h; cases h

/-- Negative: forbidden collapse partial → complete is registered. -/
theorem vec_forbidden_partial_complete :
    ForbiddenCollapse ObservationKind.kindPartial ObservationKind.complete :=
  ForbiddenCollapse.partial_complete

/-- And id projection does not enact it. -/
theorem vec_id_avoids_partial_complete :
    ¬ ForbiddenCollapse ObservationKind.kindPartial
        (idProjection ObservationKind.kindPartial) :=
  id_projection_no_forbidden_collapse ObservationKind.kindPartial

end Residiuum.Vectors
