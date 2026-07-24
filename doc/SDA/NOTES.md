# SDA + Grit0 notes

## Current gaps hit immediately

1) **Unicode token spellings now work (UTF-8 round-trip fixed)**

Unicode spellings like `⟨ ⟩`, `∈`, `∧`, `∨`, `¬`, `≠`, `≤`, `≥`, `→`, `↦`, `∣`, `⊎`, `⊖` now tokenize correctly.

Root cause was not the lexer; it was pack JSON loading: JSON string parsing was corrupting raw UTF-8 bytes in token literals.
That’s fixed in [src/common/json_min.c](src/common/json_min.c).

2) **Selector access `<selector>` needs lexer-level disambiguation**

SDA wants selector access using `<selector>` (or `⟨selector⟩`).

- If `<`/`>` are used for both comparisons and postfix selector access, the grammar becomes ambiguous at the token level.
- The SDA pseudogrammar explicitly calls out the usual solution: have the lexer return distinct tokens (`SEL_L`/`SEL_R`) for selector brackets.

With Unicode brackets available, `poc/SDA/sda.grit` can use `⟨selector⟩` without colliding with `<`/`>` comparisons.

Remaining Unicode-ish outliers:

- Column semantics: diagnostics currently count columns in bytes, not Unicode scalar values/graphemes.
- Any syntax that requires context-sensitive lexing (e.g. ASCII selector `<selector>` vs comparisons) still needs a disambiguation strategy.

5) **Unicode identifiers (XID) now work for `Identifier`**

To get closer to “languages like Zing”, the runtime lexer now supports Unicode identifiers (Greek/Chinese/Cyrillic, etc.) using Unicode XID rules.

Current behavior:

- Token named `Identifier` with the common ASCII regex pattern `[A-Za-z_][A-Za-z0-9_]*` is upgraded to a Unicode-aware matcher (XID_Start/XID_Continue plus `_`).
- This keeps pack format stable (no new token kinds), but is still a heuristic/compat shim.

If we want this to be fully explicit and spec-controlled, the next step is to add a first-class “unicode identifier” token kind in the pack format and have `seed` emit it.

3) **Unresolved rule/token refs surface as “failed to load pack”**

If a rule references a symbol that is neither a token nor a rule (e.g. `BindEntry = Selector Arrow Expr` without defining `Selector`), pack loading fails with a generic message:

- `error: failed to load pack (missing/invalid JSON?)`

This is *not* a JSON validity issue; it’s the seed-pack → internal pack compilation step failing due to an unresolved `ref`.

4) **Keyword collisions: regex token priority can lose to `Identifier`**

The lexer uses maximal-munch with tie-break `priority_then_rule_order`. In the current seed-pack compiler output:

- string tokens like `InterKW = "inter"` get priority `200`
- regex tokens like `Identifier = /.../` (and alternations that compile to regex) get priority `100`

That means a definition like:

- `DiffKW = "diff" | "\\";`

compiles to a regex token (priority `100`) and can lose to `Identifier` on ties (same length), causing `diff` to lex as `Identifier` and the parse to fail.

Workaround: split into separate string tokens (e.g. `DiffKW = "diff"; Backslash = "\\";`) so both sides keep priority `200`.
