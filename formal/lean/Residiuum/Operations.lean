import Residiuum.WellFormed

/-!
# Residiuum.Operations

Abstract `Input`, foundation `Step`, operation contracts, and WF preservation.
Normative: `FORMAL_KERNEL_MODEL_CONTRACT.md` §7–§8.
-/

namespace Residiuum

/-- Closed tagged union of abstract inputs. -/
inductive Input where
  | createHeap (heap : HeapId)
  | createCollection (qc : QualifiedCollection)
  | put (item : ItemId) (gen : Generation) (value : Value) (event : EventId)
  | delete (item : ItemId) (gen : Generation) (event : EventId)
  | get (item : ItemId)
  | scan (qc : QualifiedCollection)
  | recover (heap : HeapId)
  | reassemble (item : ItemId) (gen : Generation)
  | repair (event : EventId)
  | issueCredential (id : CredentialId) (cred : Credential)
  | blacklistCredential (heap : HeapId) (id : CredentialId)
  | rotateEpoch (heap : HeapId) (epoch : Nat)
  | atomicPrepare (id : AtomicId) (ast : AtomicState)
  | atomicDecide (id : AtomicId) (decision : AtomicDecision)
  | atomicRecover (id : AtomicId)
  | clusterAppend (node : NodeId) (idx : LogIndex) (entry : LogEntry)
  | clusterElect (node : NodeId) (term : Term)
  | clusterChangeMembership (term : Term) (m : Membership)
  | clusterRepair (node : NodeId)
  deriving Repr

/-- Stable operation_id strings matching registry / foundation contract. -/
def Input.operationId : Input → String
  | .createHeap _ => "create_heap"
  | .createCollection _ => "create_collection"
  | .put _ _ _ _ => "put"
  | .delete _ _ _ => "delete"
  | .get _ => "get"
  | .scan _ => "scan"
  | .recover _ => "recover"
  | .reassemble _ _ => "reassemble"
  | .repair _ => "repair"
  | .issueCredential _ _ => "issue_credential"
  | .blacklistCredential _ _ => "blacklist_credential"
  | .rotateEpoch _ _ => "rotate_epoch"
  | .atomicPrepare _ _ => "atomic_prepare"
  | .atomicDecide _ _ => "atomic_decide"
  | .atomicRecover _ => "atomic_recover"
  | .clusterAppend _ _ _ => "cluster_append"
  | .clusterElect _ _ => "cluster_elect"
  | .clusterChangeMembership _ _ => "cluster_change_membership"
  | .clusterRepair _ => "cluster_repair"

/-- Complete catalogue of foundation operation ids (FAS-2 contract completeness). -/
def foundationOperationIds : List String :=
  [ "create_heap", "create_collection"
  , "put", "delete", "get", "scan"
  , "recover", "reassemble", "repair"
  , "issue_credential", "blacklist_credential", "rotate_epoch"
  , "atomic_prepare", "atomic_decide", "atomic_recover"
  , "cluster_append", "cluster_elect", "cluster_change_membership", "cluster_repair"
  ]

/-- Every Input constructor maps to a registered operation id. -/
theorem input_has_operation_id (i : Input) :
    i.operationId ∈ foundationOperationIds := by
  cases i <;> simp [Input.operationId, foundationOperationIds]

/-- Read vs write classification. -/
inductive OpClass where
  | read
  | write
  deriving DecidableEq, Repr

def Input.opClass : Input → OpClass
  | .get _ | .scan _ => .read
  | _ => .write

/-- Deterministic vs relational classification. -/
inductive Determinism where
  | deterministic
  | relational
  deriving DecidableEq, Repr

def Input.determinism : Input → Determinism
  | .recover _ | .reassemble _ _ | .repair _ | .atomicRecover _ | .clusterRepair _ => .relational
  | _ => .deterministic

