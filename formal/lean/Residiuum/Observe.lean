import Residiuum.Operations

/-!
# Residiuum.Observe

Total pure observation law from abstract state.
Normative: `FORMAL_KERNEL_MODEL_CONTRACT.md` §9.
-/

namespace Residiuum

/-- Observation scope: a single item, a collection scan, or heap-level. -/
inductive Scope where
  | item (id : ItemId)
  | collection (qc : QualifiedCollection)
  | heap (h : HeapId)
  deriving Repr, DecidableEq

/--
Observe : PrincipalId → Scope → State → Observation Value

Pure and total. Establishes authority first (foundation: always authorized),
then derives knowledge from authoritative evidence — never from a derived index alone.
-/
def Observe (_principal : PrincipalId) (scope : Scope) (s : State) : Observation Value :=
  match scope with
  | .item item =>
      if !(ListSet.mem s.heaps item.heap) then
        Observation.unavailable ⟨0⟩
      else if !(ListSet.mem s.collections ⟨item.heap, item.collection⟩) then
        Observation.unknown ⟨0⟩
      else
        match s.current.get item with
        | none =>
            Observation.absentProved ⟨0⟩
        | some gen =>
            let gref : GenerationRef := ⟨item, gen⟩
            match s.publication.get gref with
            | some Publication.committed =>
                match s.values.get gref with
                | some v =>
                    match s.generation_events.get gref with
                    | some evs =>
                        if ListSet.empty? evs then
                          Observation.unknown ⟨0⟩
                        else if evs.any (fun e => ListSet.mem s.damaged_events e) then
                          Observation.damaged ⟨0⟩
                        else
                          Observation.complete v ⟨0⟩
                    | none => Observation.unknown ⟨0⟩
                | none => Observation.unknown ⟨0⟩
            | some Publication.prepared => Observation.partialObs ⟨0⟩
            | some Publication.unpublished => Observation.unknown ⟨0⟩
            | some Publication.retired => Observation.absentProved ⟨0⟩
            | none => Observation.unknown ⟨0⟩
  | .collection qc =>
      if !(ListSet.mem s.heaps qc.heap) then
        Observation.unavailable ⟨0⟩
      else if !(ListSet.mem s.collections qc) then
        Observation.unknown ⟨0⟩
      else
        Observation.partialObs ⟨0⟩
  | .heap h =>
      if !(ListSet.mem s.heaps h) then
        Observation.unavailable ⟨0⟩
      else
        Observation.partialObs ⟨0⟩

/-- Observe is total (defined for all arguments). -/
theorem observe_total (p : PrincipalId) (sc : Scope) (s : State) :
    ∃ o : Observation Value, Observe p sc s = o :=
  ⟨Observe p sc s, rfl⟩

/-- Observe never invents complete on empty Init. -/
theorem observe_init_item_not_complete (p : PrincipalId) (item : ItemId) :
    (Observe p (Scope.item item) Init).kind ≠ ObservationKind.complete := by
  simp [Observe, Init, ListSet.mem, ListSet.empty, Observation.kind]

/-- Prepared publication projects to partialObs, never complete. -/
theorem observe_prepared_is_partial (p : PrincipalId) (item : ItemId) (gen : Generation)
    (s : State)
    (hHeap : ListSet.mem s.heaps item.heap = true)
    (hColl : ListSet.mem s.collections ⟨item.heap, item.collection⟩ = true)
    (hCur : s.current.get item = some gen)
    (hPub : s.publication.get ⟨item, gen⟩ = some Publication.prepared) :
    (Observe p (Scope.item item) s).kind = ObservationKind.kindPartial := by
  simp [Observe, hHeap, hColl, hCur, hPub, Observation.kind]

end Residiuum