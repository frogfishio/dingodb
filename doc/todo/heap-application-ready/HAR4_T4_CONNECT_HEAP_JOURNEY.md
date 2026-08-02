# HAR-4 T4 — Journey: public remote tutorials use `connect_heap` only

Status: **labor done (in_review)** · 2026-08-02 · package **HAR-4 active / not accept**  
Board: `b4eda326` · Feature Query spine `1a8a3e05`  
Authority: [HEAP_APPLICATION_READY_PLAN.md](./HEAP_APPLICATION_READY_PLAN.md) §HAR-4 exit ·
[HAR4_QUERY_REMOTE_GAP_INVENTORY.md](./HAR4_QUERY_REMOTE_GAP_INVENTORY.md)

This is the **Journey** evidence pack for HAR-4 T4. It is **not** HAR-4 package
accept.

---

## 1. Exit requirement (normative)

From HEAP plan HAR-4:

> Exit: every public remote tutorial uses `connect_heap`; shared-token examples
> are in a legacy appendix only.

Product meaning:

| Surface | Product remote | Non-product (appendix only) |
|---|---|---|
| Client | `Residiuum::connect_heap` + `RemoteHeapOptions` (TLS + HeapCredential) | `Residiuum::connect` / `connect_with` + `auth_token` |
| Server library | `ServeOptions::new()` (qualified default) + TLS + `deployment_id` + `heap_registry` | `ServeOptions::legacy_token_server()` + optional `auth_token` |
| CLI | `residiuum serve … --qualified-heap-key --tls-cert … --tls-key … --deployment-id …` | `residiuum serve … --legacy-token-server [--token …]` |
| Config | `serve.qualified_heap_key=true` + TLS + `serve.deployment_id` | `serve.legacy_token_server=true` + token refs |

---

## 2. What changed (this labor)

| Artifact | Change |
|---|---|
| `crates/residiuum-server/README.md` | Quick example leads with qualified HeapKey; shared-token demoted to appendix |
| `crates/residiuum-cli/README.md` | Serve section leads with product flags; legacy serve/token appendix |
| `crates/residiuum-sdk/README.md` | Remote section leads with `connect_heap`; token path appendix |
| This pack | Journey checklist + residual honesty |
| Inventory / scoreboard | T4 closed as labor; package still not accept |

---

## 3. Product journey sketch (shape, not full ceremony)

HeapKey issuance and resident registry install remain **HAR-2/HAR-3** ceremony
surfaces. Public tutorials document the **product wire shape** without claiming
that a one-liner opens a production Heap without authority setup.

### 3.1 Server (library)

```rust
use residiuum_server::{serve_store_with, ServeOptions};
use residiuum_sdk::TlsServerOptions;
// heap_registry + deployment_id from authority / HP-008 install

let opts = ServeOptions::new() // product default: qualified_heap_key=true
    .tls(TlsServerOptions::new(cert_path, key_path))
    .deployment_id(deployment_uuid)
    .heap_registry(registry);
serve_store_with(store_path, "127.0.0.1:7434", opts)?;
```

### 3.2 Server (CLI)

```sh
residiuum serve ./app.residiuum \
  --bind 127.0.0.1:7434 \
  --qualified-heap-key \
  --tls-cert ./server.crt --tls-key ./server.key \
  --deployment-id 00000000-0000-4000-8000-000000000001
```

Startup reports label `auth_path=qualified-heap-key (product)`.

### 3.3 Client

```rust
use residiuum_sdk::{
    HeapCredential, RemoteHeapOptions, Residiuum, TlsClientOptions,
};
use std::sync::Arc;

// certificate_cose + HolderSigner from local authority ceremony (HAR-2/3)
let credential = HeapCredential::new(&certificate_cose, holder)?;
let options = RemoteHeapOptions::new(
    TlsClientOptions::new("localhost").ca_path(ca_path),
    credential,
)
.expected_heap_name("accounts");

let mut heap = Residiuum::connect_heap(
    "residiuum://127.0.0.1:7434/accounts",
    options,
)?;
// RemoteHeap: process ops + collection plane under §32.4; query via op 118 when active
```

There is **no** shared token, role, username, or plaintext product path.

---

## 4. Legacy appendix (explicit non-product)

Shared-token / open Stage-7 remain available for diagnostics and Stage-7 tests.
They **must** be labeled non-product and require `--legacy-token-server` /
`ServeOptions::legacy_token_server()` / `serve.legacy_token_server=true`.

| Forbidden product claim | Honest label |
|---|---|
| “Just `residiuum serve` + `connect` is the product remote” | Config may *imply* legacy with a warning; that is not HeapKey product |
| Token + qualified co-host | Fail-closed (HAR-4 T2/T3) |
| Tutorial primary example uses `auth_token` | Demoted to appendix |

Integration reference (qualified live path):  
`crates/residiuum-server/tests/hp007_connect_heap.rs`  
Gate locks: `crates/residiuum-server/tests/har4_query_remote_gate.rs`

---

## 5. Checklist (T4 labor exit)

| Check | Result |
|---|---|
| Public crate READMEs (server / cli / sdk) lead remote tutorial with `connect_heap` or qualified serve | **yes** |
| Shared-token examples only under labeled legacy / non-product section | **yes** |
| Inventory H4-G2 tutorial residual closed as labor | **yes** |
| HAR-4 package accept | **no** (principal) |
| Full operator ceremony tutorial (issue key → serve → connect) | residual HAR-2/3 + HAR-6 |

---

## 6. Explicit non-claims

- No HAR-4 **package accept**.
- No claim that CLI `serve` without `--qualified-heap-key` is product default
  for operators: config apply still defaults unset paths to **legacy with
  warnings** (HAR-4 T3 honesty); library `ServeOptions::default()` remains
  qualified.
- No claim that README snippets alone provision a HeapKey or registry.
- Op **118** wire status is APP-7 evidence, not this card.

---

## 7. Evidence pointers

| Artifact | Role |
|---|---|
| This file | Journey evidence pack |
| `crates/residiuum-{server,cli,sdk}/README.md` | Public tutorial honesty |
| `HAR4_QUERY_REMOTE_GAP_INVENTORY.md` | Gap table T4 row |
| `doc/wip/status/NEXT_BUILD_STATUS.md` | Scoreboard HAR-4 row |
| `hp007_connect_heap` + `har4_query_remote_gate` | Runtime product path + gates |
