# Technical proposal: migrate Tinker + Gremlin JSON/JSONL stores onto DingoDB

**Status:** Design proposal (no behavior change)  
**Date:** 2026-07-27  
**Audience:** Frogfish / Koderra (DingoDB, Gremlin daemon, Tinker client)  
**Related:** [OVERVIEW.md](../OVERVIEW.md), [DX_SPEC.md](../DX_SPEC.md), [FORMAT_SPEC.md](../FORMAT_SPEC.md); Gremlin (sibling repo) `Koderra/gremlin/docs/{PROTOCOL,CONTROL_STATE_MIGRATION,DOGFOOD}.md`

---

## 0. Executive summary

Tinker (client UI) and Gremlin (daemon / superagent) already treat **append-only JSON lines** as their durability surface, with **mutable JSON documents** as rebuildable projections. That is almost exactly DingoDB’s model: **streams for events**, **collections for current values with history**.

This proposal recommends a **phased migration** of both products’ on-disk state from ad hoc `.tinker` / `.gremlin` file trees into **one or more DingoDB stores**, without changing the Gremlin wire protocol or Tinker UX on day one. The file layout becomes a **compatibility façade** (and later an export/import profile), not the source of truth.

**Why now**

1. **Dogfood DingoDB** on a real, continuously written workload (journals, indexes, telemetry) while the product is still small enough to migrate offline.
2. **Collapse dual-home storage** (app-data + `~/.gremlin` + `{workspace}/.tinker` + `{workspace}/.gremlin`) that already needs control-state migration for sandbox safety.
3. **Replace hand-rolled integrity** (hash chains, partial-tail salvage, last-wins JSONL indexes) with Dingo’s frame integrity, independent survival, and history APIs—keeping application-level hash chains only where they remain a product invariant.
4. **Unify query surfaces**: session timeline, conversation recents, usage/arena, brains/plans become collection/stream reads instead of bespoke scanners.

**Done when (this doc):** inventory is accurate, target schema is specified, migration phases and cutover are detailed enough to implement.

---

## 1. Inventory of what exists today

### 1.1 Two products, four physical homes

| Surface | Role | Typical path | Format |
|---------|------|--------------|--------|
| **Tinker app data** | Chat-only transcripts, global conversation index, projects, UI prefs | `~/Library/Application Support/com.koderra.tinker/` | JSON + JSONL |
| **Tinker project tree** | Project-bound conversation meta (+ optional transcript body) | `{workspace}/.tinker/conversations/` | JSONL |
| **Gremlin data_dir** | Daemon root: runtime, catalog, chat-only sessions, brains, memory, telemetry | `~/.gremlin/` (`FORMAT` = `1`) | JSON + NDJSON |
| **Gremlin project tree** | Project-bound session journals (canonical when workspace set) | `{workspace}/.gremlin/sessions/<uuid>/` | NDJSON + JSON |

Live sizes on the author’s machine (order of magnitude, 2026-07-27):

| Path | ~Size | Notes |
|------|------:|-------|
| `~/.gremlin` | ~0.4 MB | Many owner/redirect pointers; small journals |
| `…/dingodb/.gremlin` | ~5.6 MB | One long-lived session journal (~6.1k events) |
| `…/gremlin/.gremlin` | ~51 MB | Dogfood repo (largest) |
| `…/dingodb/.tinker` | ~0.1 MB | Index-only (empty transcript file) |
| `…/gremlin/.tinker` | ~2.7 MB | Project chat index + bodies |
| `com.koderra.tinker` | ~2.9 MB | App chats + `legacy-backups/` |

Git already excludes these from source trees (e.g. dingodb `.gitignore`: `.gremlin/`, `.tinker/`; gremlin repo ignores `**/.gremlin/sessions/` and `**/.tinker/conversations/`).

### 1.2 Link graph between Tinker and Gremlin

```text
Tinker ConversationMeta.id  ──daemonSessionId──►  Gremlin SessionId
        │                                              │
        │ projectId / workspace                        │ owner.json.workspace
        ▼                                              ▼
Tinker Project (app projects/index.jsonl)      {workspace}/.gremlin/sessions/<sid>/
        │                                              │
        └──────── redirect in app index ───────────────┘
```

Observed example (this dingodb chat):

- Tinker conversation: `fd3d10bc-ee37-477f-9d14-c71136b77e41`
- Gremlin session: `bc8adb15-0fa0-481c-aed3-2148993d8650`
- Project: `6bac2eef-815d-419e-aa90-d3abe48ce4d6` → workspace `…/dingodb`
- App index holds a **redirect** stub; full meta lives in `{workspace}/.tinker/conversations/index.jsonl`
- Daemon catalog/owner under `~/.gremlin` point at the workspace; **journal bytes** live under the project

