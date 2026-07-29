# dingodb.org

Product and project website for [DingoDB](https://github.com/frogfishio/dingodb).

Specification: repository root [`WEBSITE_SPEC.md`](../../WEBSITE_SPEC.md).

## Stack

- TypeScript
- [Astro](https://astro.build) static output
- Structured JSON for release, capabilities, claims, navigation
- Self-hosted IBM Plex Sans / Mono
- No client framework; tiny progressive-enhancement scripts only (code copy)

## Commands

```sh
npm install
npm run sync-release   # pull VERSION + git SHA into src/data/release.json
npm run validate       # claim/status/prohibited-language checks
npm run dev            # local preview
npm run build          # validate + static build → dist/
npm run preview        # serve dist/
```

## Routes

| Path | Purpose |
|------|---------|
| `/` | Home narrative |
| `/survival/` | Independent survival model |
| `/how-it-works/` | Architecture overview |
| `/use-cases/` | Fit / not-yet |
| `/status/` | Capability maturity |
| `/roadmap/` | Direction by gates |
| `/security/` | Reporting + posture |
| `/project/` | Source, licenses, contributing |
| `/privacy/` | Privacy statement |

`/benchmarks/` is intentionally omitted until a public evidence package exists.

## Content updates

See [CONTENT_RUNBOOK.md](./CONTENT_RUNBOOK.md).

## Deploy notes

- Canonical host: `https://dingodb.org`
- Configure `www.dingodb.org` → apex permanent redirect at the CDN/DNS layer
- `public/_headers` supplies CSP and security headers for Netlify-style hosts
- Set `PUBLIC_PREVIEW=true` on preview deployments for `noindex`
