# SDA User Manual

This guide is for people who want to use SDA without already knowing algebra, programming language theory, or the internal Axiom architecture.

If words like algebra, expression, or transformation sound intimidating, ignore them for now.
You do not need a math background to use this tool.
You only need to know that SDA helps you read data, check data, and reshape data in a careful way.

The goal is simple:

- explain what SDA is
- show how to use the `sda` tool
- teach the basic language by example
- avoid academic language unless it is truly needed

## 0. A quick start you can finish in 10 minutes

If you want the fastest possible introduction, start here.

This repository includes two tiny example files:

- `SDA/examples/getting_started/person.json`
- `SDA/examples/getting_started/person_name.sda`

The JSON file contains one person record.
The SDA file contains one very small program:

```text
input<"name">!
```

That means:

- start with the incoming data
- look for the `name` entry
- treat it as required

Now run these commands:

```sh
sda check -f SDA/examples/getting_started/person_name.sda
sda fmt -f SDA/examples/getting_started/person_name.sda --check
sda eval -f SDA/examples/getting_started/person_name.sda -i SDA/examples/getting_started/person.json
```

What these do:

1. `check` makes sure the SDA source is valid
2. `fmt --check` makes sure it is written in standard style
3. `eval` actually runs it on the JSON input

If you want to try one more command right away:

```sh
echo '{"name":"Ada","city":"London"}' | sda eval -e 'input<"city">!'
```

If those examples make sense, you already understand the basic shape of SDA.

## 1. What SDA is

SDA is a small language for working with structured data.

You can think of it as a careful data tool for jobs like:

- picking values out of JSON
- checking whether data has the shape you expect
- cleaning up messy input
- turning one data shape into another
- making failures explicit instead of hiding them

If you have used tools like `jq`, spreadsheets, or simple scripts, the easiest way to think about SDA is:

> SDA is a tool for asking structured questions about data and getting structured answers back.

It is called an algebra in the formal documents, but you do not need that word to use it well.
For everyday use, it is enough to think of SDA as a precise data language.

If you want an even simpler picture:

- JSON is the thing you have
- SDA is the thing you write
- `sda` is the tool that runs it

## 2. What the `sda` command does

The command-line tool is called `sda`.

It has four main jobs:

1. run an SDA expression on input data
2. check that SDA source is valid
3. format SDA source into a consistent style
4. show version and license information

The most important commands are:

```sh
sda eval -e 'values(input)'
sda check -e 'values(input)'
sda fmt -e 'input<name>!'
sda --version
sda --license
```

## 3. A first example

Suppose you have this JSON:

```json
{"name":"Ada","city":"London"}
```

You can ask SDA to extract the `name` field like this:

```sh
echo '{"name":"Ada","city":"London"}' | sda eval -e 'input<"name">!'
```

What this means:

- `input` means “the incoming JSON value”
- `<"name">!` means “get the `name` entry, and treat it as required”

If the key exists, SDA returns an `Ok(...)` result.
If it does not exist, SDA returns a clear failure value instead of quietly pretending everything worked.

That is a big part of SDA’s style:

- be explicit
- do not hide uncertainty
- do not quietly flatten duplicates or missing values unless you asked for that

Another way to say it is:

> SDA prefers a clear answer over a convenient guess.

## 4. The three commands you will use most

### `sda eval`

Run SDA code.

Examples:

```sh
sda eval -e '1 + 2'
echo '{"a":1,"b":2}' | sda eval -e 'values(input)'
sda eval -f extract.sda -i event.json
```

Useful flags:

- `-e, --expr` for a short inline expression
- `-f, --file` to read SDA code from a file
- `-i, --input` to read JSON from a file instead of stdin
- `--compact` for one-line JSON output
- `--bind` to use a name other than `input`

### `sda check`

Check whether SDA code is valid without running it.

Examples:

```sh
sda check -e 'values(input)'
sda check -f extract.sda
```

This is useful in editors, CI, and before running a bigger script.

### `sda fmt`

Format SDA code into a consistent style.

Examples:

```sh
sda fmt -e ' input<name>! '
sda fmt -f extract.sda --check
sda fmt -f extract.sda --write
sda fmt --stdin-filepath extract.sda < extract.sda
```

Useful modes:

- plain `fmt`: print the formatted result
- `--check`: exit with an error if the source is not already formatted
- `--write`: rewrite a file in place

## 5. The kinds of values SDA understands

