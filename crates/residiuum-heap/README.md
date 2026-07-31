# residiuum-heap

Heap identity, capability, and authority kernel for Residiuum (`dingo-heap-v1`).

This crate is pure with respect to storage and network: no filesystem or TCP
runtime. Normative contract: [`HEAP_SPEC.md`](../../HEAP_SPEC.md) §§30–32, 38–41.
Machine-readable sources of truth live under [`spec/heap/`](../../spec/heap/).

## Packages

| Package | Status |
|---------|--------|
| HP-000 | Machine-readable contract + generated registry |
| HP-001 | Isolation kernel (IDs, rights, COSE cert/proof, `decide`) |

License: MIT.
