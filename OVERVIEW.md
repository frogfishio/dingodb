# DingoDB Overview Specification

## 1. Introduction

DingoDB is a resilient, human-inspectable, file-based database designed for systems where **data integrity, recoverability, provenance, and transparency** are primary requirements.

DingoDB deliberately rejects the traditional assumption that a database must be an opaque binary storage engine managed exclusively through APIs.

Instead, DingoDB treats storage as:

> a durable history of structured facts from which current state can be reconstructed.

The database directory itself is a first-class artifact.

A human with standard tools should be able to:

- inspect it;
- understand it;
- recover it;
- repair it;
- migrate it;
- archive it.

DingoDB is designed for:

- AI agent memory systems;
- project knowledge systems;
- configuration management;
- audit systems;
- local-first applications;
- developer tooling;
- structured knowledge repositories.

---

# 2. Design Philosophy

## 2.1 The database is not a black box

Traditional databases optimize for:

- transaction throughput;
- query execution;
- compact storage.

DingoDB optimizes for:

- durability of meaning;
- recoverability;
- transparency;
- long-term ownership.

The primary question is not:

> "Can the database answer this query?"

The primary question is:

> "Can a human still understand and recover this data years later?"

---

## 2.2 History is data

DingoDB treats changes as meaningful information.

The database does not primarily store:

```
current_value
```

It stores:

```
how_current_value_was_created
```

The current state is a projection.

---

## 2.3 Source of truth vs acceleration

DingoDB separates:

### Authoritative storage

The immutable historical record.

### Derived storage

Indexes, caches, search structures.

Derived data may always be:

- deleted;
- regenerated;
- repaired.

The system must never depend on an unrecoverable index.

---

# 3. Storage Model

A DingoDB database is a directory.

Example:

```
project.dingo/

    journal/
        000001.jsonl
        000002.jsonl
        000003.jsonl

    snapshots/
        snapshot-001.json

    indexes/
        facts.idx
        terms.idx

    manifest.json
```

---

# 4. Journal

The journal is the canonical source of truth.

The journal is append-only.

Example:

```json
{
 "id":"fact-001",
 "op":"create",
 "type":"fact",
 "payload":{
    "statement":"ABI is stable",
    "scope":"compiler"
 },
 "time":"2026-07-24T12:00:00Z"
}
```

---

## 4.1 Journal properties

The journal MUST be:

- append-only;
- line-oriented;
- UTF-8 encoded;
- human readable;
- recoverable after partial writes.

JSONL is preferred because:

- each record is independent;
- corruption is localized;
- streaming is possible;
- standard tools work.

---

# 5. Record Model

All stored entities are records.

Generic form:

```json
{
 "id":"unique-id",
 "kind":"fact",
 "version":1,
 "created":"timestamp",
 "updated":"timestamp",
 "body":{}
}
```

---

# 6. Updates

DingoDB does not overwrite records.

Updates create new events.

Example:

Original:

```json
{
"id":"abi",
"status":"active"
}
```

Later:

```json
{
"op":"update",
"id":"abi",
"changes":{
 "status":"superseded"
}
}
```

The current value is reconstructed.

---

# 7. Deletion

Deletion is logical.

A delete creates an event.

Example:

```json
{
"op":"delete",
"id":"fact-42",
"reason":"obsolete"
}
```

The record remains historically available.

---

# 8. Purging

Physical removal is a separate operation.

Purging is performed through:

```
compact()
```

The process:

1. Read complete journal.
2. Reconstruct current state.
3. Remove deleted records.
4. Write new snapshot.
5. Start new journal generation.

Example:

Before:

```
journal/
   0001.jsonl
   0002.jsonl
   0003.jsonl
```

After:

```
snapshot/
   state-001.json

journal/
   0004.jsonl
```

The old database can be retained as an archive.

---

# 9. Recovery Model

DingoDB assumes failure.

Possible failures:

- power loss;
- partial writes;
- corrupted index;
- interrupted compaction;
- manual file editing.

Recovery rules:

## 9.1 Journal recovery

A valid prefix is accepted.

