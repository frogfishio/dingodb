# DingoDB

## The human-readable, resilient JSON database

DingoDB is an embedded, append-only JSON database designed for systems where **data durability, transparency, and recoverability matter more than hiding complexity behind a storage engine**.

DingoDB stores its source of truth as open, inspectable data files. Every change is recorded. Indexes and materialized views are rebuildable. If something goes wrong, a human can open the database directory, understand what happened, and recover.

No black box. No opaque binary state. No database server required.

```text
application
    |
  DingoDB
    |
+----------------+
| journal.jsonl  |  <- source of truth
| snapshots/     |
| indexes/       |
+----------------+
```

---

## Why DingoDB?

Modern applications increasingly need more than storage.

They need:

- history
- provenance
- auditability
- human recovery
- deterministic reconstruction
- structured knowledge

Traditional databases are excellent at querying current state.

DingoDB is designed for systems that also need to answer:

> "How did we get here?"

---

## Core principles

### Open data

The database is made of normal files.

You can:

- inspect it with standard tools
- back it up
- copy it
- recover it manually
- write your own readers

Your data is not trapped inside a proprietary storage format.

---

### Append-only truth

DingoDB treats history as valuable.

Updates are recorded as events rather than silently overwriting the past.

Example:

```json
{
  "event": "fact.updated",
  "id": "compiler-abi",
  "value": "EXTERN C ABI is pinned",
  "timestamp": "2026-07-24T12:00:00Z"
}
```

The current state is derived from history.

---

### Rebuildable acceleration

Indexes are not the truth.

They can be:

- deleted
- regenerated
- repaired

The database survives damaged acceleration structures.

---

### Human-first recovery

A DingoDB installation should remain understandable without special tooling.

When everything fails:

```bash
cat journal.jsonl
```

should still tell you what happened.

---

## Built for knowledge systems

DingoDB is especially suited for:

- AI agent memory
- project knowledge bases
- configuration systems
- audit trails
- experiment tracking
- build systems
- developer tooling
- local-first applications

It is designed around the idea that structured knowledge is not just data — it has history, authority, and context.

---

## DingoDB + SDA

DingoDB uses **SDA (Structured Data Algebra)** as its transformation layer.

SDA provides a small, deterministic language for:

- filtering
- reshaping
- validation
- normalization
- querying structured data

Example:

```sda
facts
|> { f ∈ _ |
       f⟨status⟩ = "active"
       ∧ f⟨scope⟩ = "compiler"
   }
```

Storage and transformation remain separate:

```
DingoDB
    |
    | records
    v
SDA
    |
    | meaning
    v
application
```

---

## What DingoDB is not

DingoDB is not intended to replace:

- PostgreSQL for relational workloads
- distributed databases for massive clusters
- analytical warehouses
- high-frequency transactional systems

DingoDB optimizes for:

- trust
- inspectability
- resilience
- portability

---

## Design goals

DingoDB aims to be:

✅ embedded  
✅ local-first  
✅ crash resistant  
✅ human inspectable  
✅ deterministic  
✅ rebuildable  
✅ easy to backup  
✅ easy to understand  

---

## Status

DingoDB is currently under active development.

The initial target is a small, reliable core:

- JSONL journal
- snapshots
- indexes
- query engine
- recovery tooling
- SDA integration

---

## License

DingoDB is released under the MIT License.

The DingoDB data format and specifications are intended to remain open and publicly documented.

---

## Philosophy

> A database should not become a mystery box containing your most important information.

DingoDB exists so that software can remember — while humans can still understand.

---

I would probably put a one-line tagline right at the top:

**"DingoDB — the database that remembers how it got there."**

That actually captures the difference.