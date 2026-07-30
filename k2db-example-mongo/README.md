<!-- SPDX-FileCopyrightText: 2026 Alexander R. Croft -->
<!-- SPDX-License-Identifier: MIT -->

# k2db

Rust data layer for MongoDB with enforced metadata, soft-delete semantics, scoped ownership filtering, and guarded aggregation.

## What it provides

- Automatic `_uuid`, `_owner`, `_created`, `_updated`, and `_deleted` metadata handling.
- Soft-delete enforcement across reads, writes, and aggregation pipelines.
- Optional ownership scoping with lax and strict enforcement modes.
- Optional guarded and strict aggregation validation.
- Built-in observability hooks and Ratatouille logging integration.

## Install

```toml
[dependencies]
k2db = "0.1.1"
```

## Status

This crate is the Rust replatform of the k2db data layer and is published under MIT.

Repository: https://github.com/frogfishio/k2db