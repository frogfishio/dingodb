# JSON Schema to RRE cross-compiler specification

Status: **Normative design v1.0-draft**

Compiler profile: `json-schema-2020-12-to-dre-v1`

Source dialect: JSON Schema Draft 2020-12

Target dialect: `dre`

Audience: migration-tool, schema, compiler, SDK, and conformance implementers

Normative companions: [RRE_SPEC.md](./RRE_SPEC.md),
[RESIDIUUM_PREDICATE_SPEC.md](../../reference/query/RESIDIUUM_PREDICATE_SPEC.md), and
[ATOMICS_SPEC.md](../atomics/ATOMICS_SPEC.md)

Authoritative source specifications:

- <https://json-schema.org/draft/2020-12/json-schema-core>
- <https://json-schema.org/draft/2020-12/json-schema-validation>

## 1. Purpose

This compiler translates a precisely supported subset of JSON Schema Draft
2020-12 into Residiuum Rule Expressions.

It is an importer, not a claim that RRE implements all JSON Schema
vocabularies. JSON Schema contains applicators, dynamic references, regular
expressions, annotations, content vocabularies, and open extension mechanisms
that RRE v1 intentionally does not reproduce.

The compiler rule is:

> Preserve validation equivalence for every admissible Residiuum document, or
> refuse the schema.

Unsupported keywords are never ignored merely because some JSON Schema
validators treat unknown extension keywords as annotations.

## 2. Equivalence

For accepted source schema `J`, emitted RRE ruleset `R`, bound collection `C`,
and every admissible Residiuum JSON document `d`:

```text
ValidateJsonSchema202012(J, d)
    =
EvaluateDre(R, d)
```

The comparison concerns validation only. JSON Schema annotations such as
`title`, `description`, and `default` do not alter validity. Supported
annotations are retained in the translation receipt but do not become rules.

Compilation returns:

```text
JsonSchemaToDreResult =
    Exact
  | Refused
```

There is no best-effort executable class. A caller may request an analysis
report showing translatable fragments, but partial output cannot be activated.

## 3. Required input envelope

The root must be a JSON object containing:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object"
}
```

Requirements:

- `$schema` is mandatory and must identify Draft 2020-12;
- the root schema must require `type: "object"`;
- the compiler receives a target `HeapId`, immutable collection identity,
  ruleset ID, and revision;
- source JSON is decoded with duplicate object keys rejected;
- numbers are parsed as exact arbitrary-precision decimal values within the
  compiler ceiling;
- source bytes and all resolved references are content-hashed.

A root Boolean schema, missing root object type, or another draft is refused.

## 4. Vocabulary policy

The compiler understands only the keyword set in this document.

If `$vocabulary` is present:

- every vocabulary marked `true` must be recognized and supported for every
  keyword used;
- an unsupported required vocabulary refuses compilation;
- an unsupported optional vocabulary is accepted only when the schema uses no
  keyword from it.

Recognized annotation-only keywords:

```text
$id
$anchor
title
description
default
examples
deprecated
readOnly
writeOnly
$comment
```

They are preserved in the receipt and do not affect RRE.

`format` is accepted only under the Draft 2020-12 format-annotation vocabulary
and is recorded as non-enforcing metadata. If the format-assertion vocabulary
is required, compilation is refused because RRE v1 has no matching format
types.

Content metadata keywords are refused in v1 rather than silently discarded.

Every other unlisted keyword is `json_schema_keyword_unsupported`.

## 5. Reference resolution

V1 supports:

- local JSON Pointer references beginning `#/`;
- `$defs`;
- acyclic reference graphs;
- a `$ref` object containing only `$ref` and recognized annotations.

V1 refuses:

- `$dynamicRef` and `$dynamicAnchor`;
- recursive or cyclic `$ref`;
- remote HTTP/file resolution;
- relative external references;
- bundled resources with a different base dialect;
- assertion siblings beside `$ref`.

The compiler performs no network or filesystem access. A future profile may
accept an explicit resolver bundle:

```text
URI -> { exact bytes, media type, digest }
```

but ambient fetching will remain forbidden.

Pointer resolution follows Draft 2020-12 Core. Missing targets, invalid
escapes, duplicate canonical resource IDs, and digest mismatches are hard
errors.

## 6. Type mapping

Supported JSON Schema types map as follows:

| JSON Schema | RRE |
|---|---|
| `"null"` | `null` |
| `"boolean"` | `boolean` |
| `"string"` | `string` |
| `"number"` | `number` |
| `"integer"` | `integral` |
| `"object"` | open or closed `product` |
| `"array"` | bounded `sequence` |

JSON Schema `integer` accepts a numeric value with no fractional part. That is
why it maps to RRE `integral`, not representation-specific `integer`.

A type array is supported only when it contains:

- one type; or
- exactly one supported non-null type plus `"null"`.

