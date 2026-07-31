# SDA Cheat Sheet

This is the short version of SDA for day-to-day use.

## Core idea

- `input` is the incoming JSON value
- `residiuum-sda eval` runs SDA code
- `residiuum-sda check` validates SDA code
- `residiuum-sda fmt` formats SDA code
- **SDA (+ ENR1) is the mathematical language**; **RQL** is the official human
  dialect ([USER_GUIDE.md](../RQL/USER_GUIDE.md), [RQL_SPEC.md](../../RQL_SPEC.md)); JSON/Mongo filters and SQL-ish
  strings are foreign **dialects** that compile into pure SDA — not a hybrid of
  peer languages (see [DIALECTS.md](DIALECTS.md))
- **Null ≠ absence.** Stored `null` is `Some(null)`; missing key is `None`. If you
  need that distinction, write pure SDA — foreign dialects cannot express it losslessly

## Most useful commands

```sh
residiuum-sda eval -e 'values(input)'
residiuum-sda eval -f program.sda -i input.json
residiuum-sda check -f program.sda
residiuum-sda fmt -f program.sda --write
residiuum-sda fmt -f program.sda --check
residiuum-sda --version
residiuum-sda --license
```

## Read values

Required lookup:

```text
input<"name">!
```

Optional lookup:

```text
input<"name">?
```

## Common patterns

Get all values from an object:

```text
values(input)
```

Filter a list:

```text
{ x in input | x > 2 }
```

Transform a list:

```text
{ yield x * 2 | x in input }
```

Pipe data through steps:

```text
input |> values(_) |> count(2, _)
```

## Useful value types

- `Seq[...]` ordered list
- `Set{...}` unique items only
- `Bag{...}` duplicates matter
- `Map{"k" -> v}` key/value object-like value
- `Prod{name: v}` named-field record

## Common wrappers

- `Ok(...)` success for required-result flows
- `Fail(code, msg)` explicit failure
- `Some(...)` optional value present
- `None` optional value missing

Typical output shapes in JSON:

```json
{"$type":"ok","$value":"Ada"}
{"$type":"some","$value":"Ada"}
{"$type":"none"}
{"$type":"fail","$code":"t_sda_missing_key","$msg":"missing key"}
```

## Best beginner workflow

```sh
residiuum-sda check -f program.sda
residiuum-sda fmt -f program.sda --write
residiuum-sda eval -f program.sda -i input.json
```

## If something feels confusing

Start smaller:

1. try one lookup
2. then one filter
3. then one file-based program

See the full guide in [USER_MANUAL.md](USER_MANUAL.md).