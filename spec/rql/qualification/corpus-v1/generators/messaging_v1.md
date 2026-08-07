# Messaging fixture generators (v1)

## Shared PRNG

Same splitmix64 as [commerce_v1.md](./commerce_v1.md); each generator restarts from its own `seed`.

## `messaging.conversations_v1`

**Params:** `n_conversations` (24), `n_users` (12).

| Field | Rule |
|---|---|
| `_key` | `cv-{i:04d}` |
| `title` | `"Conversation {i}"` |
| `kind` | direct if i%4==0 else group |
| `created_at` | `2024-06-01T00:00:00Z` + i hours |
| `archived` | true every 11th |
| `last_message_at` | created_at + (next_u32()%1000) minutes |

## `messaging.messages_v1`

**Params:** `n_messages` (96), `n_conversations` (24), `n_users` (12).

| Field | Rule |
|---|---|
| `_key` | `m-{i:04d}` |
| `conversation_id` | `cv-{(i % n_conversations):04d}` |
| `sender_id` | `u-{(i % n_users):04d}` |
| `body` | `"msg body {i}"`; **every 7th** null |
| `sent_at` | `2024-06-01T00:00:00Z` + i minutes |
| `read_at` | **every 3rd** ISO timestamp; **every 5th** missing; else null (unread) |
| `edited` | true every 13th |
| `attachments` | `[]` most; every 9th `[{"type":"image","name":"a.jpg"}]` |

## `messaging.participants_v1`

**Params:** `n_conversations` (24), `n_users` (12), 2–4 participants per conversation.

| Field | Rule |
|---|---|
| `_key` | `pt-{cv}-{user}` |
| `conversation_id` | conversation key |
| `user_id` | user key |
| `role` | owner (first) / member |
| `muted` | true every 6th |
| `joined_at` | conversation created_at + offset |
