# WEB rebrand inventory (Phase 4)

**Date:** 2026-07-31  
**Feature:** WEB — Residiuum websites (Phase 4)  
**Task:** WEB-0 — Inventory + constraints freeze  
**Scope:** `web/residiuumdb.org`, `web/docs.residiuumdb.org` (exclude `node_modules/`, `dist/`)

SoT: `REBRAND.md` Phase 4, Feature WEB objective, `doc/REBRAND_CLASS_C_FREEZE.md`.

---

## 1. Directory state

| Path | Status |
|---|---|
| `web/residiuumdb.org` | **Present** — main marketing Astro site |
| `web/docs.residiuumdb.org` | **Present** — docs Astro site |
| `web/dingodb.org` | **Absent** — no leftover old dir |
| `web/docs.dingodb.org` | **Absent** — no leftover old dir |

No `git mv` of site directories is required for Phase 4.

---

## 2. Hard constraints (freeze)

### 2.1 Hosting project_id (immutable)

| Site | Path | Binding |
|---|---|---|
| Main | `web/residiuumdb.org/.openai/hosting.json` | `project_id`: **`appgprj_6a6a4bd6baf08191949a0106278a04d8`** |
| Docs | `web/docs.residiuumdb.org/.openai/` | **Missing** — no hosting binding file |

**Rule:** Never rewrite the main site `hosting.json` `project_id`. WEB-1…WEB-7 must leave it byte-identical unless the principal explicitly issues a new binding.

**Docs hosting:** No in-repo OpenAI Sites binding for docs yet. Principal may add a separate `.openai/hosting.json` later; out of WEB scope unless authorized. Do not invent a project_id.

### 2.2 Class C (retain_legacy on web)

Do **not** rewrite these as “Residiuum” wire names. Product prose may say Residiuum while values stay frozen:

| Kind | Examples observed under `web/` |
|---|---|
| Profile IDs | `dingo-backup-v1`, `dingo-scrub-v1`, `dingo-migrate-v1`, `dingo-config-v1`, `dingo-log-v1`, `dingo-rpc-v1` |
| Files with Class C literals | `web/*/src/data/capabilities.json`; docs `reference/compatibility.md`, `wire.md`, `configuration.md`, `status/compatibility.md`, ops guides (`backup-restore`, `logs`, `migration`, …) |

Full wire freeze remains in `doc/REBRAND_CLASS_C_FREEZE.md` (crates). Web only **documents** those IDs.

### 2.3 Class D / out of Feature

| Item | Disposition |
|---|---|
| GitHub remote `github.com/frogfishio/dingodb` | **Leave** unless principal authorizes rename |
| Links to that remote in release.json / content | **Leave URL path** (Class D); crate paths inside repo should already be `residiuum-*` |
| Live DNS/CDN cutover for `dingodb.org` | Principal ops (WEB-6 documents only) |

---

## 3. Naming rule (normative for WEB-1…7)

| Surface | Correct | Incorrect (fix) |
|---|---|---|
| Product name | **Residiuum** | Residuum, ResiduumDB, DingoDB, product+DB |
| Domain (hosts only) | **residiuumdb.org**, **docs.residiuumdb.org** | dingodb.org, docs.dingodb.org, residuumdb.org (one-i intermediate) |
| Query language | **RQL** | DQL (public paths/prose) |
| Rule expression | **RRE** | DRE (public paths/prose) |
| Public API samples | `residiuum_sdk`, `Residiuum::open`, `residiuum://` | `dingo_sdk`, `Dingo::open`, `dingo://` |

---

## 4. Residual brand hit summary (2026-07-31 scan)

Approximate **file counts** (excluding `node_modules` / `dist` / lock noise where noted):

| Bucket | ~Files | Owner task |
|---|---:|---|
| Product **DingoDB** copy | 26 | WEB-2, WEB-4 |
| **dingodb.org** host strings | 10 | WEB-1, WEB-2, WEB-6 |
| **docs.dingodb.org** host strings | 13 | WEB-1, WEB-4, WEB-6 |
| **ResiduumDB** / Residuum misspell | 12 + 9 | WEB-2, WEB-4, content bodies |
| **residuumdb** wrong domain (one-i path/host) | 11 | WEB-1 runbooks/README |
| **DQL** prose/labels | 6 | WEB-2, WEB-4, WEB-5 |
| **DRE** prose/labels | 7 | WEB-2, WEB-4, WEB-5 |
| **choose-dingodb** | 2 (+ content file) | WEB-5 |
| Old route hrefs (`/guides/dql` etc.) | 9 | WEB-5 |
| Legacy API samples (`dingo_sdk` / `Dingo::`) | 2 | WEB-3 |
| Class C profile literals | 11 | **retain** |

---

## 5. Package / config identity (WEB-1)

| File | Current | Target |
|---|---|---|
| `web/residiuumdb.org/package.json` `name` | `dingodb.org` | `residiuumdb.org` |
| `web/residiuumdb.org/package.json` description | DingoDB / dingodb.org | Residiuum / residiuumdb.org |
| `web/docs.residiuumdb.org/package.json` `name` | `docs.dingodb.org` | `docs.residiuumdb.org` |
| `web/docs.residiuumdb.org/package.json` description | DingoDB / docs.dingodb.org | Residiuum / docs.residiuumdb.org |
| `web/residiuumdb.org/astro.config.mjs` `site` | `https://dingodb.org` | `https://residiuumdb.org` |
| `web/docs.residiuumdb.org/astro.config.mjs` `site` | `https://docs.dingodb.org` | `https://docs.residiuumdb.org` |
| Main `src/data/release.json` URL fields | `docs.dingodb.org`… | `docs.residiuumdb.org`… |
| Docs `src/data/release.json` | `mainSite`/`docsBase` dingodb | residiuumdb hosts |
| `dingoSourceRevision` | Still dual-written with `sourceRevision` | Prefer **`sourceRevision` only**; update scripts + `PageMetadata.astro` |

