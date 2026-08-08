# Application Driver Spine

Status: **developer-ready; execution governed by `CRITICAL_PATH.md`**

Normative specification:
[ASYNC_DRIVER_SPINE_SPEC.md](./ASYNC_DRIVER_SPINE_SPEC.md).

Start `DRV-0` only. This supplies the async application path required by RQL;
it does not admit non-Rust bindings, Atomics implementation, or cluster work.

