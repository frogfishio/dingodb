---
last_verified: 2026-07-30
claim_ids:
title: CLI quickstart
description: Install dingo CLI, put/get a document, and run doctor on a temporary store.
class: tutorial
status: experimental
section: getting-started
order: 3
applies_to:
  product: 0.2
  surface: embedded-single-node
source:
  path: crates/residiuum-cli/README.md
owners:
  - cli
keywords:
  - cli
  - console
  - doctor
---

## Install

Prefer a **versioned** release or build from the repository pin matching this docs set (**0.2.0**).

From a clone of the monorepo:

```bash
cargo install --path crates/residiuum-cli --locked
dingo --version
```

Do **not** pipe an unauthenticated network installer into a shell unless the project deliberately publishes and documents that mechanism (none is advertised here).

## Create a temporary store

```bash
WORKDIR="$(mktemp -d /tmp/residiuum-cli-XXXXXX)"
echo "Using $WORKDIR"
```

## Put and get via console

```bash
printf '%s\n' \
  "PUT $WORKDIR users/user-1 {\"name\":\"hello\",\"status\":\"active\"}" \
  "GET $WORKDIR users/user-1" \
  "QUIT" \
| dingo console "$WORKDIR"
```

## Doctor / inspection

```bash
residuum doctor "$WORKDIR"
```

## Cleanup

```bash
rm -rf "$WORKDIR"
```

## Maturity

CLI and server bits carry **AGPL** networked-product licensing for serve/cluster paths. Embedded evaluation still uses store semantics shared with the SDK. Network `serve` is **development only**; `serve-cluster` is **experimental, not production**.

Next: [Rust quickstart](/getting-started/rust/) · [Operations](/operations/)