**Already good:** both packages use `sdkCrate: "residuum-sdk"`; main release has `sourceRevision` present.

---

## 6. Route rename table (WEB-5)

Content files today → target filenames/paths:

| Old public path | New public path | Content file (today) |
|---|---|---|
| `/guides/dql/` | `/guides/rql/` | `src/content/guides/dql.md` |
| `/concepts/dql/` | `/concepts/rql/` | `src/content/concepts/dql.md` |
| `/concepts/dre/` | `/concepts/rre/` | `src/content/concepts/dre.md` |
| `/reference/dql/` | `/reference/rql/` | `src/content/reference/dql.md` |
| `/specifications/dql/` | `/specifications/rql/` | `src/content/specifications/dql.md` |
| `/specifications/dre/` | `/specifications/rre/` | `src/content/specifications/dre.md` |
| `/getting-started/choose-dingodb/` | `/getting-started/choose-residiuum/` | `src/content/getting-started/choose-dingodb.md` |

Also update: `navigation.json`, `migration-manifest.json`, internal markdown hrefs, any search index inputs.

**Note:** Some content titles already say “RQL guide” / “Choose ResiduumDB” while paths remain dql/choose-dingodb — path rename still required.

Spec sources in migration-manifest still reference `DQL_SPEC.md` / `DRE_SPEC.md` if those root files remain; prefer `RQL_SPEC.md` / `RRE_SPEC.md` when they exist in the repo.

---

## 7. Redirects & continuity (WEB-6)

| Source | Current | Target |
|---|---|---|
| Main `public/_redirects` `/docs/*` | `https://docs.dingodb.org/:splat` 302 | `https://docs.residiuumdb.org/:splat` |
| Docs `public/_redirects` | **Missing** | Create: 301 from each old WEB-5 path → new path |
| DNS/CDN old hosts | Outside repo | Document for principal: `dingodb.org` / `docs.dingodb.org` → Residiuum hosts |

Default redirect status for renamed docs paths: **301**.

---

## 8. Task map (implementation order)

```
WEB-0 (this doc) → WEB-1 → (WEB-2 ∥ WEB-4) → WEB-3 → WEB-5 → WEB-6 → WEB-7
```

| Task | Focus |
|---|---|
| WEB-1 | package.json, astro `site`, release URLs, drop dual `dingoSourceRevision`, runbooks paths |
| WEB-2 | Main site visible branding (pages, chrome, OG/favicon) |
| WEB-3 | Main samples / claims / capabilities / github crate paths |
| WEB-4 | Docs chrome + product Residiuum spelling (not route file moves) |
| WEB-5 | Docs route renames + nav + migration-manifest |
| WEB-6 | Redirects + DNS notes |
| WEB-7 | Both-site validate/build + residual audit + REBRAND.md Phase 4 closeout |

---

## 9. Acceptance for WEB-0

- [x] Residual brand buckets inventoried with file counts  
- [x] Route rename table complete  
- [x] Class C keep-list for web explicit  
- [x] Hosting project_id freeze recorded; docs hosting absence noted  
- [x] Dir state: no leftover `web/dingodb.org` paths  
- [x] No package renames performed in this task  

---

## 10. Evidence commands used

```text
ls web/
cat web/residiuumdb.org/.openai/hosting.json
# no docs .openai/hosting.json
# scanned both trees excluding node_modules/dist for DingoDB|dingodb.org|Residuum*|DQL|DRE|choose-dingodb|dingo-*-v1|...
```

---

## 11. WEB-6 redirects (implemented)

### Main site (`web/residiuumdb.org/public/_redirects`)
- `/docs/*` → `https://docs.residiuumdb.org/:splat` (302)
- `/get-started/*` → docs getting-started rust quickstart (302)
- Comment documents principal DNS: `dingodb.org` / `docs.dingodb.org` cutover + `www.residiuumdb.org` → apex 301

### Docs site (`web/docs.residiuumdb.org/public/_redirects`)
301 permanent from WEB-5 old paths:
- `/guides/dql` → `/guides/rql/`
- `/concepts/dql` → `/concepts/rql/`
- `/concepts/dre` → `/concepts/rre/`
- `/reference/dql` → `/reference/rql/`
- `/specifications/dql` → `/specifications/rql/`
- `/specifications/dre` → `/specifications/rre/`
- `/getting-started/choose-dingodb` → `/getting-started/choose-residiuum/`

Legacy IA redirects retained (302): quickstart, recovery, architecture, api.

### Hosting
`web/residiuumdb.org/.openai/hosting.json` `project_id` **unchanged**: `appgprj_6a6a4bd6baf08191949a0106278a04d8`.

### Principal remaining ops
1. DNS/CDN: map `dingodb.org` and `docs.dingodb.org` to Residiuum hosts when ready.
2. CDN: `www.residiuumdb.org` → apex 301.
3. Optional: separate docs OpenAI Sites hosting binding if/when authorized.
