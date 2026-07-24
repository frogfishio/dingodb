# SDA Cheat Sheet

This is the short version of SDA for day-to-day use.

## Core idea

- `input` is the incoming JSON value
- `sda eval` runs SDA code
- `sda check` validates SDA code
- `sda fmt` formats SDA code

## Most useful commands

```sh
sda eval -e 'values(input)'
sda eval -f program.sda -i input.json
sda check -f program.sda
sda fmt -f program.sda --write
sda fmt -f program.sda --check
sda --version
sda --license
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
sda check -f program.sda
sda fmt -f program.sda --write
sda eval -f program.sda -i input.json
```

## If something feels confusing

Start smaller:

1. try one lookup
2. then one filter
3. then one file-based program

See the full guide in [USER_MANUAL.md](USER_MANUAL.md).