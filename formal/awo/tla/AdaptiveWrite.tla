\* Adaptive Write Optimiser — TLA+ skeleton (AWO-0).
\*
\* Variable and transition names are closed by
\* ADAPTIVE_WRITE_OPTIMISER_IMPLEMENTATION_PLAN.md §17 and
\* ADAPTIVE_WRITE_OPTIMISER_SPEC.md request/ordering/ACK laws.
\*
\* This is a bounded structural skeleton for AWO-0/G1. Full model-check
\* campaigns and FAS connection are AWO-6. TLC config: AdaptiveWrite.cfg.
---- MODULE AdaptiveWrite ----
EXTENDS Naturals, FiniteSets, Sequences

CONSTANTS
  Requests,          \* set of request ids (bounded)
  Lanes,             \* set of lane keys
  MaxTicket,         \* max tickets per lane
  MaxCookers,        \* maximum active cookers
  MaxQueueEntries    \* queue entry capacity

ASSUME
  /\ Requests # {}
  /\ Lanes # {}
  /\ MaxTicket \in Nat /\ MaxTicket >= 1
  /\ MaxCookers \in Nat /\ MaxCookers >= 1
  /\ MaxQueueEntries \in Nat /\ MaxQueueEntries >= 1

\* Request lifecycle states (states-v1.json closed set, simplified for model).
States == {
  "received", "queued", "cooking", "ready", "persisting",
  "persisted", "published", "acknowledged",
  "rejected", "failed", "uncertain_pending_recovery"
}

Terminal == {"acknowledged", "rejected", "failed", "uncertain_pending_recovery"}
AuthorityYes == {"persisted", "published", "acknowledged"}

VARIABLES
  reqState,          \* [r \in Requests |-> state]
  reqLane,           \* [r \in Requests |-> lane or None]
  reqTicket,         \* [r \in Requests |-> ticket or 0]
  reqDurability,     \* [r \in Requests |-> "buffered" | "durable" | "memory" | "none"]
  laneNextAdmit,     \* [l \in Lanes |-> next ticket to assign]
  laneNextInstall,   \* [l \in Lanes |-> next ticket to install]
  queueBytes,        \* total reserved queue bytes (abstract units)
  queueEntries,      \* total queued entries
  cookOwner,         \* [r \in Requests |-> cooker id or 0]
  ready,             \* SUBSET Requests cooked and ordered-ready
  reservation,       \* SUBSET Requests with open reservation
  persisted,         \* SUBSET Requests with successful persist
  published,         \* SUBSET Requests published
  acked,             \* SUBSET Requests acknowledged
  failed,            \* SUBSET Requests failed known
  uncertain,         \* SUBSET Requests uncertain pending recovery
  activeCookers,     \* number of active cooker permits
  controllerMode,    \* "disabled" | "static" | "adaptive"
  writerHealth       \* "ok" | "poisoned" | "draining"

vars == <<
  reqState, reqLane, reqTicket, reqDurability,
  laneNextAdmit, laneNextInstall,
  queueBytes, queueEntries, cookOwner, ready,
  reservation, persisted, published, acked, failed, uncertain,
  activeCookers, controllerMode, writerHealth
>>

None == 0

TypeOK ==
  /\ reqState \in [Requests -> States]
  /\ reqLane \in [Requests -> (Lanes \cup {None})]
  /\ reqTicket \in [Requests -> 0..MaxTicket]
  /\ reqDurability \in [Requests -> {"none", "memory", "buffered", "durable"}]
  /\ laneNextAdmit \in [Lanes -> 1..(MaxTicket + 1)]
  /\ laneNextInstall \in [Lanes -> 1..(MaxTicket + 1)]
  /\ queueBytes \in Nat
  /\ queueEntries \in 0..MaxQueueEntries
  /\ cookOwner \in [Requests -> 0..MaxCookers]
  /\ ready \subseteq Requests
  /\ reservation \subseteq Requests
  /\ persisted \subseteq Requests
  /\ published \subseteq Requests
  /\ acked \subseteq Requests
  /\ failed \subseteq Requests
  /\ uncertain \subseteq Requests
  /\ activeCookers \in 0..MaxCookers
  /\ controllerMode \in {"disabled", "static", "adaptive"}
  /\ writerHealth \in {"ok", "poisoned", "draining"}

\* SPEC: Ack(r) => PersistSucceeded(r) and VisibleAsAcknowledged => Persist.
AckImpliesPersisted ==
  acked \subseteq persisted

\* No ordinary publication before persist.
NoPublishBeforePersist ==
  published \subseteq persisted

\* Published implies ready for ack path; acked \subseteq published for successful path.
AckImpliesPublished ==
  acked \subseteq published

\* Terminal partition (at most one failure class).
TerminalDisjoint ==
  /\ acked \cap failed = {}
  /\ acked \cap uncertain = {}
  /\ failed \cap uncertain = {}

\* Install cursor never ahead of admit cursor.
TicketCursorOk ==
  \A l \in Lanes : laneNextInstall[l] <= laneNextAdmit[l]

