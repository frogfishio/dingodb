---
last_verified: 2026-07-30
claim_ids:
title: Documentation contributions
description: How docs are structured, verified, and released.
class: how-to
status: experimental
section: contributing
order: 1
applies_to:
  product: 0.2
  surface: all-profiles
source:
  path: doc/done/web/DOCS_SITE_SPEC.md
owners:
  - docs
keywords:
  - docs
---

## Principles

1. Task first
2. Truth at point of use (status badges)
3. One source — repository owns APIs and specs
4. Evidence reachable
5. Damage explicit

## Local site

```bash
cd web/docs.residiuumdb.org
npm install
npm run dev
npm run validate && npm run build
```

## Frontmatter

Every page declares class, status, applies_to, source.path, last_verified, owners, keywords.

## Release

See [CONTENT_RUNBOOK.md](https://github.com/frogfishio/dingodb/blob/main/web/docs.residiuumdb.org/CONTENT_RUNBOOK.md) in this package after publish.