The latter maps to `nullable(T)`. Other unions are refused.

If a keyword is type-specific, the applicable `type` must be explicit in the
same resolved schema. The compiler does not guess a type from the keyword.

An empty schema object, or a schema object containing only recognized
annotations, maps to `any` when used as a property or item schema.

## 7. Object mapping

### 7.1 Properties and required

Given:

```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "age": { "type": "integer" }
  },
  "required": ["name"]
}
```

the type component becomes:

```text
document as product {
  name: string,
  optional age: integral
}
```

Rules:

- a property in `required` is a required product field;
- every other declared property is optional;
- `required` may name a property absent from `properties`; it maps to a
  required field of type `any`;
- required means present, not non-null by itself;
- Null is valid only if the property's translated type admits Null;
- property names use canonical bracket path segments when not RRE identifiers.

This preserves JSON Schema's distinction between a missing required property
and a present property whose value is Null.

### 7.2 Additional properties

```text
additionalProperties absent or true  -> open product
additionalProperties false           -> closed product
additionalProperties schema object   -> refused
```

JSON Schema `properties` does not close an object by default. The compiler must
not emit `closed product` unless the source says `additionalProperties: false`.

### 7.3 Nested objects

Nested objects translate recursively into nested product types. Their own
`required` and `additionalProperties` apply at that nested object only.

### 7.4 Dependent required

```json
{
  "dependentRequired": {
    "credit_card": ["billing_address"]
  }
}
```

adds:

```text
require billing_address
  when present(credit_card)
```

Every dependent name is a direct property name. The source array must contain
unique strings. Cycles are legal because the declarations are implications,
not procedures.

### 7.5 Refused object keywords

V1 refuses:

- `patternProperties`;
- schema-valued `additionalProperties`;
- `unevaluatedProperties`;
- `propertyNames`;
- `minProperties` and `maxProperties`;
- `dependentSchemas`.

## 8. Array mapping

Supported homogeneous array:

```json
{
  "type": "array",
  "items": { "type": "string" },
  "minItems": 1,
  "maxItems": 20
}
```

maps to:

```text
sequence(string, min 1, max 20)
```

Rules:

- `items` must be one supported schema;
- `maxItems` is mandatory and must be at most the RRE profile ceiling;
- `minItems` defaults to zero;
- `minItems <= maxItems`;
- every element must satisfy the translated item type.

V1 refuses:

- missing `maxItems`;
- `prefixItems` tuple validation;
- Boolean-false `items`;
- `contains`, `minContains`, and `maxContains`;
- `uniqueItems: true`;
- `unevaluatedItems`.

`uniqueItems: false` is accepted as the default and emits no rule.

## 9. String mapping

```json
{
  "type": "string",
  "minLength": 1,
  "maxLength": 100
}
```

maps to:

```text
string(min 1, max 100)
```

Both JSON Schema and the target count Unicode scalar/code points, not UTF-8
bytes or grapheme clusters.

Either bound may appear independently. Bounds must fit the RRE profile.

V1 refuses:

- `pattern`;
- enforcing `format`;
- content encoding/media/schema keywords.

No regex dialect is selected implicitly.

## 10. Numeric mapping

For property path `price`:

```json
{
  "type": "number",
  "minimum": 0,
  "exclusiveMaximum": 1000
}
```

emits the type plus:

```text
constrain price
  where missing(price)
     or (price >= 0 and price < 1000)
```

When the property is required, the compiler may omit the `missing(price)`
guard. When its type admits Null, the guard also includes `price is null`.

Mappings:

| JSON Schema | RRE predicate |
|---|---|
| `minimum: n` | `p >= n` |
| `maximum: n` | `p <= n` |
| `exclusiveMinimum: n` | `p > n` |
| `exclusiveMaximum: n` | `p < n` |

Bounds are exact decimals. NaN and infinities are not JSON numbers and cannot
enter the source profile.

`multipleOf` is refused in v1 because the RRE predicate profile has no exact
divisibility operator.

## 11. Enum and const

`enum` maps to RRE `enum(...)` only when every member is a unique scalar from:

```text
Null | Boolean | String
```

`const` maps to a singleton enum under the same restriction.

Numeric, array, and object enum members are refused in v1 because JSON Schema
numeric equality and RRE/SDA representation equality have not yet been given a
shared canonical mapping.

An enum combined with a type must have every member conform to the translated
type; otherwise the schema is statically unsatisfiable and refused.

## 12. Boolean schemas and applicators

Schema `true` is accepted only as a property/items subschema and maps to `any`.

Schema `false` is refused. RRE v1 activation rejects unsatisfiable rulesets and
does not use an impossible schema as executable configuration.

V1 refuses all general schema combinators:

- `allOf`;
- `anyOf`;
- `oneOf`;
- `not`;
- `if`, `then`, and `else`.

It also refuses applicator behavior not explicitly supported in §§7–8.