Inv ==
  /\ TypeOK
  /\ AckImpliesPersisted
  /\ NoPublishBeforePersist
  /\ AckImpliesPublished
  /\ TerminalDisjoint
  /\ TicketCursorOk
  /\ writerHealth = "poisoned" => reservation = {}  \* no new reservation while poisoned (abstract)

----------------------------------------------------------------------------
Init ==
  /\ reqState = [r \in Requests |-> "received"]
  /\ reqLane = [r \in Requests |-> None]
  /\ reqTicket = [r \in Requests |-> 0]
  /\ reqDurability = [r \in Requests |-> "none"]
  /\ laneNextAdmit = [l \in Lanes |-> 1]
  /\ laneNextInstall = [l \in Lanes |-> 1]
  /\ queueBytes = 0
  /\ queueEntries = 0
  /\ cookOwner = [r \in Requests |-> 0]
  /\ ready = {}
  /\ reservation = {}
  /\ persisted = {}
  /\ published = {}
  /\ acked = {}
  /\ failed = {}
  /\ uncertain = {}
  /\ activeCookers = 1
  /\ controllerMode = "disabled"
  /\ writerHealth = "ok"

----------------------------------------------------------------------------
\* Named transitions (plan §17). Bodies are minimal stubs for AWO-0.

Receive(r) ==
  /\ r \in Requests
  /\ reqState[r] = "received"
  /\ UNCHANGED vars  \* identity placeholder; real model uses environment arrival

Reject(r) ==
  /\ r \in Requests
  /\ reqState[r] = "received"
  /\ writerHealth \in {"ok", "draining"}
  /\ reqState' = [reqState EXCEPT ![r] = "rejected"]
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready, reservation,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

Admit(r, l, d) ==
  /\ r \in Requests
  /\ l \in Lanes
  /\ d \in {"buffered", "durable"}
  /\ reqState[r] = "received"
  /\ writerHealth = "ok"
  /\ queueEntries < MaxQueueEntries
  /\ laneNextAdmit[l] <= MaxTicket
  /\ reqState' = [reqState EXCEPT ![r] = "queued"]
  /\ reqLane' = [reqLane EXCEPT ![r] = l]
  /\ reqTicket' = [reqTicket EXCEPT ![r] = laneNextAdmit[l]]
  /\ reqDurability' = [reqDurability EXCEPT ![r] = d]
  /\ laneNextAdmit' = [laneNextAdmit EXCEPT ![l] = laneNextAdmit[l] + 1]
  /\ queueEntries' = queueEntries + 1
  /\ queueBytes' = queueBytes + 1
  /\ UNCHANGED <<
      laneNextInstall, cookOwner, ready, reservation,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

FormNatural(r) ==
  /\ r \in Requests
  /\ reqState[r] = "queued"
  /\ writerHealth = "ok"
  /\ reqState' = [reqState EXCEPT ![r] = "ready"]  \* natural skips cook abstraction
  /\ ready' = ready \cup {r}
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, reservation,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

FormBatch(S) ==
  /\ S \subseteq Requests
  /\ S # {}
  /\ \A r \in S : reqState[r] = "queued"
  /\ writerHealth = "ok"
  /\ UNCHANGED vars  \* batch formation is scheduling; Reserve owns mutation

Reserve(S) ==
  /\ S \subseteq Requests
  /\ S # {}
  /\ \A r \in S : reqState[r] \in {"queued", "ready"}
  /\ writerHealth = "ok"
  /\ reservation = {}  \* V1: at most one unresolved reservation (abstract global)
  /\ reservation' = S
  /\ UNCHANGED <<
      reqState, reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

StartCook(r, c) ==
  /\ r \in Requests
  /\ c \in 1..activeCookers
  /\ reqState[r] = "queued"
  /\ writerHealth = "ok"
  /\ reqState' = [reqState EXCEPT ![r] = "cooking"]
  /\ cookOwner' = [cookOwner EXCEPT ![r] = c]
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, ready, reservation,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

FinishCook(r) ==
  /\ r \in Requests
  /\ reqState[r] = "cooking"
  /\ reqState' = [reqState EXCEPT ![r] = "ready"]
  /\ ready' = ready \cup {r}
  /\ cookOwner' = [cookOwner EXCEPT ![r] = 0]
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, reservation,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

CookFail(r) ==
  /\ r \in Requests
  /\ reqState[r] = "cooking"
  /\ reqState' = [reqState EXCEPT ![r] = "failed"]
  /\ failed' = failed \cup {r}
  /\ cookOwner' = [cookOwner EXCEPT ![r] = 0]
  /\ queueEntries' = IF queueEntries > 0 THEN queueEntries - 1 ELSE 0
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, ready, reservation,
      persisted, published, acked, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

Install(r) ==
  /\ r \in Requests
  /\ r \in ready
  /\ reqState[r] = "ready"
  /\ reqLane[r] \in Lanes
  /\ reqTicket[r] = laneNextInstall[reqLane[r]]
  /\ writerHealth = "ok"
  /\ reqState' = [reqState EXCEPT ![r] = "persisting"]
  /\ laneNextInstall' = [laneNextInstall EXCEPT ![reqLane[r]] = laneNextInstall[reqLane[r]] + 1]
  /\ ready' = ready \ {r}
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit,
      queueBytes, queueEntries, cookOwner, reservation,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

