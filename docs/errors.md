# Error Reference

This document is the complete reference for every error type that the Alif
proof verifier can produce. It covers the error hierarchy, field-level
descriptions, display formats, common error messages with their causes and
fixes, exit codes, and concrete stderr output examples.

---

## Table of Contents

1. [Error Hierarchy](#error-hierarchy)
2. [ParseError](#parseerror)
   - [What triggers a ParseError](#what-triggers-a-parseerror)
   - [Fields](#parseerror-fields)
   - [Display format](#parseerror-display-format)
   - [Common parse errors](#common-parse-errors)
   - [ParseError examples](#parseerror-examples)
3. [CheckError](#checkerror)
   - [What triggers a CheckError](#what-triggers-a-checkerror)
   - [Fields](#checkerror-fields)
   - [Display format](#checkerror-display-format)
   - [Common check errors](#common-check-errors)
   - [CheckError examples](#checkerror-examples)
4. [RuleError](#ruleerror)
   - [Fields](#ruleerror-fields)
   - [Display format](#ruleerror-display-format)
   - [All rule error messages](#all-rule-error-messages)
   - [RuleError examples](#ruleerror-examples)
5. [Exit Codes](#exit-codes)
6. [Complete stderr output examples](#complete-stderr-output-examples)

---

## Error Hierarchy

Alif defines three error types arranged in the following hierarchy:

```
AlifError                         (top-level, returned by verify_source)
├── AlifError::Parse(ParseError)  (lexer / grammar failure)
└── AlifError::Check(CheckError)  (proof validity failure)
                    └── wraps RuleError messages (as strings in CheckError.message)
```

`AlifError` is the type returned by `verify_source`, the main public API. It
is an enum with two variants:

| Variant | Inner type | Meaning |
|---|---|---|
| `AlifError::Parse(e)` | `ParseError` | The source text could not be lexed or parsed. |
| `AlifError::Check(e)` | `CheckError` | The source parsed successfully but a theorem's proof is invalid. |

`RuleError` is **not** a variant of `AlifError`. Instead, when an inference
rule fails inside the checker, the `RuleError`'s `Display` string is captured
and stored as the `message` field of a `CheckError`. From the caller's
perspective, a rule failure surfaces as a `CheckError`.

All three types implement `std::error::Error` and `std::fmt::Display`.
`AlifError` implements `std::error::Error::source()` to expose the inner
`ParseError` or `CheckError` as the cause.

---

## ParseError

### What triggers a ParseError

A `ParseError` is produced by either the **lexer** (`src/lexer.rs`) or the
**parser** (`src/parser.rs`) when the source text does not conform to the Alif
grammar.

The lexer produces an error when it encounters a **character that is not part
of any recognised token**. The only characters the lexer recognises are:

- ASCII letters, digits, and underscores (identifiers and keywords)
- The operators `|-`, `=>`, `:=`, `:`, `,`, `(`, `)`
- The connective keywords `AND`, `OR`, `NOT` (uppercase only)
- Whitespace and `-- ...` line comments (silently skipped)

Any other character — including `@`, `#`, `$`, `%`, `!`, `?`, `"`, `'`,
semicolons, curly braces, and Unicode characters other than those appearing in
identifier names — will cause the lexer to return an error immediately.

The parser produces an error when the **token sequence does not match the
expected grammar**. Common grammar violations include:

- Missing `:` after an axiom or theorem name
- Missing `:=` in a `have` step
- Writing `qed` where a step keyword is expected
- Providing multiple formulas before `proof` without a `|-` turnstile
- An empty sequent (no conclusion formula)
- An unterminated justification argument list (missing `)`)
- Unexpected end of input in the middle of any construct

### ParseError fields

| Field | Type | Meaning |
|---|---|---|
| `message` | `String` | Human-readable description of the failure. |
| `offset` | `Option<usize>` | Byte offset in the source where the failure occurred, if known. |

The `offset` field is `Some(n)` when the **lexer** reports the error (because
the lexer operates on raw byte spans and always knows the position). When the
**parser** reports the error, `offset` is `None` — the parser works on the
token stream, not the raw source, and does not track byte positions.

### ParseError display format

```
parse error at offset <n>: <message>    -- when offset is Some(n)
parse error: <message>                  -- when offset is None
```

The CLI wraps this in a prefix:

```
parse error: parse error at offset <n>: <message>
```

or

```
parse error: parse error: <message>
```

(The outer "parse error: " prefix is added by the CLI's error handler in
`examples/alif.rs`.)

### Common parse errors

The table below lists the most common parse error scenarios, the exact `message`
string produced, and the recommended fix.

| Scenario | `message` | Fix |
|---|---|---|
| Unrecognised character such as `@`, `#`, `!` | `unrecognised token at byte offset <n>: "<char>"` | Remove or replace the character. Only ASCII letters, digits, `_`, and the listed punctuation are valid. |
| Missing `:` after axiom name | `expected \`:\`, got <token> (<text>)` | Add `:` between the axiom/theorem name and its formula. |
| Missing `:` after theorem name | `expected \`:\`, got <token> (<text>)` | Same as above. |
| Missing `:` after `assume` name | `expected \`:\`, got <token> (<text>)` | Add `:` after the hypothesis name. |
| Missing `:=` in `have` step | `expected \`:=\`, got <token> (<text>)` | Add `:=` before the justification. |
| `qed` found where step expected | `expected \`assume\`, \`have\`, or \`exact\`, got Qed` | Remove the stray `qed` or add the missing step keywords. |
| Multiple hypotheses without `\|-` | `multiple formulas before \`proof\` but no \`\|-\` turnstile` | Insert `\|-` between hypotheses and conclusion. |
| Empty sequent | `empty sequent: expected at least a conclusion formula` | Add a conclusion formula after `theorem name:`. |
| Unexpected end of input | `unexpected end of input` | Check for an unclosed block (missing `qed`, missing `)`, etc.). |
| End of input inside justification arg list | `unexpected end of input in justification argument list` | Add the closing `)` to the rule invocation. |
| End of input while parsing a formula | `unexpected end of input while parsing formula` | Complete the formula expression. |
| End of input while parsing sequent | `unexpected end of input while parsing sequent` | Add the missing formula or `proof` block. |
| Token where formula atom expected | `expected a formula atom, got <Token>` | Replace the unexpected token with a valid formula atom (identifier, `(`, `forall`, `exists`, `NOT`). |
| Token where identifier expected | `expected identifier, got <Token> (<text>)` | Replace with a valid identifier. |
| File starts with something other than `axiom` or `theorem` | `expected \`axiom\` or \`theorem\`, got <Token>` | Ensure the file contains only `axiom` and `theorem` top-level declarations. |

### ParseError examples

**Unrecognised character:**

Input:
```
axiom id: A @bad
```

Lexer output:
```
ParseError {
    message: "unrecognised token at byte offset 12: \"@\"",
    offset: None
}
```

CLI stderr:
```
parse error: parse error: unrecognised token at byte offset 12: "@bad"
```

**Missing colon after theorem name:**

Input:
```
theorem my_thm A |- A
proof
  assume h: A
  exact h
qed
```

Parser output:
```
ParseError {
    message: "expected `:`\, got Ident (\"A\")",
    offset: None
}
```

CLI stderr:
```
parse error: parse error: expected `:`\, got Ident ("A")
```

**Missing `:=` in `have`:**

Input:
```
theorem t: A AND B |- A
proof
  assume h: A AND B
  have a: A AndElimLeft(h)
  exact a
qed
```

Parser output:
```
ParseError {
    message: "expected `:=`, got Ident (\"AndElimLeft\")",
    offset: None
}
```

CLI stderr:
```
parse error: parse error: expected `:=`, got Ident ("AndElimLeft")
```

**Unexpected `qed`:**

Input:
```
theorem t: A |- A
proof
qed
qed
```

The second `qed` is outside any theorem block. It is reached by `parse_item`,
which expects `axiom` or `theorem`:

```
ParseError {
    message: "expected `axiom` or `theorem`, got Qed",
    offset: None
}
```

CLI stderr:
```
parse error: parse error: expected `axiom` or `theorem`, got Qed
```

---

## CheckError

### What triggers a CheckError

A `CheckError` is produced by `checker::check` when a theorem's proof steps do
not constitute a valid proof. The checker works through steps sequentially and
halts at the first invalid step. Possible failure conditions are:

- A hypothesis name referenced in a justification is not in the current
  environment scope.
- An axiom name referenced in a justification is neither in scope nor in the
  axiom map.
- A `have` step's computed formula does not match its declared type annotation.
- An `exact` step's computed formula does not match the theorem's conclusion.
- A rule invocation fails (wrong number of premises, wrong shape, etc.).
- The proof has no steps at all.
- The last formula produced by the proof does not equal the conclusion (final
  post-loop check).

### CheckError fields

| Field | Type | Meaning |
|---|---|---|
| `step_index` | `usize` | Zero-based index of the failing step within `theorem.steps`. |
| `step` | `Box<ProofStep>` | A clone of the failing step (boxed to keep `CheckError` small on the stack). |
| `message` | `String` | Human-readable explanation of the failure. |

### CheckError display format

```
proof error at step <step_index>: <message>
```

The `step_index` is zero-based. The first step is step 0.

The CLI wraps this in a prefix:

```
proof error: proof error at step <n>: <message>
```

### Common check errors

| Scenario | `message` string | Fix |
|---|---|---|
| Name not in scope and not an axiom (in `have` or `exact` Axiom justification) | `unknown name \`<name>\`: not in scope and not a known axiom` | Declare the name with `assume` or `have` before using it, or add an `axiom` declaration. |
| Name not in scope (in `have` Axiom justification, inner resolver) | `rule \`<name>\` failed: name \`<name>\` is not in scope and is not a known axiom` | Same fix. |
| Hypothesis not in scope as a rule argument | `rule \`<RuleName>\` failed: hypothesis \`<name>\` is not in scope` | Ensure the argument name was introduced before this step. |
| Unknown rule name | `rule \`<name>\` failed: unknown rule name` | Check spelling and capitalisation. Rule names are PascalCase. |
| Derived formula does not match declared formula | `derived formula \`<derived>\` does not match declared formula \`<declared>\`` | Correct the type annotation on the `have` step to match what the rule actually produces, or fix the rule arguments. |
| `exact` produced wrong formula | `` `exact` produced `<derived>` but the theorem's conclusion is `<conclusion>` `` | Ensure the final `exact` step references a formula equal to the theorem's conclusion. |
| Proof ends on wrong formula (post-loop check) | `proof ends with \`<formula>\` but conclusion is \`<conclusion>\`` | Ensure the last step in the proof establishes the conclusion. |
| Proof has no steps | `proof has no steps` | Add at least one step to the proof body. |
| Rule-specific failure (any of the twelve rules) | `rule \`<RuleName>\` failed: <reason>` | See the [RuleError section](#ruleerror) and the [Inference Rules reference](inference-rules.md). |

### CheckError examples

**Unknown hypothesis name:**

Source:
```
theorem bad:
  A |- A
proof
  exact nonexistent
qed
```

Result:
```
CheckError {
    step_index: 0,
    step: Exact { justification: Axiom("nonexistent") },
    message: "unknown name `nonexistent`: not in scope and not a known axiom"
}
```

CLI stderr:
```
proof error: proof error at step 0: unknown name `nonexistent`: not in scope and not a known axiom
```

**Derived formula does not match declared formula:**

Source:
```
theorem bad:
  A AND B |- B
proof
  assume h: A AND B
  have wrong: A := AndElimRight(h)
  exact wrong
qed
```

`AndElimRight` on `A AND B` produces `B`, not `A`. The annotation says `A`.

Result:
```
CheckError {
    step_index: 1,
    step: Have { name: "wrong", formula: Var("A"), justification: Rule("AndElimRight", ["h"]) },
    message: "derived formula `B` does not match declared formula `A`"
}
```

CLI stderr:
```
proof error: proof error at step 1: derived formula `B` does not match declared formula `A`
```

**`exact` produced wrong formula:**

Source:
```
theorem bad:
  A |- B
proof
  assume h: A
  exact h
qed
```

`h` holds formula `A` but the conclusion is `B`.

Result:
```
CheckError {
    step_index: 1,
    step: Exact { justification: Axiom("h") },
    message: "`exact` produced `A` but the theorem's conclusion is `B`"
}
```

CLI stderr:
```
proof error: proof error at step 1: `exact` produced `A` but the theorem's conclusion is `B`
```

**Empty proof:**

Source:
```
theorem empty:
  A |- A
proof
qed
```

The proof body is empty. The checker's final check finds no last formula.

Result:
```
CheckError {
    step_index: 0,
    step: Exact { justification: Axiom("_") },
    message: "proof has no steps"
}
```

CLI stderr:
```
proof error: proof error at step 0: proof has no steps
```

---

## RuleError

### Fields

| Field | Type | Meaning |
|---|---|---|
| `rule` | `String` | The canonical PascalCase name of the rule that was invoked (e.g. `"AndIntro"`). |
| `reason` | `String` | A human-readable explanation of why the rule could not be applied. |

`RuleError` derives `Clone` and `PartialEq`, making it suitable for use in
assertions and comparisons in test code. It implements `std::error::Error` and
`std::fmt::Display`.

### RuleError display format

```
rule `<rule>` failed: <reason>
```

When a `RuleError` is returned from `apply_rule`, the checker converts it to a
`CheckError` by calling `.to_string()` on it, storing the result as
`CheckError.message`. As a result, the `CheckError.message` for any rule
failure begins with `rule \`RuleName\` failed: ...`.

### All rule error messages

The following table lists every possible `(rule, reason)` pair that can be
produced by `apply_rule`. The `reason` strings are literal; they appear
verbatim in error output.

#### AndIntro

| Condition | `reason` |
|---|---|
| `premises.len() != 2` | `AndIntro requires exactly 2 premises` |

#### AndElimLeft

| Condition | `reason` |
|---|---|
| `premises.len() != 1` | `AndElimLeft requires exactly 1 premise` |
| Premise is not `Formula::And` | `premise must be a conjunction (A AND B)` |

#### AndElimRight

| Condition | `reason` |
|---|---|
| `premises.len() != 1` | `AndElimRight requires exactly 1 premise` |
| Premise is not `Formula::And` | `premise must be a conjunction (A AND B)` |

#### OrIntroLeft

| Condition | `reason` |
|---|---|
| `premises.len() != 2` | `OrIntroLeft requires 2 arguments: the left formula and the right formula` |

#### OrIntroRight

| Condition | `reason` |
|---|---|
| `premises.len() != 2` | `OrIntroRight requires 2 arguments: the left formula and the right formula` |

#### ModusPonens

| Condition | `reason` |
|---|---|
| `premises.len() != 2` | `ModusPonens requires exactly 2 premises: (A => B) and A` |
| First premise is `A => B` but `A ≠ premises[1]` | `second premise does not match the antecedent of the implication` |
| Second premise is `A => B` but `A ≠ premises[0]` | `first premise does not match the antecedent of the implication` |
| Neither premise is `Formula::Implies` | `one premise must be an implication (A => B)` |

#### ImpliesIntro

| Condition | `reason` |
|---|---|
| `premises.len() != 2` | `ImpliesIntro requires 2 premises: the hypothesis A and the conclusion B` |

#### NotElim

| Condition | `reason` |
|---|---|
| `premises.len() != 2` | `NotElim requires exactly 2 premises: A and NOT A (or vice versa)` |
| Premises are not `(F, NOT F)` or `(NOT F, F)` | `premises must be a formula and its negation` |

#### NotIntro

| Condition | `reason` |
|---|---|
| `premises.len() != 1` | `NotIntro requires 1 premise: the formula A to be negated` |

#### ForallElim

| Condition | `reason` |
|---|---|
| `premises.len() != 2` | `ForallElim requires 2 premises: (forall X: F) and the term to substitute` |
| `premises[0]` is not `Formula::Forall` | `first premise must be a universal quantification (forall X: F)` |

#### ForallIntro

| Condition | `reason` |
|---|---|
| `premises.len() != 2` | `ForallIntro requires 2 premises: the variable name (as Var) and the formula` |
| `premises[0]` is not `Formula::Var` | `first premise must be a variable name` |

#### ExistsIntro

| Condition | `reason` |
|---|---|
| `premises.len() != 3` | `ExistsIntro requires 3 premises: the variable, the witness term, and the formula F[X:=t]` |
| `premises[0]` is not `Formula::Var` | `first premise must be a variable name` |

#### Resolver errors (not from `apply_rule` itself)

These are produced by `resolve_rule` in `checker.rs` before `apply_rule` is
ever called:

| Condition | `reason` |
|---|---|
| Rule name string does not match any known rule | `unknown rule name` |
| Argument name not found in environment | `hypothesis \`<name>\` is not in scope` |

### RuleError examples

**Wrong premise count for AndIntro:**

Source:
```
theorem bad:
  A |- A AND A
proof
  assume h: A
  have c: A AND A := AndIntro(h)
  exact c
qed
```

`AndIntro` receives one argument (`h`) but requires two.

`RuleError { rule: "AndIntro", reason: "AndIntro requires exactly 2 premises" }`

Becomes `CheckError.message`:
```
rule `AndIntro` failed: AndIntro requires exactly 2 premises
```

CLI stderr:
```
proof error: proof error at step 1: rule `AndIntro` failed: AndIntro requires exactly 2 premises
```

**Wrong shape for AndElimLeft:**

Source:
```
theorem bad:
  A => B |- A
proof
  assume h: A => B
  have a: A := AndElimLeft(h)
  exact a
qed
```

`h` holds `A => B` which is not a conjunction.

`RuleError { rule: "AndElimLeft", reason: "premise must be a conjunction (A AND B)" }`

CLI stderr:
```
proof error: proof error at step 1: rule `AndElimLeft` failed: premise must be a conjunction (A AND B)
```

**ModusPonens antecedent mismatch:**

Source:
```
theorem bad:
  A => B, C |- B
proof
  assume impl: A => B
  assume hc: C
  have b: B := ModusPonens(impl, hc)
  exact b
qed
```

The implication's antecedent is `A` but the second argument is `C`.

`RuleError { rule: "ModusPonens", reason: "second premise does not match the antecedent of the implication" }`

CLI stderr:
```
proof error: proof error at step 2: rule `ModusPonens` failed: second premise does not match the antecedent of the implication
```

**Unknown rule name:**

Source:
```
theorem bad:
  A, B |- A AND B
proof
  assume ha: A
  assume hb: B
  have c: A AND B := andintro(ha, hb)
  exact c
qed
```

`andintro` (lowercase) is not a recognised rule.

`RuleError { rule: "andintro", reason: "unknown rule name" }`

CLI stderr:
```
proof error: proof error at step 2: rule `andintro` failed: unknown rule name
```

---

## Exit Codes

The Alif CLI (`alif verify <file>` or `alif check <file>`) exits with one of
three codes:

| Code | Meaning | Triggered by |
|---|---|---|
| `0` | All theorems verified successfully. | `verify_source` returns `Ok(())`. Also printed to stdout: `✓ QED`. |
| `1` | Proof error — source parsed successfully but a theorem's proof is invalid. | `verify_source` returns `Err(AlifError::Check(_))`. Prints to stderr: `proof error: <CheckError>`. |
| `2` | Parse error or usage error — source could not be parsed, or the CLI was invoked incorrectly. | `verify_source` returns `Err(AlifError::Parse(_))`, or the file cannot be read, or wrong number of CLI arguments, or unknown command. Prints to stderr: `parse error: <ParseError>` or the relevant usage/IO message. |

The `alif_verify` C FFI function uses the same codes (0, 1, 2) with the same
semantics, plus `2` for a null pointer or invalid UTF-8 input.

**Design rationale:** exit code `2` is used for both parse errors and usage
errors so that callers can distinguish "the proof itself is wrong" (code 1)
from "the input could not even be understood" (code 2) from "everything is
fine" (code 0).

---

## Complete stderr output examples

The following examples show exactly what appears on stderr for each error
category when running the CLI.

### Successful verification (no stderr, exit 0)

```
$ alif verify examples/and_comm.alif
✓ QED
```

No stderr. Exit code 0.

---

### Parse error: unrecognised character (exit 2)

Input file `bad.alif`:
```
axiom id: A @oops
```

```
$ alif verify bad.alif
parse error: parse error: unrecognised token at byte offset 12: "@oops"
```

Exit code 2.

---

### Parse error: missing `:` (exit 2)

Input file `bad.alif`:
```
theorem my_thm A |- A
proof
  assume h: A
  exact h
qed
```

```
$ alif verify bad.alif
parse error: parse error: expected `:`, got Ident ("A")
```

Exit code 2.

---

### Check error: unknown name (exit 1)

Input file `bad.alif`:
```
theorem t:
  A |- A
proof
  exact ghost
qed
```

```
$ alif verify bad.alif
proof error: proof error at step 0: unknown name `ghost`: not in scope and not a known axiom
```

Exit code 1.

---

### Check error: formula mismatch in `have` (exit 1)

Input file `bad.alif`:
```
theorem t:
  A AND B |- A
proof
  assume h: A AND B
  have wrong: B := AndElimLeft(h)
  exact wrong
qed
```

`AndElimLeft` on `A AND B` produces `A`, but the annotation says `B`.

```
$ alif verify bad.alif
proof error: proof error at step 1: derived formula `A` does not match declared formula `B`
```

Exit code 1.

---

### Check error: wrong `exact` conclusion (exit 1)

Input file `bad.alif`:
```
theorem t:
  A |- B
proof
  assume h: A
  exact h
qed
```

```
$ alif verify bad.alif
proof error: proof error at step 1: `exact` produced `A` but the theorem's conclusion is `B`
```

Exit code 1.

---

### Check error: empty proof (exit 1)

Input file `bad.alif`:
```
theorem t:
  A |- A
proof
qed
```

```
$ alif verify bad.alif
proof error: proof error at step 0: proof has no steps
```

Exit code 1.

---

### Check error: rule failure — wrong arity (exit 1)

Input file `bad.alif`:
```
theorem t:
  A |- A AND A
proof
  assume h: A
  have c: A AND A := AndIntro(h)
  exact c
qed
```

```
$ alif verify bad.alif
proof error: proof error at step 1: rule `AndIntro` failed: AndIntro requires exactly 2 premises
```

Exit code 1.

---

### Check error: rule failure — wrong premise shape (exit 1)

Input file `bad.alif`:
```
theorem t:
  A |- A
proof
  assume h: A
  have bad: A := AndElimLeft(h)
  exact bad
qed
```

```
$ alif verify bad.alif
proof error: proof error at step 1: rule `AndElimLeft` failed: premise must be a conjunction (A AND B)
```

Exit code 1.

---

### Usage error: wrong number of arguments (exit 2)

```
$ alif verify
usage: alif verify <file.alif>
       alif check  <file.alif>
```

Exit code 2.

---

### Usage error: unknown command (exit 2)

```
$ alif run myproof.alif
unknown command `run`. Use `verify` or `check`.
```

Exit code 2.

---

### IO error: file not found (exit 2)

```
$ alif verify nonexistent.alif
error reading `nonexistent.alif`: No such file or directory (os error 2)
```

Exit code 2.
