# Project management fixture generators (v1)

## Shared PRNG

Same splitmix64 as [commerce_v1.md](./commerce_v1.md); each generator restarts from its own `seed`.

## `project_management.projects_v1`

**Params:** `n_projects` (20), `statuses` `["active","on_hold","done","archived"]`,
`owners` `u-0000`…`u-0007`.

| Field | Rule |
|---|---|
| `_key` | `prj-{i:04d}` |
| `title` | `"Project {i}"` |
| `status` | statuses[i % 4] |
| `owner_id` | owners[i % 8] |
| `created_at` | `2024-02-01T00:00:00Z` + i×120 minutes |
| `priority` | 1 + (i % 5) |
| `description` | null every 4th; omit every 7th; else `"Desc {i}"` |
| `labels` | `[]` every 5th; else `["pm", status]` |

## `project_management.tasks_v1`

**Params:** `n_tasks` (80), `n_projects` (20), `n_users` (8),
`statuses` `["todo","doing","in_review","done"]`.

| Field | Rule |
|---|---|
| `_key` | `tsk-{i:04d}` |
| `project_id` | `prj-{(i % n_projects):04d}` |
| `title` | `"Task {i}"` |
| `status` | statuses[i % 4] |
| `assignee_id` | `u-{(i % n_users):04d}` |
| `updated_at` | base + i×10 minutes |
| `estimate_points` | 1 + (i % 8) |
| `due_at` | null every 6th; omit every 9th; else logical due |
| `blocked_by` | `[]` most; sometimes prior task key |

## `project_management.revisions_v1`

**Params:** `n_revisions` (48), `n_projects` (20), `n_users` (8).

| Field | Rule |
|---|---|
| `_key` | `rev-{i:04d}` |
| `project_id` | project key |
| `rev_no` | (i // n_projects) + 1 |
| `author_id` | user key |
| `summary` | `"Revision {i}"` |
| `committed_at` | `2024-02-15T00:00:00Z` + i×30 minutes |
| `bytes` | 100 + i×17 |

## `project_management.memberships_v1`

**Params:** `n_projects` (20), `n_users` (8), `roles` `["owner","editor","viewer"]`.

| Field | Rule |
|---|---|
| `_key` | `mb-prj-{p}-u-{u}` |
| `project_id` | project key |
| `user_id` | user key |
| `role` | owner for first member; else editor/viewer |
| `joined_at` | project base + offset |

2–4 memberships per project (2 + p%3).
