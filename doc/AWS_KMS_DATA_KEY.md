# AWS KMS data-key provider (live)

**Status:** connected (feature-gated)  
**Crate:** `residuum-store` feature `aws-kms`  
**Type:** [`AwsKmsDataKeyProvider`](../crates/residuum-store/src/heap/kms_aws.rs)

## What it does

| Op | Behavior |
|----|----------|
| **generate** | KMS `GenerateDataKey` (`AES_256`) under your CMK; returns envelope DEK (plaintext + ciphertext) on a [`DataKeyHandle`] |
| **destroy** | Zero local plaintext + ciphertext; durable destroy receipt under `meta/lifecycle/` — **does not** delete the shared CMK |

Encryption context on generate:

- `residuum-heap-id` — hex heap id  
- `dingo-profile` — `residuum-heap-v1`

## Build

```bash
cargo check -p residuum-store --features aws-kms
```

Default store builds do **not** pull HTTP/SigV4 deps.

## Configure

| Variable | Role |
|----------|------|
| `DINGO_AWS_KMS_KEY_ID` or `DINGO_KMS_KEY_ARN` | CMK id, alias, or ARN (**required**) |
| `AWS_REGION` or `DINGO_AWS_REGION` | Region (default `us-east-1`) |
| `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` | Credentials (**required** for live calls) |
| `AWS_SESSION_TOKEN` | Optional session token |
| `DINGO_AWS_ENDPOINT_URL` or `AWS_ENDPOINT_URL` | Optional (LocalStack / VPC endpoint) |

```rust
use residuum_store::{AwsKmsDataKeyProvider, DataKeyProvider, HsmDataKeyConfig};

// From env
let p = AwsKmsDataKeyProvider::from_env()?;

// Or explicit
let cfg = HsmDataKeyConfig::aws_kms(
    "us-east-1",
    "alias/residuum-heap",
    None, // or Some("http://localhost:4566".into()) for LocalStack
);
let p = AwsKmsDataKeyProvider::from_config(&cfg)?;
let mut h = p.generate(heap_id)?;
p.destroy(data_root, &mut h)?;
```

## Live Accept test

```bash
export DINGO_KMS_LIVE=1
export DINGO_AWS_KMS_KEY_ID=alias/your-cmk
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
cargo test -p residuum-store --features aws-kms aws_kms_live_generate_destroy -- --ignored --nocapture
```

## Honesty

- This is a **live wire protocol** client (SigV4 HTTPS), not the in-process mock.
- Plaintext DEK is returned once by KMS (envelope pattern); handle must wipe on destroy.
- Shared CMK is never schedule-deleted by destroy (would break multi-tenant keys).
- PKCS#11 / GCP KMS / Azure Key Vault remain scaffold-only until similar connectors land.