### 1.3 Tinker data model (from `tinker/src-tauri/src/conversations.rs`)

#### Layout

```text
# App data (chat-only + global discovery)
com.koderra.tinker/
  tinker.json                         # UI/daemon prefs (not conversation state)
  projects/index.jsonl                # Project records (last-wins by id)
  conversations/
    index.jsonl                       # ConversationMeta OR ConversationRedirect
    <conversationId>.jsonl            # TranscriptLine rows (append)
    legacy-backups/…                  # Pre-migration snapshots

# Project-bound
{workspace}/.tinker/conversations/
  index.jsonl                         # full ConversationMeta for this project
  <conversationId>.jsonl              # transcripts (may be empty if UI is journal-backed)
```

#### `ConversationMeta` (v1, camelCase JSONL, **append last-wins**)

| Field | Purpose |
|-------|---------|
| `v`, `id`, `title`, `createdAt`, `updatedAt` | Identity + recents ordering |
| `daemonSessionId`, `backend` | Link to Gremlin session |
| `selectedProvider` / `selectedModel` / `smart` / `enginePref` / `featureOverrides` | Routing pins |
| `preview` | Sidebar snippet |
| `projectId`, `workspace` | Project bind (denormalized path) |
| `colorTag`, `archived`, `deleted` | UI housekeeping |

#### `ConversationRedirect`

```json
{
  "kind": "redirect",
  "id": "<conversationId>",
  "workspace": "/abs/path",
  "projectId": "<optional>",
  "updatedAt": "…"
}
```

Global index keeps redirects so Recents can discover project chats without scanning every workspace.

#### `TranscriptLine` (v1)

User/assistant text plus optional `journalSeq`, `turnId`, route metadata, thumbs feedback, and a `timeline` of thought/tool rows. Project-bound chats often leave `<id>.jsonl` empty and rely on the daemon journal for history (observed: dingodb project transcript file is 0 bytes while the session journal is multi‑MB).

#### `Project` (app `projects/index.jsonl`)

`{ v, id, name, workspace, createdAt, updatedAt }` — last-wins JSONL catalog.

#### Write semantics

- Indexes: **append a full replacement record** per update; readers fold last-wins by `id`.
- Transcripts: **append** `TranscriptLine`.
- Atomic rewrite used for some derived paths (`atomic_write_bytes`: temp + rename + fsync).
- Compaction of index JSONL is opportunistic / not a strong product feature today (dingodb project index: **247 lines for 1 conversation** — pure update churn).

### 1.4 Gremlin data model (from PROTOCOL + `journal_store.rs`)

#### Layout (`data_dir` = `~/.gremlin` by default)

```text
~/.gremlin/
  FORMAT                              # single byte/text "1"
  runtime.json                        # daemon policy, engines, pricing, gates, context
  secrets-index.json                  # non-secret index of providers (values in Keychain)
  sessions/
    catalog.ndjson                    # SessionCatalogEntry rows (rebuildable)
    <uuid>.json                       # legacy full snapshot OR redirect {kind:redirect, workspace}
    <uuid>/
      owner.json                      # { v, sessionId, workspace?, ts }
      journal.ndjson                  # append-only SoT (chat-only sessions)
      manifest.json                   # derived { v, sessionId, nextSeq, headHash }
      snapshot.json                   # derived projection / restart aid
  brains/
    sessions/<uuid>.json              # plan + plan_history + counters
    playbooks/…                       # reusable plan seeds
  memory/
    project.json                      # array of sticky notes (global sample)
    sessions/  tasks/                 # reserved / sparse
  telemetry/
    usage.jsonl                       # per-call token/latency rows
    arena.json                        # learned band/labor cells
  tasks/                              # optional TaskRecord files
  package-worktrees/                  # optional (SUBCONTRACT)
  locks/                              # runtime locks

{workspace}/.gremlin/sessions/<uuid>/ # project-bound: journal + manifest + snapshot
  (owner/catalog still under data_dir)
```

**Placement rule (today):** if `SessionCreate.workspace` is set, the **canonical journal** is under the project tree; `data_dir` keeps catalog + `owner.json` so restart can list/load. That is exactly what CONTROL_STATE_MIGRATION wants to invert for sandbox safety (canonical → `data_dir`, project path → pointer only).

#### Journal record (v2, camelCase NDJSON)

```text
{
  v, sessionId, seq, ts, kind, payload,
  turnId?, prevHash?, hash,
  batchId?, batchIndex?, batchLen?
}
```

| Property | Observed / specified |
|----------|----------------------|
| Integrity | SHA-256 hash chain via `prevHash`/`hash`; gap-free `seq` |
| Salvage | Torn multi-record batch / partial tail dropped before next append |
| Hot path | Hydrate once → in-memory head + message offset index; append from head |
| Fsync | `every_batch` (default) or `terminal_only` for mid-turn batches |
| Derived | `manifest.json`, `snapshot.json`, catalog rows rebuild from journal |

