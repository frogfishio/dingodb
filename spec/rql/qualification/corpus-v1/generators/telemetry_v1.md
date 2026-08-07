# Telemetry fixture generators (v1)

## Shared PRNG

Same splitmix64 as [commerce_v1.md](./commerce_v1.md); each generator restarts from its own `seed`.

## `telemetry.devices_v1`

**Params:** `n_devices` (24), `types` `["sensor","gateway","actuator"]`,
`sites` `["plant-a","plant-b","field"]`, `statuses` `["online","offline","degraded"]`.

| Field | Rule |
|---|---|
| `_key` | `d-{i:04d}` |
| `type` | types[i % 3] |
| `site` | sites[i % 3] |
| `status` | statuses[i % 3] |
| `last_seen` | `2024-08-01T00:00:00Z` + i×15 minutes |
| `firmware` | `1.{i%5}.{i%10}` |
| `labels` | `[]` every 5th; else `["edge", site]` |
| `retired_at` | null every 7th; omit every 11th; else absent meaning live |

## `telemetry.events_v1`

**Params:** `n_events` (128), `n_devices` (24),
`severities` `["info","warn","error","critical"]`.

| Field | Rule |
|---|---|
| `_key` | `ev-{i:04d}` |
| `device_id` | `d-{(i % n_devices):04d}` |
| `ts` | `2024-08-01T00:00:00Z` + i minutes |
| `severity` | severities[i % 4] |
| `metric` | pick temp_c/humidity/voltage/rpm |
| `value` | numeric most; **every 9th** string `"NaN"` (wrong_type) |
| `payload` | null every 6th; omit every 8th; else `{raw, unit}` |
| `tags` | `[]` every 10th; else `[metric, severity]` |

## `telemetry.metrics_v1`

**Params:** `n_metrics` (64), `n_devices` (24),
`metric_names` `["temp_c","humidity","voltage"]`.

| Field | Rule |
|---|---|
| `_key` | `mt-{i:04d}` |
| `device_id` | device key |
| `metric_name` | names[i % 3] |
| `value` | float(i % 100) |
| `recorded_at` | base + i×5 minutes |
