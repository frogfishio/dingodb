# dingo-client

Thin **MIT** network client primitives for DingoDB: length-prefixed framed JSON
RPC (`dingo-rpc-v1`), hello/welcome handshake, and feature negotiation.

This crate has **no** dependency on the store, cluster, or server. Application
collection APIs live in `dingo-sdk`; TCP serve lives in `dingo-server`.

See `doc/LICENSING.md` for the multi-tier license map.