**Kind distribution** (dingodb session, n≈6155): ~88% `tool_outcome`, then metadata, package lifecycle, turn lifecycle, `assistant_message`, rare `plan_checkpoint`. Assistant messages dominate **bytes** (avg ~23 KB/line vs tool ~0.7 KB).

**Payload sketches (high signal)**

| Kind | Payload shape |
|------|----------------|
| `session_created` | `backendId`, `label`, `workspace`, … |
| `turn_accepted` | `userMessage`, `request` |
| `route_selected` | `engine`, `model`, `provider`, `tier`, `reason`, `situation_key` |
| `work_package_awarded` / `finished` | `package_id`, `labor_source`, `goal` / `summary`, … |
| `tool_outcome` | `call_id`, `activity`, `result`, `type` |
| `assistant_message` | `text`, `metadata` (route, timeline, labor, why) |
| `session_metadata_updated` | often `conversationId` + `workspace` (+ preview/title/backend) |
| `plan_checkpoint` | plan snapshot or step advance |
| terminals | `turn_completed` (empty), `turn_failed` (`message`) |

#### Other Gremlin documents

| File | Semantics |
|------|-----------|
| `runtime.json` | Single mutable config document (schema versioned) |
| `brains/sessions/<id>.json` | Current plan + archived plan_history |
| `memory/project.json` | Array of `{id, scope, key, content, timestamps, source}` |
| `telemetry/usage.jsonl` | Append-only metrics |
| `telemetry/arena.json` | Whole-document rewrite of learned cells |
| `secrets-index.json` | **Must stay out of any guest-visible tree**; secrets remain Keychain |

### 1.5 Pain points of the current organization

1. **Split brain.** Canonical session bytes may live in the project tree while Tinker meta lives in `.tinker` and app data; redirects and owner pointers paper over it. Sandbox materialization must exclude `.gremlin/**` carefully (CONTROL_STATE_MIGRATION).
2. **JSONL last-wins indexes bloat.** Meta updates rewrite full rows; 247 lines for one conversation is pure churn and slows cold start fold.
3. **Duplicate history.** Transcript JSONL vs journal NDJSON vs live ring buffer—three projections of “what the user saw,” not always filled the same way.
4. **Hand-rolled durability.** Hash chain + salvage is good, but reimplemented per product; no shared compaction, tiering, or independent-survival scan.
5. **Cross-session / cross-project analytics** (usage, arena, “all my dingodb chats”) require ad hoc filesystem walks.
6. **No typed import/export** beyond “copy the folder”; backup/restore is rsync folklore.
7. **Scale cliff.** One dogfood journal is already multi‑MB with 5k+ tool events; multi-year retention wants segments, indexes, and tiering (Dingo’s job).

---

## 2. Goals and non-goals

### 2.1 Goals

| ID | Goal |
|----|------|
| G1 | Single **authoritative** store backend for Gremlin daemon session continuity (journals + derived projections). |
| G2 | Single **authoritative** store backend for Tinker conversation/project catalogs and optional transcript cache. |
| G3 | Preserve **wire protocol** and client UX; storage is an implementation detail behind existing RPCs / Tauri commands. |
| G4 | Preserve journal **seq + hash chain** as an application invariant (product continuity / audit), even when physical integrity is provided by Dingo frames. |
| G5 | Align with control-state policy: **no trusted journals required under `{workspace}`** for sandbox guests; workspace may keep only opaque redirect markers if needed. |
| G6 | Online dual-read migration with **verify-before-delete**; crash-safe; reversible for one release. |
| G7 | Dogfood DingoDB streams/collections/history APIs on a real product path. |

### 2.2 Non-goals (v1 migration)

- Replacing Gremlin’s live event ring, stream-join, or labor engines.
- Storing secrets or provider tokens in Dingo (Keychain + `secrets-index` only).
- Multi-tenant server deployment of personal chat history (local embedded store first).
- Perfect schema freeze of every journal `kind` payload (store as JSON values; evolve app schema independently).
- Migrating Grok Build / Cursor CLI home dirs (`~/.grok`, cursor-agent caches).

---

## 3. Why DingoDB (reasons)

### 3.1 Semantic fit

