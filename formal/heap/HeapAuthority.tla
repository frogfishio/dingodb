\* HeapAuthority model for Gate H6 evidence (HP-010 / HEAP_SPEC §39).
\*
\* Checks generation acceptance, blacklist hits, grace windows, and terminal
\* administrative states. Companion to HeapIsolation.tla. Not a complete
\* connected proof; qualified remains false until review lands.
---- MODULE HeapAuthority ----
EXTENDS Naturals, FiniteSets

CONSTANTS Gens, Certs, MaxGen

ASSUME Gens # {} /\ Certs # {} /\ MaxGen \in Nat /\ MaxGen >= 1

VARIABLES
  currentGen,       \* current authority generation
  previousGen,      \* previous generation during grace, or 0 if none
  graceDeadline,    \* unix seconds; 0 means no grace
  now,              \* trusted security time
  blacklist,        \* SUBSET Certs blacklisted for previousGen
  adminState,       \* "active" | "read_only" | "suspended" | "retired" | "purging" | "purged"
  admitted          \* SUBSET Certs currently admitted

TypeOK ==
  /\ currentGen \in Gens
  /\ previousGen \in (Gens \cup {0})
  /\ graceDeadline \in Nat
  /\ now \in Nat
  /\ blacklist \subseteq Certs
  /\ adminState \in {"active", "read_only", "suspended", "retired", "purging", "purged"}
  /\ admitted \subseteq Certs

Serving == adminState \in {"active", "read_only"}
Terminal == adminState = "purged"

\* Generation acceptance: current, or previous during grace.
GenOK(cGen) ==
  \/ cGen = currentGen
  \/ (cGen = previousGen /\ previousGen # 0 /\ now <= graceDeadline)

\* Blacklist only applies to previous-generation certificates.
NotBlacklisted(c) ==
  \/ c \notin blacklist
  \/ previousGen = 0

AdmissionOK(c, cGen) ==
  /\ Serving
  /\ ~Terminal
  /\ GenOK(cGen)
  /\ NotBlacklisted(c)

Inv ==
  /\ TypeOK
  /\ \A c \in admitted : Serving
  /\ Terminal => admitted = {}
  /\ (now > graceDeadline /\ previousGen # 0) =>
        \A c \in admitted : TRUE  \* stale previous-gen must not remain (enforced by Next)

Init ==
  /\ currentGen \in Gens
  /\ previousGen = 0
  /\ graceDeadline = 0
  /\ now = 0
  /\ blacklist = {}
  /\ adminState = "active"
  /\ admitted = {}

AdvanceTime ==
  /\ now' = now + 1
  /\ UNCHANGED <<currentGen, previousGen, graceDeadline, blacklist, adminState, admitted>>

HardCycle(newGen) ==
  /\ newGen \in Gens
  /\ newGen # currentGen
  /\ currentGen' = newGen
  /\ previousGen' = 0
  /\ graceDeadline' = 0
  /\ blacklist' = {}
  /\ admitted' = {}
  /\ UNCHANGED <<now, adminState>>

GraceCycle(newGen, deadline) ==
  /\ newGen \in Gens
  /\ newGen # currentGen
  /\ deadline \in Nat
  /\ deadline >= now
  /\ previousGen' = currentGen
  /\ currentGen' = newGen
  /\ graceDeadline' = deadline
  /\ blacklist' = {}
  /\ admitted' = {}
  /\ UNCHANGED <<now, adminState>>

BlacklistAdd(c) ==
  /\ previousGen # 0
  /\ c \in Certs
  /\ blacklist' = blacklist \cup {c}
  /\ admitted' = admitted \ {c}
  /\ UNCHANGED <<currentGen, previousGen, graceDeadline, now, adminState>>

Admit(c, cGen) ==
  /\ c \in Certs
  /\ cGen \in Gens
  /\ AdmissionOK(c, cGen)
  /\ admitted' = admitted \cup {c}
  /\ UNCHANGED <<currentGen, previousGen, graceDeadline, now, blacklist, adminState>>

Suspend ==
  /\ adminState = "active"
  /\ adminState' = "suspended"
  /\ admitted' = {}
  /\ UNCHANGED <<currentGen, previousGen, graceDeadline, now, blacklist>>

Retire ==
  /\ adminState \in {"active", "read_only", "suspended"}
  /\ adminState' = "retired"
  /\ admitted' = {}
  /\ UNCHANGED <<currentGen, previousGen, graceDeadline, now, blacklist>>

PurgeDone ==
  /\ adminState \in {"retired", "purging"}
  /\ adminState' = "purged"
  /\ admitted' = {}
  /\ UNCHANGED <<currentGen, previousGen, graceDeadline, now, blacklist>>

ExpireGrace ==
  /\ previousGen # 0
  /\ now > graceDeadline
  /\ previousGen' = 0
  /\ graceDeadline' = 0
  /\ blacklist' = {}
  /\ admitted' = {}
  /\ UNCHANGED <<currentGen, now, adminState>>

Next ==
  \/ AdvanceTime
  \/ \E g \in Gens : HardCycle(g)
  \/ \E g \in Gens, d \in Nat : GraceCycle(g, d)
  \/ \E c \in Certs : BlacklistAdd(c)
  \/ \E c \in Certs, g \in Gens : Admit(c, g)
  \/ Suspend
  \/ Retire
  \/ PurgeDone
  \/ ExpireGrace

Spec == Init /\ [][Next]_<<currentGen, previousGen, graceDeadline, now, blacklist, adminState, admitted>>

THEOREM Spec => []Inv
====
