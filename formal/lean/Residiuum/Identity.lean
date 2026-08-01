/-!
# Residiuum.Identity

Primitive identities and qualified keys for the FAS-2 abstract kernel.
Normative: `FORMAL_KERNEL_MODEL_CONTRACT.md` §2.
-/

namespace Residiuum

/-- Opaque heap identifier. Theorems must not depend on encoding. -/
structure HeapId where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Opaque collection identifier (qualified only with HeapId). -/
structure CollectionId where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Opaque item key within a collection. -/
structure Key where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Generation number for an item version. -/
structure Generation where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Durable event identity. -/
structure EventId where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Principal (subject) identity. -/
structure PrincipalId where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Credential identity. -/
structure CredentialId where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Atomic (multi-item commit) identity. -/
structure AtomicId where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Cluster node identity. -/
structure NodeId where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Raft-style term. -/
structure Term where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Log index on a node. -/
structure LogIndex where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Rule identity (active policy). -/
structure RuleId where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Opaque value payload (abstract). -/
structure Value where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Evidence digest for observation/outcome honesty. -/
structure EvidenceDigest where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Abstract time (epoch wall-clock stand-in). -/
structure Time where
  val : Nat
  deriving DecidableEq, Repr, Inhabited

/-- Collection always qualified by Heap — no unqualified collection namespace. -/
structure QualifiedCollection where
  heap : HeapId
  collection : CollectionId
  deriving DecidableEq, Repr, Inhabited

/-- Item identity: heap × collection × key. -/
structure ItemId where
  heap : HeapId
  collection : CollectionId
  key : Key
  deriving DecidableEq, Repr, Inhabited

/-- Generation reference: item × generation. -/
structure GenerationRef where
  item : ItemId
  generation : Generation
  deriving DecidableEq, Repr, Inhabited

/-- Minimal list-map used in place of Finset/Mathlib maps for kernel purity. -/
abbrev ListMap (α β : Type) := List (α × β)

namespace ListMap

variable {α β : Type} [DecidableEq α]

def empty : ListMap α β := []

def get (m : ListMap α β) (k : α) : Option β :=
  match m with
  | [] => none
  | (k', v) :: rest => if k = k' then some v else get rest k

def insert (m : ListMap α β) (k : α) (v : β) : ListMap α β :=
  (k, v) :: m.filter (fun p => p.1 ≠ k)

def contains (m : ListMap α β) (k : α) : Bool :=
  (get m k).isSome

def keys (m : ListMap α β) : List α :=
  m.map (·.1)

end ListMap

/-- Finite set as list with membership helpers (no Mathlib). -/
abbrev ListSet (α : Type) := List α

namespace ListSet

variable {α : Type} [DecidableEq α]

def empty : ListSet α := []

def mem : ListSet α → α → Bool
  | [], _ => false
  | y :: ys, x => if x = y then true else mem ys x

def insert (s : ListSet α) (x : α) : ListSet α :=
  if mem s x then s else x :: s

def subset (a b : ListSet α) : Bool :=
  a.all (fun x => mem b x)

def empty? : ListSet α → Bool
  | [] => true
  | _ => false

end ListSet

end Residiuum