| Current pattern | Dingo primitive | Why |
|-----------------|-----------------|-----|
| `journal.ndjson` append | **Stream** `gremlin.sessions.<sid>.journal` (or partitioned `gremlin.journal` with session key) | Append-oriented events; cursors; retention |
| `ConversationMeta` / catalog last-wins | **Collection** `tinker.conversations` / `gremlin.sessions` | `put` = current value; **history** retains prior metas without JSONL bloat |
| `TranscriptLine` append | Stream **or** history of conversation doc | Prefer stream if high volume; or derive from journal only |
| `runtime.json`, `arena.json`, brain docs | Collections | Document put + version receipts |
| `usage.jsonl` | Stream `gremlin.telemetry.usage` | Append metrics |
| `manifest` / `snapshot` | Derived collections or in-process only | Rebuild from stream; optional materialize for restart speed |
| Cross-cutting “list my sessions” | Secondary indexes / examination | Avoid catalog.ndjson special case |

Dingo’s governing rule—**what remains still lives**—matches journal salvage better than a single fragile file: a torn segment tail is a **hole**, not a reason to discard earlier islands.

### 3.2 Product and engineering reasons

1. **Dogfood loop.** Building Dingo while daily work generates continuous structured event traffic is the highest-signal test harness you can buy.
2. **One durability story.** Atomic meta, crash matrix, compaction, tiering, and recovery become shared infrastructure instead of three Rust modules.
3. **History without bloat.** Conversation title/preview updates become versioned document history, not 247 index lines.
4. **Examination.** SDA over session journals (“find all `turn_failed` with labor timeout”) is a first-class Dingo story, not a one-off Python script.
5. **Sandbox alignment.** Control store lives outside the workspace by construction (`~/.dingo/gremlin` or daemon data dir), completing CONTROL_STATE_MIGRATION with a stronger store than “move the NDJSON files.”
6. **Future multi-device / backup.** Export is a Dingo store copy or segment pack, not a nest of redirect stubs.

### 3.3 Why not “just keep JSONL forever”

JSONL is an excellent **interchange and debug** format (and Dingo already names JSONL as import/export/diagnostic). It is a weak **primary** store once you need concurrent writers, compaction, indexes, tiering, and damage isolation. Keep JSONL as:

- migration source,
- `dingo export --profile jsonl`,
- operator forensics.

---

## 4. Target architecture

### 4.1 Store topology

```text
~/.dingo/                                    # or GREMLIN_DATA_DIR / TINKER_DATA_DIR override
  gremlin.dingo                              # daemon control + sessions + telemetry + brains
  tinker.dingo                               # optional: client-only UI state
  # Alternative v1: single personal.dingo with namespaced collection prefixes
```

**Recommendation:** **one store per process trust domain** for v1:

| Process | Store | Rationale |
|---------|-------|-----------|
| `gremlin-daemon` | `gremlin.dingo` under `data_dir` | Principal control plane; journals; arena; runtime |
| Tinker (Tauri) | `tinker.dingo` under app data **or** RPC-only into daemon | Avoid two writers to the same session journal |

**Preferred end state:** Tinker does **not** own session history. Tinker stores only UI prefs + project list + thin conversation **cards** (title, preview, pins) and always loads message history via daemon `session_messages`. That deletes the dual transcript problem. Migration can still import existing transcript files as a one-shot cache seed.

### 4.2 Logical schema (collections & streams)

Namespace prefix convention: `tinker.*` / `gremlin.*` (stable string keys).

#### Collections (current value + history)

| Collection | Key | Value (JSON) | Notes |
|------------|-----|--------------|-------|
| `gremlin.meta` | `format` | `{ store_format: 1, migrated_from: "fs-v1", … }` | Store marker (replaces `FORMAT` file) |
| `gremlin.runtime` | `default` | full `runtime.json` body | Single doc |
| `gremlin.sessions` | `<sessionId>` | `{ workspace?, label, backend_id, status, conversation_id?, created_at, updated_at, provider_continuation?, … }` | Catalog row; rebuildable from journal but kept hot |
| `gremlin.session_manifest` | `<sessionId>` | `{ next_seq, head_hash }` | Derived head |
| `gremlin.brains` | `<sessionId>` | plan + history | Today’s brain JSON |
| `gremlin.memory.project` | `<projectKey>` | note document | Prefer key=`workspace_hash/key` over giant array |
| `gremlin.arena` | `default` | arena document | Or cell-per-key if large |
| `gremlin.secrets_index` | `default` | providers list only | No secret material |
| `tinker.projects` | `<projectId>` | project record | |
| `tinker.conversations` | `<conversationId>` | ConversationMeta **or** redirect fields | No more index fold |
| `tinker.prefs` | `default` | `tinker.json` | |

#### Streams (append-only)

| Stream | Event id / order | Payload | Notes |
|--------|------------------|---------|-------|
| `gremlin.journal.<sessionId>` **or** partition key `sessionId` on `gremlin.journal` | caller-supplied `seq` (u64) as stable id | full journal record JSON (incl. hash fields) | **SoT for session continuity** |
| `gremlin.usage` | generated | usage row | optional secondary |
| `tinker.transcript.<conversationId>` | generated or `ts+role` | TranscriptLine | **Optional**; prefer derive from journal |

