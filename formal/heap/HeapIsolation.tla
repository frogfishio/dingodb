//! TLA+ placeholder for HeapIsolation (HP-001 verification scaffold).
---- MODULE HeapIsolation ----
EXTENDS Naturals

\* Full model lands with H6; this file reserves the formal/heap path.
VARIABLES owners

Init == owners = {}
Next == UNCHANGED owners
Spec == Init /\ [][Next]_owners
====