These can be added only when a future RRE profile can preserve their exact
validation algebra and dependency bounds.

## 13. Generated RRE

Input:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.test/person",
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 200
    },
    "age": {
      "type": "integer",
      "minimum": 0,
      "maximum": 150
    },
    "nickname": {
      "type": ["string", "null"],
      "maxLength": 100
    }
  },
  "required": ["name", "age"],
  "additionalProperties": false
}
```

Output:

```text
rules for people named person_schema revision 1

document as closed product {
  name: string(min 1, max 200),
  age: integral,
  optional nickname: nullable(string(max 100))
}

constrain age
  where age >= 0 and age <= 150
```

The compiler emits canonical bracket paths when JSON member names require
them.

## 14. Canonical translation result

```text
JsonSchemaToDreExact {
  compiler_profile
  source_dialect
  source_id?
  source_hash
  resolved_resource_hashes
  target_heap_id
  target_collection_id
  ruleset_id
  revision
  dre_source
  canonical_dre_ir
  dependency_set
  required_atomic_scope
  translation_receipt
}
```

The receipt records:

- every source JSON Pointer;
- the generated RRE declaration/IR node;
- type and constraint mapping;
- required/optional status;
- open/closed object decision;
- ignored annotations;
- reference resolutions and hashes;
- profile ceilings;
- compiler version.

The output is a **proposed** RRE revision. Translation never activates it.
Normal RRE compilation, independent artifact verification, existing-data
validation, coverage proof, and Atomic activation still apply.

## 15. Stable diagnostics

```text
json_schema_parse_error
json_schema_duplicate_key
json_schema_dialect_required
json_schema_dialect_unsupported
json_schema_vocabulary_unsupported
json_schema_keyword_unsupported
json_schema_root_object_required
json_schema_reference_invalid
json_schema_reference_unresolved
json_schema_reference_cycle
json_schema_remote_reference_forbidden
json_schema_type_unsupported
json_schema_type_required
json_schema_union_unsupported
json_schema_object_keyword_unsupported
json_schema_array_unbounded
json_schema_array_keyword_unsupported
json_schema_string_keyword_unsupported
json_schema_numeric_keyword_unsupported
json_schema_enum_unsupported
json_schema_combinator_unsupported
json_schema_unsatisfiable
json_schema_profile_limit
json_schema_target_scope_invalid
json_schema_generated_dre_invalid
```

Diagnostics contain the source JSON Pointer, bounded explanation, and source
span when the parser retains it.

## 16. Limits

V1 ceilings:

| Quantity | Maximum |
|---|---:|
| source schema bytes | 1,048,576 |
| resolved local schema nodes | 65,536 |
| `$ref` depth | 64 |
| object properties at one level | 4,096 |
| total generated declarations | 1,024 |
| path segments | RRE profile maximum |
| string/array bounds | RRE profile maximum |
| generated RRE source | 262,144 bytes |
| generated canonical IR | RRE artifact maximum |

The target Heap may impose lower limits. Exceeding a limit refuses compilation;
the compiler never truncates a schema.

## 17. Security

- No network or filesystem reference resolution occurs.
- All source and resolved bytes are hashed.
- Duplicate JSON object keys are rejected.
- Target collection identity is bound inside exactly one Heap.
- Schema text grants no capability.
- Translation cannot activate or replace a RRE.
- Annotation strings are data and never executable.
- Regex, scripts, code generation hooks, and custom runtime functions are
  absent.
- Compiler resource limits apply before expansion to prevent reference bombs.

## 18. Conformance

Conformance requires:

- the official Draft 2020-12 validation examples for every supported keyword;
- required versus optional and Null versus missing;
- open versus closed nested objects;
- exact number/integer/integral cases;
- Unicode length cases;
- bounded homogeneous arrays;
- dependent-required cycles;
- local `$defs` and escaped JSON Pointers;
- annotation-only format handling;
- every explicit refusal;
- unknown and required-vocabulary refusal;
- duplicate-key and reference-bomb cases;
- generated RRE parse, canonicalization, and independent verification;
- property-based equivalence:

```text
ValidateJsonSchema202012(J, d)
    =
EvaluateDre(Compile(J), d)
```

for generated supported schemas and admissible documents.

## 19. Implementation sequence

1. Build duplicate-rejecting exact-number JSON decoding.
2. Validate Draft 2020-12 dialect and vocabulary declarations.
3. Implement bounded local `$ref` resolution.
4. Build the supported schema AST and refusal walker.
5. Translate primitive and object types.
6. Translate bounded arrays and string lengths.
7. Translate exact numeric bounds and dependent-required.
8. Emit canonical RRE source and IR.
9. Emit translation receipts.
10. Differentially test against a conforming Draft 2020-12 reference
    validator.
11. Submit the result through ordinary RRE verification and activation.

No step may silently ignore an assertion keyword.
