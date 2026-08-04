# Supported versions policy (DEF-063-A)

Policy id: `residiuum-supported-versions-v1`
Companion: [SECURITY.md](../../../SECURITY.md), [VERSION](../../../VERSION),
[doc/wip/status/CAPABILITY_MATRIX.md](../../wip/status/CAPABILITY_MATRIX.md)

## Purpose

Define which software versions receive security and critical-correctness fixes,
and what “supported” means before production maturity is claimed.

## Current product maturity

| Claim | Status |
|-------|--------|
| Production-ready | **No** — do not deploy as a sole system of record without accepting residual risk |
| Semantic version in tree | See root [`VERSION`](../../../VERSION) (crate workspace version) |
| Wire profile | `WIRE_PROFILE_LABEL = "1.0-draft"` until DEF-053 freeze ([WIRE_MAJOR1_FREEZE.md](../../wip/format/WIRE_MAJOR1_FREEZE.md)) |

## Support classes

| Class | What we commit to | When it applies |
|-------|-------------------|-----------------|
| **Development tip** | Best-effort fixes on `main` / default branch | Always for active development |
| **Tagged pre-release** | Security-relevant fixes **if** maintainers designate the tag as supported in release notes | Only when release notes say so |
| **Supported release line** | Security + critical correctness for the declared window | Only after a production maturity claim and support line announcement |
| **Unsupported** | No commitment | Everything else (forks, unofficial builds, expired lines) |

Until a production maturity announcement:

1. **Supported line = development tip** (latest default branch) plus any tag
   explicitly marked “security-supported” in its release notes.
2. Older tags and third-party packages are **unsupported** by default.
3. Experimental surfaces (`serve-cluster`, unfinished Studio paths) are
   **unsupported for production** even on tip.

### Packaging 0.2.0 — unsafe for continued store writes

Workspace packaging **0.2.0** / `residiuum-store` **0.2.0** is **unsafe for
continued writes across reopen/rotation** due to segment-identity remint and
sealed-media replacement (see
[SECURITY_ADVISORY_SEGID_0.2.0.md](../../todo/performance-qualification/SECURITY_ADVISORY_SEGID_0.2.0.md)).
Upgrade to **≥ 0.2.2** when published. **0.2.0** and **0.2.1** are yanked on
crates.io (0.2.1 withdrawn for red qualification suite). Prefer yank status as SoT. Affected on-disk stores are **not** auto-repaired.

## Version identity

- **Crate/workspace version:** monorepo `VERSION` / Cargo package versions.
- **Wire profile:** `residiuum_format::WIRE_PROFILE_LABEL` — independent of crate
  version; remains draft until DEF-053 freeze criteria pass (checklist and
  compatibility policy: [WIRE_MAJOR1_FREEZE.md](../../wip/format/WIRE_MAJOR1_FREEZE.md);
  runtime: `wire_is_frozen()` / `wire_freeze_summary()`).
- **Store on-disk:** store meta / descriptor generations; upgrade/migration
  rules are per DEF-052 and capability matrix — not all historical media are
  forever writable by new majors.

## Security fixes

For a **supported** version:

- Critical/high security issues get a fix or published mitigation under
  [SECURITY.md](../../../SECURITY.md) timelines.
- Fixes may be shipped as a new patch tag; operators must upgrade.

For **unsupported** versions:

- No commitment to backports. Maintainers may still land tip fixes that help
  operators rebuild from source.

## Operator expectations

1. Pin commits or tags intentionally; record the exact revision in ops notes.
2. Run `residiuum doctor` / salvage / backup procedures on upgrade paths as
   documented.
3. Do not assume `1.0-draft` wire will remain binary-compatible without reading
   migration notes after DEF-053 freeze (see [WIRE_MAJOR1_FREEZE.md](../../wip/format/WIRE_MAJOR1_FREEZE.md)
   §4 relabel procedure).
4. Treat experimental multi-node paths as non-production until DEF-041-N (and
   related) evidence is accepted.

## Changing this policy

Material changes require:

1. Update of this document’s policy id or a dated revision note.
2. Link from [SECURITY.md](../../../SECURITY.md) and README security section.
3. CAPABILITY_MATRIX honesty if support claims affect maturity language.

## Residual

Production **support line** windows (N-1 minor, LTS, etc.) are **not** declared
until production maturity is claimed. This document intentionally avoids
promising LTS before that gate.