PersistOk(r) ==
  /\ r \in Requests
  /\ reqState[r] = "persisting"
  /\ writerHealth = "ok"
  /\ reqState' = [reqState EXCEPT ![r] = "persisted"]
  /\ persisted' = persisted \cup {r}
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready, reservation,
      published, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

PersistFail(r, kind) ==
  /\ r \in Requests
  /\ reqState[r] = "persisting"
  /\ kind \in {"known", "uncertain"}
  /\ IF kind = "known"
     THEN /\ reqState' = [reqState EXCEPT ![r] = "failed"]
          /\ failed' = failed \cup {r}
          /\ uncertain' = uncertain
          /\ writerHealth' = writerHealth
     ELSE /\ reqState' = [reqState EXCEPT ![r] = "uncertain_pending_recovery"]
          /\ uncertain' = uncertain \cup {r}
          /\ failed' = failed
          /\ writerHealth' = "poisoned"
  /\ reservation' = {}
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready,
      persisted, published, acked,
      activeCookers, controllerMode
    >>

Publish(r) ==
  /\ r \in Requests
  /\ reqState[r] = "persisted"
  /\ r \in persisted
  /\ reqState' = [reqState EXCEPT ![r] = "published"]
  /\ published' = published \cup {r}
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready, reservation,
      persisted, acked, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

Complete(r) ==
  /\ r \in Requests
  /\ reqState[r] = "published"
  /\ r \in published
  /\ reqState' = [reqState EXCEPT ![r] = "acknowledged"]
  /\ acked' = acked \cup {r}
  /\ queueEntries' = IF queueEntries > 0 THEN queueEntries - 1 ELSE 0
  /\ UNCHANGED <<
      reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, cookOwner, ready, reservation,
      persisted, published, failed, uncertain,
      activeCookers, controllerMode, writerHealth
    >>

CancelWaiter(r) ==
  /\ r \in Requests
  /\ reqState[r] \notin Terminal
  /\ UNCHANGED vars  \* waiter detach only; mutation continues

ReleaseCredit ==
  /\ queueEntries > 0
  /\ UNCHANGED vars  \* accounting refinement deferred

ActivateCooker ==
  /\ activeCookers < MaxCookers
  /\ writerHealth = "ok"
  /\ activeCookers' = activeCookers + 1
  /\ UNCHANGED <<
      reqState, reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready, reservation,
      persisted, published, acked, failed, uncertain,
      controllerMode, writerHealth
    >>

ParkCooker ==
  /\ activeCookers > 1
  /\ activeCookers' = activeCookers - 1
  /\ UNCHANGED <<
      reqState, reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready, reservation,
      persisted, published, acked, failed, uncertain,
      controllerMode, writerHealth
    >>

BeginDrain ==
  /\ writerHealth = "ok"
  /\ writerHealth' = "draining"
  /\ UNCHANGED <<
      reqState, reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready, reservation,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode
    >>

FinishDrain ==
  /\ writerHealth = "draining"
  /\ queueEntries = 0
  /\ reservation = {}
  /\ writerHealth' = "ok"
  /\ UNCHANGED <<
      reqState, reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready, reservation,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode
    >>

Crash ==
  /\ writerHealth' = "poisoned"
  /\ reservation' = {}
  /\ UNCHANGED <<
      reqState, reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode
    >>

Recover ==
  /\ writerHealth = "poisoned"
  /\ writerHealth' = "ok"
  /\ reservation' = {}
  /\ UNCHANGED <<
      reqState, reqLane, reqTicket, reqDurability, laneNextAdmit, laneNextInstall,
      queueBytes, queueEntries, cookOwner, ready,
      persisted, published, acked, failed, uncertain,
      activeCookers, controllerMode
    >>

----------------------------------------------------------------------------
Next ==
  \/ \E r \in Requests : Reject(r)
  \/ \E r \in Requests, l \in Lanes, d \in {"buffered", "durable"} : Admit(r, l, d)
  \/ \E r \in Requests : FormNatural(r)
  \/ \E r \in Requests, c \in 1..MaxCookers : StartCook(r, c)
  \/ \E r \in Requests : FinishCook(r)
  \/ \E r \in Requests : CookFail(r)
  \/ \E r \in Requests : Install(r)
  \/ \E r \in Requests : PersistOk(r)
  \/ \E r \in Requests, k \in {"known", "uncertain"} : PersistFail(r, k)
  \/ \E r \in Requests : Publish(r)
  \/ \E r \in Requests : Complete(r)
  \/ ActivateCooker
  \/ ParkCooker
  \/ BeginDrain
  \/ FinishDrain
  \/ Crash
  \/ Recover

Spec == Init /\ [][Next]_vars

\* Optional fairness omitted in AWO-0 skeleton.
====