**Partitioning:** one physical stream per session scales file handles; one global stream with `sessionId` partition key scales catalog simplicity. Recommend **per-session stream** for isolation and salvage locality (matches today’s one-file-per-session).

### 4.3 Integrity layering

```text
┌─────────────────────────────────────────────┐
│ Application: seq monotonic + prevHash/hash  │  ← product audit / reconnect
├─────────────────────────────────────────────┤
│ DingoDB: frame CRC/BLAKE3, segments, holes  │  ← media survival
└─────────────────────────────────────────────┘
```

- On append: daemon still computes journal `hash` over the record bytes it considers canonical (define a **canonicalization profile** once—same as today’s serializer).
- Dingo acknowledges durable put/stream append under the daemon’s durability mode.
- On open: verify Dingo item health, then verify application hash chain over the stream projection; surface both in `session_journal_health`.

Do **not** drop application hashes in v1: clients and operators already reason about them; dual integrity is cheap relative to trust.

### 4.4 Façade layout (compatibility)

After cutover, optional thin markers (not trusted data):

```text
{workspace}/.gremlin/sessions/<uuid>.redirect.json
  → { "v": 1, "kind": "dingo", "store": "gremlin", "sessionId": "…" }

{workspace}/.tinker/conversations/README or marker
  → "canonical store: tinker.dingo / daemon"

~/.gremlin/FORMAT → "dingo" or remain "1" with runtime feature flag
```

Dual-read order during migration:

1. Dingo session stream if present and healthy  
2. Else project `journal.ndjson`  
3. Else `data_dir` journal  

Write path after flag flip: **Dingo only** (with optional async JSONL export for debug).

### 4.5 Process ownership (writers)

| Data | Writer | Readers |
|------|--------|---------|
| Journal stream | **daemon only** | daemon, Tinker via RPC |
| Session catalog / brains / arena / usage | **daemon only** | daemon, Tinker via RPC |
| Conversation cards | Tinker **or** daemon (prefer daemon updates via `session_metadata_updated`) | Tinker |
| Projects / prefs | Tinker | Tinker |
| Runtime | daemon | daemon, Tinker settings RPC |

Eliminating Tinker writers on journals is a hard requirement for embedded Dingo single-writer simplicity.

---

## 5. Mapping: old → new (detailed)

### 5.1 Gremlin journal

| Source | Target |
|--------|--------|
| `{ws}/.gremlin/sessions/<sid>/journal.ndjson` line N | stream append event with id = `seq`, value = parsed JSON object |
| `manifest.json` | `gremlin.session_manifest[<sid>]` put after import |
| `snapshot.json` | optional put `gremlin.session_snapshot[<sid>]` or discard and rebuild |
| `owner.json` | merge into `gremlin.sessions[<sid>].workspace` + timestamps |
| `data_dir/sessions/catalog.ndjson` | rebuild from `gremlin.sessions` collection scan |
| `data_dir/sessions/<sid>.json` redirect | sessions collection field only |
| legacy full snapshot files | import `events` if any; prefer journal if both exist |

**Import algorithm (per session)**

1. Locate winning journal path (project if journal exists and owner workspace matches; else data_dir).  
2. Stream-read NDJSON; skip torn tail (reuse existing salvage logic).  
3. Verify hash chain; record `JournalHealth`.  
4. If invalid prefix: import verified prefix only; mark session `imported_partial=true`.  
5. Batch stream-append into Dingo with durability = same policy as `every_batch` for import (or bulk load API if available).  
6. Put manifest + sessions catalog row.  
7. Write migration receipt: `{ sessionId, source_path, source_bytes, records, head_hash, dingo_receipts… }`.  
8. Only after verify job: mark source `migrated` and stop dual-write.

### 5.2 Tinker conversations

| Source | Target |
|--------|--------|
| App + project `index.jsonl` | fold last-wins → `tinker.conversations.put(id, meta)` |
| Redirects | store as meta with `kind: "redirect"` **or** resolve and store full meta only in project scope |
| `<id>.jsonl` transcripts | optional stream import; attach `imported_from: "tinker_transcript"` |
| `projects/index.jsonl` | `tinker.projects` |
| `legacy-backups/**` | **do not auto-import** (archive; operator may opt in) |
| `tinker.json` | `tinker.prefs` |

**Conflict rule:** if both app redirect and project full meta exist, **project full meta wins** for fields; redirect’s `updatedAt` only used for recents ordering if newer.

### 5.3 Brains, memory, telemetry, runtime

| Source | Target |
|--------|--------|
| `brains/sessions/*.json` | `gremlin.brains` |
| `memory/project.json` array | explode to `gremlin.memory.project` keys; keep `id` |
| `telemetry/usage.jsonl` | stream `gremlin.usage` |
| `telemetry/arena.json` | `gremlin.arena` |
| `runtime.json` | `gremlin.runtime` |
| `secrets-index.json` | `gremlin.secrets_index` (still no secret values) |