You do not need to memorize every formal value type at the start.
The important ones are these:

### Plain values

- numbers like `1` and `3.5`
- text like `"Ada"`
- true/false values: `true`, `false`
- `null`

### Collections

- `Seq[...]` means an ordered list
- `Set{...}` means unique items, with duplicates removed
- `Bag{...}` means a collection where duplicates matter

Examples:

```text
Seq[1, 2, 3]
Set{"red", "green", "red"}
Bag{"red", "green", "red"}
```

### Keyed data

- `Map{"name" -> "Ada"}` means key/value data like a JSON object
- `Prod{name: "Ada", city: "London"}` means a record with named fields

For beginners, it is fine to think of both as “objects with named pieces,” but SDA keeps a distinction because it helps the language stay precise.

If that distinction feels too technical right now, do not worry about it.
You can learn a lot of SDA just by following examples and copying working patterns.

### Result-like wrappers

Sometimes SDA returns wrappers to make uncertainty visible:

- `Ok(...)` means a required lookup succeeded
- `Fail(code, msg)` means something went wrong in a defined way
- `Some(...)` and `None` are used for optional values

This can look unusual at first, but it prevents a lot of silent data mistakes.

The short version is:

- sometimes SDA tells you the value
- and also tells you how safe or certain that value is

## 6. Reading values out of data

This is one of the most common jobs.

If you only learn one SDA skill on day one, learn this section.

### Required lookup

```text
input<"name">!
```

Meaning:

- look for `name`
- if it is there, return `Ok(...)`
- if it is missing, return a failure

### Optional lookup

```text
input<"name">?
```

Meaning:

- look for `name`
- if it is there, return `Some(...)`
- if it is missing, return `None`

### Plain field access

```text
Prod{name: "Ada"}<name>
```

This is used when the field is part of a known record shape.

## 7. Doing simple transformations

### Arithmetic

```text
1 + 2
10 / 2
```

### Text join

```text
"Ada" ++ " Lovelace"
```

### Membership

```text
2 in Seq[1, 2, 3]
"name" in Map{"name" -> "Ada"}
```

### Boolean checks

```text
1 < 2
true and false
not false
```

## 8. Filtering a list

Suppose the input is:

```json
[1,2,3,4,5]
```

You can keep only numbers greater than `2` like this:

```text
{ x in input | x > 2 }
```

Run it:

```sh
echo '[1,2,3,4,5]' | sda eval -e '{ x in input | x > 2 }'
```

You can also transform values while filtering:

```text
{ yield x * 10 | x in input | x > 2 }
```

That means:

1. read each item from `input`
2. keep only items where `x > 2`
3. output `x * 10`

If the word `yield` is new to you, read it as:

> for each item that passes the check, output this new value

## 9. Working with JSON objects

Suppose the input is:

```json
{
  "user": {
    "id": 7,
    "name": "Ada"
  }
}
```

You can pull out the inner object like this:

```text
input<"user">!
```

You can list the values of a plain JSON object like this:

```text
values(input)
```

Example:

```sh
echo '{"a":1,"b":2}' | sda eval -e 'values(input)'
```

## 10. Chaining steps with pipes

SDA supports a pipe operator:

```text
input |> values(_)
```

This means:

1. start with `input`
2. pass it into `values(_)`

The `_` symbol is the placeholder for “the value flowing through the pipe.”

If you have seen shell pipes before, the idea is similar:

- take one thing
- send it into the next step
- then into the next step after that

Another example:

```text
input |> values(_) |> count(2, _)
```

This reads as:

- take the input
- get its values
- count how many times `2` appears

## 11. Why SDA sometimes returns `Ok(...)` or `Fail(...)`

Many data tools make strong assumptions quietly.

For example, if a key is missing, they may:

- return `null`
- return nothing
- silently continue

SDA tries not to guess.

If you asked for something required, SDA gives you a result that says clearly whether it worked.

That can feel stricter at first, but it is often safer in real data work.

This is especially useful when your data comes from:

- APIs that change over time
- logs with missing fields
- imported JSON files you do not fully trust
- duplicate-bearing inputs where “just take one” would be dangerous

For example:

```text
input<"name">!
```

does not mean “maybe get `name`.”
It means “this value is required, so say clearly whether it exists.”

## 11.1 What the output often looks like

If you are new to SDA, one confusing thing at first is the shape of the output.

Here are the most common patterns.

### Plain value output

Command:

```sh
sda eval -e '1 + 2'
```

Output:

```json
3
```

### Sequence output

Command:

```sh
echo '{"a":1,"b":2}' | sda eval -e 'values(input)'
```

Output:

```json
[
  1,
  2
]
```

### Required lookup success

Command:

```sh
echo '{"name":"Ada"}' | sda eval -e 'input<"name">!'
```

Typical output shape:

```json
{
  "$type": "ok",
  "$value": "Ada"
}
```

### Optional lookup success

Command:

```sh
echo '{"name":"Ada"}' | sda eval -e 'input<"name">?'
```

Typical output shape:

```json
{
  "$type": "some",
  "$value": "Ada"
}
```

### Optional lookup missing

Command:

```sh
echo '{}' | sda eval -e 'input<"name">?'
```

Typical output shape:

```json
{
  "$type": "none"
}
```

### Required lookup failure

Command:

```sh
echo '{}' | sda eval -e 'input<"name">!'
```

Typical output shape:

```json
{
  "$type": "fail",
  "$code": "t_sda_missing_key",
  "$msg": "missing key"
}
```

You do not need to memorize every wrapper.
What matters is the idea:

- plain values stay plain
- optional and required operations may return wrapper objects
- failures are explicit and named

## 12. Typical everyday workflow

If you are just starting, this is a good pattern:

1. try a small expression inline with `sda eval -e ...`
2. move it into a file when it gets longer
3. run `sda check -f your_file.sda`
4. run `sda fmt -f your_file.sda --write`
5. then use `sda eval -f your_file.sda -i your_input.json`

This is a good habit because it separates three different questions:

- is the source valid?
- is the source tidy?
- does it produce the result I want?

Example:

```sh
sda check -f extract.sda
sda fmt -f extract.sda --write
sda eval -f extract.sda -i event.json --compact
```

## 13. A few examples you can copy

### Get a required field

```sh
echo '{"name":"Ada"}' | sda eval -e 'input<"name">!'
```

### Get all values from an object

```sh
echo '{"a":1,"b":2}' | sda eval -e 'values(input)'
```

### Filter a list

```sh
echo '[1,2,3,4]' | sda eval -e '{ x in input | x > 2 }'
```

### Transform a list

```sh
echo '[1,2,3,4]' | sda eval -e '{ yield x * 2 | x in input }'
```

### Format a file

```sh
sda fmt -f extract.sda --write
```

### Check formatting in CI

```sh
sda fmt -f extract.sda --check
```

### Show the tool version

```sh
sda --version
```

### Show copyright and license notice

```sh
sda --license
```

## 13.2 One failure on purpose

It is useful to see one failure on purpose, because SDA failures are part of the normal workflow.

Try this command:

```sh
echo '{}' | sda eval -e 'input<"name">!'
```

Why this fails:

- the input is an empty object
- the program asks for a required `name`
- there is no `name` key

Typical result:

```json
{
  "$type": "fail",
  "$code": "t_sda_missing_key",
  "$msg": "missing key"
}
```

How to read that result:

- `$type: fail` means this was a defined SDA failure, not random garbage output
- `$code` gives a stable machine-friendly reason
- `$msg` gives a short human-friendly explanation

If you want a softer version that does not fail when the key is missing, try this instead:

```sh
echo '{}' | sda eval -e 'input<"name">?'
```

That returns `None` rather than a failure.

## 13.1 Copy these commands exactly

If you prefer learning by typing exactly what you see, start with this transcript.

```sh
$ sda check -f SDA/examples/getting_started/person_name.sda
ok

$ sda fmt -f SDA/examples/getting_started/person_name.sda --check

$ sda eval -f SDA/examples/getting_started/person_name.sda -i SDA/examples/getting_started/person.json
{
  "$type": "ok",
  "$value": "Ada"
}

$ sda eval -e '1 + 2'
3

$ echo '{"a":1,"b":2}' | sda eval -e 'values(input)'
[
  1,
  2
]
```

That transcript shows the three most important beginner ideas:

1. SDA source can be checked before running
2. SDA source can be formatted automatically
3. SDA output may be plain JSON or an explicit wrapper like `Ok(...)`

## 14. A second small tutorial: make a summary object

Once you are comfortable with the first quick start, try this next step.

There are two more example files for this tutorial:

- `SDA/examples/getting_started/person_summary.sda`
- `SDA/examples/getting_started/person.json`

The SDA program is:

```text
bindRes(
  input<"name">!,
  name => bindRes(
    input<"city">!,
    city => Ok(Map{"person" -> name, "location" -> city})
  )
)
```

Run it like this:

```sh
sda check -f SDA/examples/getting_started/person_summary.sda
sda fmt -f SDA/examples/getting_started/person_summary.sda --write
sda eval -f SDA/examples/getting_started/person_summary.sda -i SDA/examples/getting_started/person.json
```

What it does:

1. get the required `name`
2. get the required `city`
3. build a new object with the keys `person` and `location`

Why it uses `bindRes` and `Ok(...)`:

- `input<"name">!` returns a required-result value
- `bindRes(...)` says “if that worked, continue”
- `Ok(...)` wraps the final successful result

If this feels more advanced than the first tutorial, that is fine.
The important thing to notice is that SDA can build new output, not only read old input.

## 15. Things that are different from ordinary JSON tools

SDA is friendly for shell use, but it is intentionally stricter than many lightweight JSON tools.

Key differences:

- duplicates can matter
- missing values are treated explicitly
- failures are named and visible
- formatting and checking are built in
- the language tries to avoid hidden guesses

That strictness is not there to be difficult.
It is there to help you trust the result.

## 16. Common mistakes and what they usually mean

If SDA gives you an error or a failure value, it usually means something specific and useful.

Here are common beginner mistakes.

### Mistake: forgetting that `input` is the starting value

Example:

```text
<"name">!
```

Problem:

- SDA needs to know what value you are reading from

Better:

```text
input<"name">!
```

### Mistake: using `_` when there is no pipe

Example:

```text
_
```

Problem:

- `_` only has meaning inside a pipe step

Better:

```text
input |> values(_)
```

### Mistake: expecting required lookup to behave like optional lookup

Example:

```text
input<"name">!
```

Problem:

- `!` means required
- if the key is missing, SDA will tell you clearly instead of quietly continuing

If you want the softer form, use:

```text
input<"name">?
```

### Mistake: writing messy SDA and then debugging the mess

Better workflow:

```sh
sda check -f program.sda
sda fmt -f program.sda --write
sda eval -f program.sda -i input.json
```

### Mistake: assuming every JSON object is just a loose bag of fields

In beginner examples, that assumption is often fine.
But as your programs grow, SDA may be more explicit than other tools about:

- missing keys
- duplicate-bearing structures
- whether something is optional or required

That is normal.
It is one of the reasons SDA is useful.

## 17. If SDA feels strange at first

That is normal.

Almost everyone needs a short adjustment period when moving from loose JSON tools to something more explicit.

The fastest way to get comfortable is:

1. start with `input`
2. try one lookup
3. try one filter
4. try one formatting command
5. keep expressions small until the pattern becomes natural

You do not need to learn the whole language at once.

If you remember only four things from this manual, remember these:

1. `input` is your incoming data
2. `sda eval` runs SDA code
3. `sda check` validates SDA code
4. `sda fmt` keeps SDA code tidy and consistent

## 18. Short FAQ

### Do I need to know algebra to use SDA?

No.
You can treat SDA as a careful data language.

### Is SDA only for programmers?

No, but it helps if you are comfortable with command-line tools and JSON.
The language is small enough that non-specialists can learn it in pieces.

### Why does SDA return things like `Ok(...)` and `Fail(...)`?

Because SDA tries to show whether an operation was safely successful, optional, or a real failure.
That is often safer than silently returning nothing.

### Is SDA the same as jq?

No.
It can be used in a jq-like way, but it is stricter and more explicit about missing values, duplicates, and failure states.

If you come from jq, read [FOR_JQ_USERS.md](FOR_JQ_USERS.md) next.

### What should I learn first?

Start with:

1. `input`
2. `input<"name">!`
3. `values(input)`
4. `sda eval`
5. `sda check`

### What should I do when something looks confusing?

Make the program smaller.
Try one lookup, or one filter, and see what comes back.
Small examples are the fastest way to learn SDA.

## 19. Where to go next

If you want more after this guide:

- [SDA Cheat Sheet](CHEATSHEET.md)
- [SDA for jq Users](FOR_JQ_USERS.md)
- [SDA JSON Filter Demo](JSON_FILTER_DEMO.md)
- [Commands Overview](../COMMANDS.md)
- [SDA Specification](SDA_SPEC.md)

Use this manual first.
Use the specification when you need exact rules.