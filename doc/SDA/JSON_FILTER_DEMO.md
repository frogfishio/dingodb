# SDA JSON Filter Demo

This document shows the current `sda` CLI as a jq-like JSON filter over stdin or JSON files.

The point is not that SDA copies jq syntax.
The point is that SDA can already act as a precise JSON transformation filter while preserving the stronger carrier and failure semantics of the algebra.

## 1. Basic usage

Evaluate an inline expression over stdin:

```sh
echo '{"a":1,"b":2}' | cargo run -q -p sda -- eval -e 'values(input)'
```

Check source without evaluating it:

```sh
cargo run -q -p sda -- check -e 'values(input)'
```

Read the program from a file and the JSON from stdin:

```sh
cargo run -q -p sda -- eval -f extract.sda < event.json
```

Read the program and input from files:

```sh
cargo run -q -p sda -- eval -f extract.sda -i event.json
```

Compact output for shell pipelines:

```sh
echo '[1,2,3]' | cargo run -q -p sda -- eval -e 'input ++ Seq[4]' --compact
```

## 2. Filter a sequence of records

Input:

```json
[
  {"$type":"prod","$fields":{"name":"steve","city":"la"}},
  {"$type":"prod","$fields":{"name":"steve","city":"ny"}},
  {"$type":"prod","$fields":{"name":"ada","city":"la"}}
]
```

Program, using wrapped `Prod` records so total field access stays total:

```text
{ a in input | a<name> = "steve" and a<city> in Set{"la", "ny"} }
```

Command:

```sh
cargo run -q -p sda -- eval -e '{ a in input | a<name> = "steve" and a<city> in Set{"la", "ny"} }' -i records.json
```

This is the closest current jq-style story:

- stdin or file JSON in
- structured filter expression
- structured JSON out

## 3. Normalize duplicate headers safely

Input:

```json
{
  "$type": "bagkv",
  "$items": [
    ["content-type", "application/json"],
    ["x-trace", "a"],
    ["x-trace", "b"]
  ]
}
```

Program:

```text
input
|> normalizeUnique(_)
|> bindRes(_, m => m<"content-type">!)
```

Command:

```sh
cargo run -q -p sda -- eval -f header_lookup.sda -i headers.json
```

This shows what jq does not naturally emphasize:

- duplicate-bearing input stays explicit as `BagKV`
- normalization policy is explicit
- missing/duplicate behavior stays visible in the result algebra

## 4. Project and reshape JSON input

Input:

```json
{
  "user": {
    "id": 7,
    "name": "Ada"
  }
}
```

Program, using explicit result handling because plain JSON objects decode to `Map`:

```text
bindRes(
  input<"user">!,
  user => bindRes(
    user<"id">!,
    id => bindRes(
      user<"name">!,
      name => Ok(Map{"id" -> id, "display" -> name})
    )
  )
)
```

Command:

```sh
echo '{"user":{"id":7,"name":"Ada"}}' | cargo run -q -p sda -- eval -e 'bindRes(input<"user">!, user => bindRes(user<"id">!, id => bindRes(user<"name">!, name => Ok(Map{"id" -> id, "display" -> name}))))'
```

The output is an `Ok(...)` wrapper because the extraction path is required and explicit.

## 5. Why this is jq-like but not jq

What it shares with jq:

- JSON in / JSON out
- shell-friendly stdin/stdout usage
- inline expression execution
- small file-based filters

What is intentionally different:

- SDA keeps carrier distinctions beyond plain JSON when needed
- failures are explicit semantic values where the algebra defines them
- multiplicity and duplicate-bearing structures are first-class
- the goal is certainty, not just convenience