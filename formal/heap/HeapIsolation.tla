\* HeapIsolation model for Gate H6 evidence (HP-010 / HEAP_SPEC §39).
\*
\* This is an executable TLA+ sketch checked by humans and (when TLC is
\* available) by scripts/verify-heap.sh full. It is not yet a complete
\* connected proof; qualified remains false until H0-H5 + review land.
---- MODULE HeapIsolation ----
EXTENDS Naturals, FiniteSets, Sequences

CONSTANTS Heaps, Units, MaxUnits

ASSUME Heaps # {} /\ Units # {} /\ MaxUnits \in Nat /\ MaxUnits >= 1

\* Each independently damageable unit maps to at most one owner heap, or None.
VARIABLES
  owners,          \* [unit \in Units |-> heap \in Heaps \cup {None}]
  admitted,        \* SUBSET Units currently admitted under their owner
  boundHeap        \* capability-bound heap for the observation under test

TypeOK ==
  /\ owners \in [Units -> (Heaps \cup {None})]
  /\ admitted \subseteq Units
  /\ boundHeap \in Heaps

\* Ownership disjointness: each unit has at most one owner.
OwnershipDisjoint ==
  \A u \in Units :
     owners[u] = None \/ owners[u] \in Heaps

\* Read confinement: an observation bound to boundHeap only admits units
\* whose owner equals boundHeap.
ReadConfinement ==
  \A u \in admitted : owners[u] = boundHeap

\* Write confinement: only units owned by boundHeap may enter admitted.
WriteConfinement ==
  \A u \in Units :
     owners[u] # boundHeap => u \notin admitted

\* Revocation: clearing ownership removes admission.
RevocationClosed ==
  \A u \in Units :
     owners[u] = None => u \notin admitted

Init ==
  /\ owners = [u \in Units |-> None]
  /\ admitted = {}
  /\ boundHeap \in Heaps

AssignOwner(u, h) ==
  /\ u \in Units /\ h \in Heaps
  /\ owners[u] = None
  /\ owners' = [owners EXCEPT ![u] = h]
  /\ UNCHANGED <<admitted, boundHeap>>

Admit(u) ==
  /\ u \in Units
  /\ owners[u] = boundHeap
  /\ admitted' = admitted \cup {u}
  /\ UNCHANGED <<owners, boundHeap>>

Damage(u) ==
  /\ u \in Units
  /\ owners' = [owners EXCEPT ![u] = None]
  /\ admitted' = admitted \ {u}
  /\ UNCHANGED boundHeap

Rebind(h) ==
  /\ h \in Heaps
  /\ boundHeap' = h
  /\ admitted' = {}          \* prior observation dies on rebind
  /\ UNCHANGED owners

Next ==
  \/ \E u \in Units, h \in Heaps : AssignOwner(u, h)
  \/ \E u \in Units : Admit(u)
  \/ \E u \in Units : Damage(u)
  \/ \E h \in Heaps : Rebind(h)

Spec == Init /\ [][Next]_<<owners, admitted, boundHeap>>

Inv ==
  /\ TypeOK
  /\ OwnershipDisjoint
  /\ ReadConfinement
  /\ WriteConfinement
  /\ RevocationClosed

THEOREM Spec => []Inv
====
