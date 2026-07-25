# Protocol golden fixtures (DEF-031)

Compatibility fixtures for the framed `dingo-rpc-v1` network protocol.

## Framing

Every production message is:

```
u32 big-endian length | UTF-8 JSON payload (exactly `length` bytes)
```

Handshake messages and application RPCs share this framing. Application bodies
are documented in `dingo-sdk` as `RpcRequest` / `RpcResponse`.

## Files

| Fixture | Role |
|---------|------|
| `hello.v1.json` | Client first message (control) |
| `welcome.v1.json` | Server success handshake |
| `reject_version.v1.json` | Version mismatch reject |
| `ping_request.v1.json` | Minimal application request |
| `ping_response.v1.json` | Minimal application response |

Fixtures are canonical **payloads** (not length-prefixed). Tests build frames
with `encode_frame` / `write_frame`.

## Diagnostic profile

Newline-delimited JSON without handshake is available only when both client and
server set `diagnostic_line_protocol = true`. It is not covered by these
production fixtures.
