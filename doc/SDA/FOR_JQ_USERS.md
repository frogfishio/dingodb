# SDA for jq Users

This page is for people who already know `jq` and want to understand SDA quickly.

The short version is:

- both tools can read JSON from stdin
- both can run short inline programs
- both can be used in shell pipelines
- SDA is usually stricter and more explicit

SDA is not trying to replace every part of jq.
It is aiming at a different tradeoff:

> less guesswork, more explicit data semantics

## 1. What will feel familiar

These ideas transfer easily from jq:

- JSON in on stdin
- JSON out on stdout
- short expressions for quick tasks
- file-based programs for longer tasks
- shell-friendly use in scripts and pipelines

Examples:

```sh
echo '{"a":1,"b":2}' | sda eval -e 'values(input)'
sda eval -f transform.sda -i input.json
```

## 2. What will feel different

SDA is more explicit about a few things that jq often leaves loose.

### Missing values

In SDA, missing data is often represented explicitly.

Examples:

- optional lookup: `input<"name">?`
- required lookup: `input<"name">!`

That means you will see wrapper results like:

- `Some(...)`
- `None`
- `Ok(...)`
- `Fail(...)`

### Failure is part of the model

SDA does not try to smooth over every problem.
It often gives you a named failure instead.

Example:

```sh
echo '{}' | sda eval -e 'input<"name">!'
```

Typical output:

```json
{
  "$type": "fail",
  "$code": "t_sda_missing_key",
  "$msg": "missing key"
}
```

### Duplicates matter more

SDA has explicit collection kinds for cases where duplicates are meaningful.

Important examples:

- `Seq[...]` ordered list
- `Set{...}` unique values only
- `Bag{...}` duplicates matter
- `BagKV{...}` duplicate-bearing keyed values matter

If you are used to ordinary JSON objects flattening everything into one map-like shape, SDA may feel more formal here.

## 3. A useful mental translation

You can often think in this rough way:

- jq filter input: `.`
- SDA filter input: `input`

So where jq often starts from `.`:

```jq
.name
```

SDA often starts from `input`:

```text
input<"name">!
```

That is not exact feature-for-feature equivalence.
It is just a useful starting intuition.

## 4. Similar jobs, different style

### Get values from an object

SDA:

```sh
echo '{"a":1,"b":2}' | sda eval -e 'values(input)'
```

### Filter a list

SDA:

```sh
echo '[1,2,3,4]' | sda eval -e '{ x in input | x > 2 }'
```

### Transform a list

SDA:

```sh
echo '[1,2,3,4]' | sda eval -e '{ yield x * 2 | x in input }'
```

## 5. Why SDA may seem stricter

If you come from jq, SDA may seem fussy at first.

That usually comes from three design choices:

1. make missing/optional/required behavior explicit
2. keep duplicate-bearing structures visible
3. return named failures instead of silently guessing

That strictness is intentional.
It helps when you care more about certainty than convenience.

## 6. Best way to learn SDA if you know jq

Do not try to translate every jq habit directly.

Instead:

1. learn `input`
2. learn `input<"name">!` and `input<"name">?`
3. learn `values(input)`
4. learn one comprehension: `{ x in input | x > 2 }`
5. learn one pipe: `input |> values(_)`

That is enough to become productive quickly.

## 7. Where next

- [SDA User Manual](USER_MANUAL.md)
- [SDA Cheat Sheet](CHEATSHEET.md)
- [SDA JSON Filter Demo](JSON_FILTER_DEMO.md)
- [SDA Specification](SDA_SPEC.md)