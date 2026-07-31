# docs.residiuumdb.org

Canonical documentation for [Residiuum](https://github.com/frogfishio/dingodb).

Specification: repository [`DOCS_SITE_SPEC.md`](../../doc/done/web/DOCS_SITE_SPEC.md). Shared truth/status contract: [`WEBSITE_SPEC.md`](../../doc/done/web/WEBSITE_SPEC.md).

## Stack

- Astro static site (TypeScript)
- Markdown content under `src/content/` with validated frontmatter
- Shared design tokens with residiuumdb.org (paper/ink palette, IBM Plex)
- Local search via on-demand `search-index.json` (no external provider)
- Build-time Markdown → HTML (no client Markdown evaluation)

## Commands

```sh
npm install
npm run sync-release
npm run validate
npm run dev
npm run build
npm run preview
```

## Content

Pages live in `src/content/**/*.md`. Frontmatter fields: `title`, `description`, `class`, `status`, `applies_to`, `source`, `last_verified`, `owners`, `keywords`, `claim_ids`, optional `spec_state`.

Migration mapping: `src/data/migration-manifest.json`.

Release process: [CONTENT_RUNBOOK.md](./CONTENT_RUNBOOK.md).