### 5.4 Identity stability

All UUIDs (**sessionId**, **conversationId**, **projectId**, **package_id**, **turnId**) remain the same strings. Migration is storage plumbing, not renumbering. `provider_continuation` stays on the session catalog document.

---

## 6. Migration procedure (how)

### Phase 0 — Prerequisites (Dingo + Gremlin)

1. **Embedded open path** stable: `Dingo::open(path)` local store used by daemon tests.  
2. **Stream append + cursor read** API usable from Rust (`dingo-sdk`).  
3. **Collection put/get/history** usable.  
4. **Durability modes** mapped: journal `every_batch` → Dingo durability that fsyncs segment; `terminal_only` → group mid-turn stream appends + fsync on terminal kinds.  
5. Feature flags:  
   - `GREMLIN_STORE=fs|dingo|dual`  
   - `TINKER_STORE=fs|dingo|dual`  
6. Migration tool binary: `gremlin-migrate` (or `dingo-tool import-gremlin`) with `--dry-run`, `--verify`, `--commit`.

**Exit criteria:** synthetic journal of 10k events round-trips with identical hash head.

### Phase 1 — Offline importer (no cutover)

1. Implement FS inventory walker (all of §1 homes).  
2. Import into a **side** store path `~/.dingo/gremlin.migrate-tmp.dingo`.  
3. Verification suite:  
   - record count per session  
   - first/last seq, head hash  
   - sample kind histogram  
   - conversation meta equality after fold  
4. Emit human report (JSON + markdown).  
5. Keep FS as sole live writer.

**Exit criteria:** dry-run on dogfood machine green; report reviewed.

### Phase 2 — Dual-read / dual-write daemon

1. On session open: prefer Dingo stream if session marked migrated; else FS.  
2. On append: if `dual`, write **FS first** (legacy SoT), then Dingo; if Dingo fails, log and continue (degraded).  
3. Background backfill worker migrates cold sessions.  
4. `session_journal_health` includes both FS and Dingo counters during dual.

**Exit criteria:** 7 days dogfood; no hash mismatch; p99 append latency acceptable.

### Phase 3 — Dingo becomes SoT for Gremlin

1. Flip `GREMLIN_STORE=dingo`.  
2. Append path: Dingo only; optional debug JSONL mirror off by default.  
3. FS journals frozen; marker file `MIGRATED` in old session dirs.  
4. Catalog/brains/runtime/arena/usage read-write via collections.  
5. CONTROL_STATE_MIGRATION: stop creating project-tree journals; owner redirect only if needed for old tools.

**Exit criteria:** restart recovery from Dingo only; FS journals unused for 14 days.

### Phase 4 — Tinker cutover

1. ConversationStore backend trait: `FsConversationStore` | `DingoConversationStore` | `DaemonBackedConversationStore`.  
2. Prefer **daemon-backed** cards: metadata updates already flow as `session_metadata_updated` with `conversationId`.  
3. Import remaining app-data chats.  
4. Stop writing project `.tinker/conversations/index.jsonl` except optional mirror.  
5. Transcript UI always pages via `session_messages` when `daemonSessionId` set; local transcript stream only for pre-daemon legacy chats.

**Exit criteria:** delete empty project transcript files; Recents works fully from Dingo/daemon.

### Phase 5 — Cleanup

1. Tool: `gremlin-migrate gc-fs --older-than 30d` moves FS trees to `~/Library/Archives/gremlin-fs-…` (not delete by default).  
2. Shrink dual-read codepaths.  
3. Document operator recovery: `dingo salvage`, export JSONL profile.  
4. Update PROTOCOL.md durable storage section; mark CONTROL_STATE_MIGRATION complete via Dingo.

---

## 7. Concrete import/verify algorithms

### 7.1 Journal verify (must match daemon)

```text
function import_journal(path, session_id):
  records, health = salvage_and_parse(path)   # existing journal_store logic
  assert health.error is None or allow_partial
  prev_hash = None
  for r in records:
    assert r.sessionId == session_id
    assert r.seq == expected_seq
    assert verify_hash(r, prev_hash)
    prev_hash = r.hash
    expected_seq += 1
  stream = dingo.stream("gremlin.journal." + session_id)
  for r in records:
    stream.append(id=str(r.seq), value=r, metadata={ kind: r.kind, turnId: r.turnId })
  dingo.collection("gremlin.session_manifest").put(session_id, {
    next_seq: expected_seq,
    head_hash: prev_hash
  })
  return health
```

### 7.2 Conversation index fold

