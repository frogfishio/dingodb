import Residiuum.Identity
import Residiuum.Observation

/-!
# Residiuum.State

Canonical abstract `State` record, feature subrecords, and `Init`.
Normative: `FORMAL_KERNEL_MODEL_CONTRACT.md` §4, §6.
-/


namespace Residiuum

/-- Credential record (minimum fields from kernel contract). -/
structure Credential where
  heap : HeapId
  subject : PrincipalId
  epoch : Nat
  rights : ListSet Nat
  kind : Nat
  parent : Option CredentialId
  not_before : Time
  not_after : Time
  signature : EvidenceDigest
  deriving Repr

/-- Atomic multi-item state. -/
structure AtomicState where
  heap : HeapId
  exact_members : ListSet ItemId
  prepared_members : ListSet ItemId
  decision : AtomicDecision
  evidence : EvidenceDigest
  deriving Repr

/-- Cluster node local state. -/
structure NodeState where
  term : Term
  role : NodeRole
  voted_for : Option NodeId
  commit_index : LogIndex
  applied_index : LogIndex
  deriving Repr

/-- Log entry on a node. -/
structure LogEntry where
  term : Term
  heap : HeapId
  operation_digest : EvidenceDigest
  decision_digest : EvidenceDigest
  deriving Repr

/-- Membership change record. -/
structure Membership where
  old_voters : ListSet NodeId
  new_voters : ListSet NodeId
  phase : Nat
  deriving Repr

/--
Canonical abstract Residiuum state.
Maps are total via ListMap with neutral defaults outside the domain.
-/
structure State where
  heaps : ListSet HeapId
  collections : ListSet QualifiedCollection
  generations : ListMap ItemId (ListSet Generation)
  values : ListMap GenerationRef Value
  publication : ListMap GenerationRef Publication
  current : ListMap ItemId Generation
  generation_events : ListMap GenerationRef (ListSet EventId)
  durable_events : ListSet EventId
  damaged_events : ListSet EventId
  coverage : ListSet EventId
  credentials : ListMap CredentialId Credential
  heap_epoch : ListMap HeapId Nat
  blacklist : ListMap HeapId (ListSet CredentialId)
  active_rules : ListMap QualifiedCollection (ListSet RuleId)
  atomics : ListMap AtomicId AtomicState
  nodes : ListMap NodeId NodeState
  log : ListMap (NodeId × LogIndex) LogEntry
  membership : ListMap Term Membership
  deriving Repr

/-- Empty initial state — no heaps, collections, credentials, atomics, nodes. -/
def Init : State where
  heaps := ListSet.empty
  collections := ListSet.empty
  generations := ListMap.empty
  values := ListMap.empty
  publication := ListMap.empty
  current := ListMap.empty
  generation_events := ListMap.empty
  durable_events := ListSet.empty
  damaged_events := ListSet.empty
  coverage := ListSet.empty
  credentials := ListMap.empty
  heap_epoch := ListMap.empty
  blacklist := ListMap.empty
  active_rules := ListMap.empty
  atomics := ListMap.empty
  nodes := ListMap.empty
  log := ListMap.empty
  membership := ListMap.empty

/-- Helper: publication of a generation ref. -/
def State.pubOf (s : State) (gref : GenerationRef) : Publication :=
  (s.publication.get gref).getD .unpublished

/-- Helper: whether an item has a current generation. -/
def State.currentOf (s : State) (item : ItemId) : Option Generation :=
  s.current.get item

/-- Helper: value of a generation ref. -/
def State.valueOf (s : State) (gref : GenerationRef) : Option Value :=
  s.values.get gref

/-- Whether a heap exists in state. -/
def State.hasHeap (s : State) (h : HeapId) : Bool :=
  ListSet.mem s.heaps h

/-- Whether a qualified collection exists. -/
def State.hasCollection (s : State) (qc : QualifiedCollection) : Bool :=
  ListSet.mem s.collections qc

end Residiuum
