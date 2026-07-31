# Documentation release runbook

## Freeze and generate

1. Freeze Residiuum source revision (git tag or SHA).
2. From `web/docs.residiuumdb.org`:
   ```sh
   npm run sync-release
   ```
3. Update `src/data/capabilities.json` from `doc/CAPABILITY_MATRIX.md`.
4. Refresh critical `last_verified` fields on getting-started, operations, security, and reference pages.
5. Run:
   ```sh
   npm run validate
   npm run build
   ```

## Version channels

| Channel | URL | Notes |
|---------|-----|-------|
| Current | `https://docs.residiuumdb.org/` | Latest published minor line |
| Next | `/next/` | Unreleased; `noindex` |
| Archive | `/versions/<major.minor>/` | Frozen line |

## Deploy checklist

1. Build static `dist/`
2. Deploy archive first if cutting a new minor
3. Switch canonical current docs
4. Verify: home, quickstart, status, search-index.json, sitemap, 404
5. Record docs revision + product version

## Content ownership

| Section | Owner role |
|---------|------------|
| Getting started | sdk |
| Guides | sdk |
| Operations | ops |
| Reference | sdk |
| Specifications | specs |
| Status | release |
| Contributing | maintainers |