```text
function fold_index(jsonl_path) -> Map[id, meta]:
  map = {}
  for line in jsonl_path:
    obj = parse(line)
    id = obj.id
    map[id] = obj   # last wins
  return map
```

Merge app + project maps with project full meta winning over redirect.

### 7.3 Idempotency

- Migration receipt collection `gremlin.migration_receipts` keyed by `sha256(source_path + size + mtime)`.  
- Re-run skips completed receipts unless `--force`.  
- Stream append with caller ids = `seq` makes re-import naturally idempotent if the SDK rejects duplicate ids or if importer checks `manifest.next_seq` first.

### 7.4 Failure modes

| Failure | Response |
|---------|----------|
| Hash break mid-file | Import verified prefix; flag session; do not delete FS |
| Dingo full disk | Abort commit; keep FS SoT |
| Concurrent daemon write during import | Require daemon stop **or** flock session dir; online dual-write only after Phase 2 |
| Partial dual-write (FS ok, Dingo fail) | Session stays FS-canonical; retry backfill |
| User edits FS by hand during dual | Detect mtime/size vs receipt; re-verify |

---

## 8. API / code change surface (implementation sketch)

### 8.1 Gremlin daemon

```text
trait SessionStore {
  fn append_batch(...);
  fn hydrate(session_id) -> JournalProjection;
  fn health(session_id) -> JournalHealth;
  fn list_sessions() -> Vec<SessionCatalogEntry>;
}

struct FsSessionStore { ... }       // today’s journal_store.rs
struct DingoSessionStore { dingo: Dingo }
struct DualSessionStore { fs, dingo, mode }
```

- `SessionJournalStore::session_dir` gains a placement policy: `ControlHome` vs `Workspace` (CONTROL_STATE already designed).  
- Replace direct `OpenOptions::append` with `SessionStore::append_batch`.  
- Snapshot/manifest writers become collection puts.

### 8.2 Tinker

```text
trait ConversationRepository {
  fn list_summaries(...) -> Vec<ConversationSummary>;
  fn get_meta(id) -> ConversationMeta;
  fn upsert_meta(meta);
  fn append_message(id, line);  // deprecated for daemon-linked chats
  fn page_messages(...);        // FS or RPC
}
```

Tauri commands stay stable; repository swaps underneath.

### 8.3 Dependency

- Daemon: `dingo-sdk` / `dingo-store` path dependency or crates.io once published.  
- Feature `store-dingo` to keep default builds FS-only until ready.

---

## 9. Relationship to existing CONTROL_STATE_MIGRATION

Gremlin `docs/CONTROL_STATE_MIGRATION.md` moves journals from `{workspace}/.gremlin` to `~/.gremlin` for sandbox exclusion. **This proposal subsumes that move:**

| CONTROL_STATE step | Dingo migration equivalent |
|--------------------|----------------------------|
| Copy journal to data_dir | Import stream into `gremlin.dingo` under data_dir |
| Owner pointer in project | Optional `.redirect` marker only |
| Dual-read FS paths | Dual-read FS vs Dingo |
| Drop project trusted writes | Never write journals into workspace |

Do **not** do two sequential full migrations (FS→FS home, then FS→Dingo) on dogfood data if avoidable: **import project journals straight into Dingo** while writing control-home markers. One cutover, two wins (containment + store).

---

## 10. Security, privacy, sandbox

1. **Store path** always under daemon `data_dir` / app data — never inside a project sandbox view.  
2. Materializer exclusion list gains `**/*.dingo`, `**/.dingo/**` if any workspace-adjacent debug copies exist.  
3. Secrets: unchanged (Keychain); refuse to import any historical file that looks like raw API keys into collections.  
4. Telemetry/journals may contain source code snippets and paths — backup encryption is an operator concern (future: encrypted segment tier).  
5. Multi-user machine: file modes `0700` on store directory (same as `~/.gremlin` expectation).

---

## 11. Performance notes

| Workload | FS today | Dingo expectation | Mitigation |
|----------|----------|-------------------|------------|
| Mid-turn tool_outcome flood | append + optional fsync | stream append | `terminal_only` grouping; batch append API |
| Cold hydrate 50 MB journal | linear scan + hash | stream scan + index | keep message offset index in memory; optional snapshot collection |
| Recents list | fold multi-100-line JSONL | collection scan / index on `updated_at` | secondary index when available |
| Arena whole-doc rewrite | rewrite JSON file | put with history | cell-per-key if document grows |

Budget: dual-write Phase 2 should stay within +10–20% turn latency; if not, dual-write only terminal kinds to Dingo until bulk backfill catches up.

---

## 12. Testing plan