/-- Foundation Step for core data-path ops. -/
def stepFoundation (s : State) (i : Input) : State × Outcome (Observation Value) :=
  match i with
  | .createHeap h =>
      if ListSet.mem s.heaps h then
        (s, Outcome.rejected RejectReason.conflict)
      else
        let s' := { s with
          heaps := ListSet.insert s.heaps h
          heap_epoch := ListMap.insert s.heap_epoch h 0 }
        (s', Outcome.ok (Observation.complete ⟨0⟩ ⟨0⟩))
  | .createCollection qc =>
      if !(ListSet.mem s.heaps qc.heap) then
        (s, Outcome.rejected RejectReason.invalidInput)
      else if ListSet.mem s.collections qc then
        (s, Outcome.rejected RejectReason.conflict)
      else
        let s' := { s with collections := ListSet.insert s.collections qc }
        (s', Outcome.ok (Observation.complete ⟨0⟩ ⟨0⟩))
  | .put item gen value event =>
      if !(ListSet.mem s.heaps item.heap) then
        (s, Outcome.rejected RejectReason.invalidInput)
      else if !(ListSet.mem s.collections ⟨item.heap, item.collection⟩) then
        (s, Outcome.rejected RejectReason.invalidInput)
      else
        let gref : GenerationRef := ⟨item, gen⟩
        let gens := (s.generations.get item).getD []
        let s' : State := {
          s with
          generations := ListMap.insert s.generations item (ListSet.insert gens gen)
          values := ListMap.insert s.values gref value
          publication := ListMap.insert s.publication gref Publication.committed
          current := ListMap.insert s.current item gen
          generation_events := ListMap.insert s.generation_events gref (ListSet.insert [] event)
          durable_events := ListSet.insert s.durable_events event
          coverage := ListSet.insert s.coverage event
        }
        (s', Outcome.ok (Observation.complete value ⟨event.val⟩))
  | .get item =>
      match s.current.get item with
      | none =>
          if ListSet.mem s.collections ⟨item.heap, item.collection⟩ then
            (s, Outcome.ok (Observation.absentProved ⟨0⟩))
          else
            (s, Outcome.ok (Observation.unknown ⟨0⟩))
      | some gen =>
          let gref : GenerationRef := ⟨item, gen⟩
          match s.values.get gref, s.pubOf gref with
          | some v, Publication.committed =>
              (s, Outcome.ok (Observation.complete v ⟨0⟩))
          | _, Publication.prepared =>
              (s, Outcome.ok (Observation.partialObs ⟨0⟩))
          | _, _ =>
              (s, Outcome.ok (Observation.unknown ⟨0⟩))
  | .delete item gen event =>
      if !(ListSet.mem s.heaps item.heap) then
        (s, Outcome.rejected RejectReason.invalidInput)
      else
        let gref : GenerationRef := ⟨item, gen⟩
        let s' : State := {
          s with
          publication := ListMap.insert s.publication gref Publication.retired
          current := s.current.filter (fun p => p.1 ≠ item)
          durable_events := ListSet.insert s.durable_events event
        }
        (s', Outcome.ok (Observation.absentProved ⟨event.val⟩))
  | _ =>
      (s, Outcome.rejected RejectReason.unsupported)

/-- Relational form of Step (Prop). -/
def Step (s : State) (i : Input) (s' : State) (o : Outcome (Observation Value)) : Prop :=
  stepFoundation s i = (s', o)

/-- Explicit post-state of createHeap on Init (definitional, no branching). -/
def afterCreateHeap (h : HeapId) : State where
  heaps := [h]
  collections := []
  generations := []
  values := []
  publication := []
  current := []
  generation_events := []
  durable_events := []
  damaged_events := []
  coverage := []
  credentials := []
  heap_epoch := [(h, 0)]
  blacklist := []
  active_rules := []
  atomics := []
  nodes := []
  log := []
  membership := []

/-- WellFormed preservation for create_heap from Init (empty residual maps). -/
theorem create_heap_preserves_wf (h : HeapId) :
    WellFormed (afterCreateHeap h) :=
  ⟨rfl, trivial, rfl, trivial, rfl, trivial, rfl, rfl, rfl, rfl, trivial, rfl⟩

/-- stepFoundation createHeap on Init agrees with afterCreateHeap (heaps start empty). -/
theorem step_create_heap_init (h : HeapId) :
    (stepFoundation Init (Input.createHeap h)).1 = afterCreateHeap h := by
  -- Init.heaps = []; mem is definitionally false
  rfl

/-- Crash is not an ordinary Input (separate relation stub for FAS-3). -/
def CrashStep (_prefix : Unit) (_concrete : Unit) : Prop := True

/-- Recovery step relation stub. -/
def RecoverStep (s s' : State) (o : Outcome Unit) : Prop :=
  s' = s ∧ o = Outcome.ok ()

end Residiuum