Example:

```
record
record
record
partial JSON...
```

The first three records survive.

The incomplete tail is discarded or repaired.

---

## 9.2 Index recovery

Indexes are disposable.

Recovery:

```
delete index
rebuild index
continue
```

---

# 10. Query System

DingoDB does not require SQL.

The database exposes structured records to query engines.

The preferred query layer is:

# SDA (Structured Data Algebra)

DingoDB stores.

SDA transforms.

Example:

```
facts
|> { f ∈ _
     |
       f<status> = "active"
   }
```

---

# 11. Indexing

DingoDB supports optional indexes.

Indexes are never authoritative.

Possible indexes:

## 11.1 Primary index

Maps:

```
id -> location
```

---

## 11.2 Type index

Maps:

```
kind -> records
```

---

## 11.3 Tag index

Maps:

```
tag -> records
```

---

## 11.4 Semantic index

Optional.

Examples:

- embeddings;
- full text;
- similarity search.

Semantic indexes are hints, not truth.

---

# 12. Project Facts Model

A primary use case is governed project knowledge.

Example:

```json
{
"id":"fact-123",
"kind":"project_fact",

"statement":
"Generated files must never be edited manually",

"scope":
"compiler",

"status":
"active",

"authority":
"principal",

"confidence":
"authoritative",

"evidence":[
 "docs/build.md"
]
}
```

---

## 12.1 Fact lifecycle

Facts move through states:

```
PROPOSED
    |
    v
ACTIVE
    |
    +------+
    |      |
    v      v
SUPERSEDED REJECTED
```

Possible additional state:

```
CONTESTED
```

Meaning:

> conflicting evidence exists.

---

# 13. Security and Tamper Resistance

DingoDB is designed to make tampering detectable.

Possible mechanisms:

## 13.1 Hash chaining

Each journal entry may contain:

```
previous_hash
current_hash
```

Example:

```json
{
"id":42,
"previous_hash":"abc123",
"hash":"def456"
}
```

---

## 13.2 Signed records

Records may optionally contain:

```
signature
authority
```

Allowing:

- trusted facts;
- signed migrations;
- audit trails.

---

# 14. Performance Model

DingoDB favors:

- sequential writes;
- append operations;
- streaming reads.

Writes:

```
O(1)
```

with append.

Queries use:

```
journal
   |
indexes
   |
candidate records
   |
SDA evaluation
```

---

# 15. Why not SQL?

SQL databases are excellent at:

- relational modelling;
- transactional workloads;
- concurrent access.

DingoDB solves a different problem.

DingoDB is optimized for:

- knowledge persistence;
- transparency;
- history;
- recovery;
- local ownership.

---

# 16. Why not MongoDB?

MongoDB provides:

- document storage;
- flexible schemas.

DingoDB differs because:

MongoDB asks:

> "What documents exist now?"

DingoDB asks:

> "What happened, and what should we believe now?"

---

# 17. Intended Applications

## AI systems

Agent memory:

```
conversation
      |
      v
facts
      |
      v
future reasoning
```

---

## Software projects

Store:

- architecture decisions;
- rejected approaches;
- ABI guarantees;
- design constraints.

---

## Developer tooling

Examples:

- build databases;
- compiler metadata;
- dependency knowledge.

---

# 18. Non Goals

DingoDB is not intended to be:

- a replacement for PostgreSQL;
- a distributed database;
- a high frequency trading engine;
- a blob store.

---

# 19. Guiding Principle

DingoDB follows one rule:

> If the machine disappears, the data should still make sense to a human.

A database should preserve not only information, but understanding.

---

This is the kind of document I would put in `docs/OVERVIEW.md` and then split later into:

```
docs/
  OVERVIEW.md
  STORAGE.md
  JOURNAL.md
  RECOVERY.md
  INDEXES.md
  SDA.md
  FACTS.md
  SECURITY.md
```

The interesting thing is: this is actually **not Mongo-like anymore**. The closest mental model is probably **Git + SQLite + event sourcing + knowledge graph**, but with JSONL as the primitive. That is a pretty coherent design space.