1. **Unit:** hash chain parity, torn-tail salvage parity, index fold golden files.  
2. **Import fixtures:** copy anonymized micro journals into `dingo-store` tests.  
3. **Crash matrix:** kill -9 mid dual-write; ensure FS or Dingo recoverable and no silent fork (receipt decides canonical).  
4. **Dogfood:** migrate gremlin repo sessions (~50 MB) on a branch daemon; run one day of real work.  
5. **Tinker Recents:** redirect + project meta + archived/deleted flags.  
6. **Protocol:** existing `persist_restart` tests retargeted at `DingoSessionStore`.

---

## 13. Rollout flags and operator UX

```text
gremlin start --store dingo --data-dir ~/.gremlin
gremlin migrate inventory
gremlin migrate import --dry-run
gremlin migrate import --commit
gremlin migrate verify --session <uuid>
gremlin migrate set-canonical dingo
```

Tinker: Settings → Advanced → “Conversation storage: File / Dingo / Daemon” (default Daemon when daemon ≥ version X).

---

## 14. Risks and open decisions

| Risk / decision | Options | Recommendation |
|-----------------|---------|----------------|
| One store vs two | `personal.dingo` vs split tinker/gremlin | Split by process; merge later if needed |
| Per-session stream vs global | isolation vs simplicity | Per-session stream |
| Keep app hash chain | yes / rely on Dingo only | **Keep** in v1 |
| Transcript duplication | import / ignore / derive | Derive from journal; import legacy only |
| Online vs offline import | daemon stopped / flock | Offline for Phase 1; dual for Phase 2 |
| Windows/Linux app data paths | only macOS Application Support observed | Use existing Tinker app-data helper |
| crates.io coupling | path dep vs versioned | Path/workspace until dingo 0.1 publish |

---

## 15. Success metrics

1. **Correctness:** 100% of sessions with healthy FS journals import with identical `head_hash` and `next_seq`.  
2. **Containment:** zero new trusted journal bytes under `{workspace}/.gremlin/sessions/**` after Phase 3.  
3. **UX:** Tinker Recents + reconnect + stream-join unchanged.  
4. **Ops:** single-command backup = copy `gremlin.dingo` (+ WAL if any).  
5. **Dogfood:** ≥1 week of principal labor traffic on Dingo SoT without FS fallback.

---

## 16. Suggested implementation order (PR-sized)

| PR | Scope |
|----|--------|
| P0 | `SessionStore` trait + FS adapter (behavior-neutral refactor) |
| P1 | `DingoSessionStore` append/hydrate for journal only |
| P2 | `gremlin migrate` inventory + import + verify CLI |
| P3 | Dual mode + backfill worker |
| P4 | Catalog/brains/runtime/arena/usage collections |
| P5 | Stop workspace journal placement (CONTROL_STATE complete) |
| P6 | Tinker repository trait + daemon-backed metas |
| P7 | GC/archive FS trees + docs |

---

## 17. Appendix A — Observed live linkage (dingodb dogfood)

```text
Tinker conversation  fd3d10bc-ee37-477f-9d14-c71136b77e41
  projectId          6bac2eef-815d-419e-aa90-d3abe48ce4d6  (dingodb)
  workspace          /Users/…/Code/Frogfish/dingodb
  daemonSessionId    bc8adb15-0fa0-481c-aed3-2148993d8650
  project index      .tinker/conversations/index.jsonl  (many last-wins rows)
  project transcript .tinker/conversations/<id>.jsonl   (empty)
  app index          redirect → workspace

Gremlin session      bc8adb15-0fa0-481c-aed3-2148993d8650
  journal            .gremlin/sessions/<sid>/journal.ndjson  (~5.6 MB, seq 0..6157)
  manifest           nextSeq ≈ 6155–6158, headHash present
  home owner         ~/.gremlin/sessions/<sid>/owner.json → workspace
  brain              ~/.gremlin/brains/sessions/<sid>.json
```

This single session is a complete **golden fixture** for the importer.

## 18. Appendix B — Why “migrate both” together

Migrating only Gremlin leaves Tinker’s last-wins indexes and redirect graph as a second brittle system that still points at workspace paths. Migrating only Tinker leaves the multi‑MB principal journal on FS. The products are already **identity-linked** (`daemonSessionId` / `conversationId` in journal metadata). One migration program with two store namespaces avoids a year of half-migrated state.

---

## 19. Conclusion

Tinker and Gremlin already speak Dingo’s language: **append events, project current state, salvage damage, ignore catalogs when needed**. The file trees under `.tinker` and `.gremlin` were the right MVP; they are the wrong long-term primary store for a durability-centric stack that is itself shipping DingoDB.

**Migrate journals and catalogs into Dingo collections/streams under the daemon data dir; keep JSONL as import/export; finish control-state containment in the same stroke; let Tinker become a pure client of session history.**

That is the smallest migration that pays for itself in dogfood signal, operational simplicity, and architectural honesty.
