# Content runbook (residiuumdb.org)

For release owners updating version and maturity without editing the same fact in multiple pages.

## Single sources of truth

| Fact | File |
|------|------|
| Product version, docs/GitHub URLs, site revision | `src/data/release.json` |
| Capability matrix (profiles, works-today, limitations) | `src/data/capabilities.json` |
| Claim records | `src/data/claims.json` |
| Status vocabulary | `src/data/status-vocabulary.json` |
| Nav / footer links | `src/data/navigation.json` |
| Roadmap tracks | `src/data/roadmap.json` |
| Repository VERSION | `../../VERSION` (synced into release.json) |

Canonical product maturity evidence remains `doc/CAPABILITY_MATRIX.md` in the monorepo. Update that document first, then mirror user-facing labels into `capabilities.json`.

## Release bump procedure

1. Confirm `VERSION` and capability matrix in the monorepo.
2. From `web/residiuumdb.org`:
   ```sh
   npm run sync-release
   ```
3. Update `capabilities.json` and any claim `verified_for` / `last_verified` fields to the new version.
4. Run:
   ```sh
   npm run validate
   npm run build
   ```
5. Record website revision + product revision in the release notes (footer already renders both).

## Adding a claim

1. Add a record to `claims.json` with a stable `id` (e.g. `claim.area.name`).
2. Reference the id from page components via `<Claim id="..." />` or `claim_id` on capability rows.
3. Never put a stronger public status than the claim record or capability matrix.

## Status labels only

Allowed: `available`, `experimental`, `development-only`, `scaffold`, `design`, `deferred`.

Unknown values fail `npm run validate` and the Astro build (StatusBadge).

## Prohibited language

See `WEBSITE_SPEC.md` §5.4. The validate script scans page sources for common prohibited phrases.

## Benchmarks page

`/benchmarks/` is omitted while `release.benchmarksPublished` is `false`. Status page §Benchmark status explains that no comparative result is claimed. Set the flag and add the page only when a full methodology package exists.