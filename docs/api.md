# Alif Library — Rust API Reference

This document is the complete reference for the `alif` crate's public Rust API.
It covers every exported function, type, and module, with field-level
descriptions, `Display` output formats, derive information, and runnable usage
examples.

---

## Adding `alif` as a dependency

`alif` is built as both an `rlib` (Rust library) and a `cdylib` (C-compatible
shared library).  To use it from another Rust project, add it to your
`Cargo.toml` as a path or registry dependency:

```toml
[dependencies]
alif = { path = "../alif" }
```

All public items are re-exported from the crate root, so a single `use alif::*`
is sufficient for most use cases.  More focused imports are also fine:

```rust
use alif::{verify_source, AlifError, CheckError, ParseError};
use alif::term::{Formula, Item, ProofStep, Justification, Theorem};
use alif::rules::{Rule, apply_rule, substitute};
use alif::stdlib::load_stdlib;
```

---

## Crate layout

| Module | Responsibility |
|---|---|
| `alif` (crate root, `src/lib.rs`) | Re-exports, `verify_source`, FFI entry point |
| `alif::term` | Core AST types: `Formula`, `ProofStep`, `Justification`, `Theorem`, `Item` |
| `alif::lexer` | Tokenizer (`logos` 0.14), `Token` enum, `lex` function |
| `alif::parser` | Recursive-descent parser, `parse_source` |
| `alif::checker` | Proof checker, `check` |
| `alif::rules` | Inference rules, `Rule` enum, `apply_rule`, `substitute` |
| `alif::stdlib` | Embedded standard axiom library, `load_stdlib` |
| `alif::error` | Error types: `AlifError`, `CheckError`, `ParseError`, `RuleError` |

---

## Top-level functions

### `verify_source`

```rust
pub fn verify_source(source: &str) -> Result<(), AlifError>
```

Verify all theorems in an Alif source string.  This is the primary entry point
for the library and the function called by the CLI.

**What it does, step by step:**

1. **Lex and parse** — calls `parse_source(source)`, which internally runs the
   `logos`-based lexer followed by the recursive-descent parser.  Any lexer or
   grammar error is wrapped in `AlifError::Parse` and returned immediately.

2. **Load the standard library** — calls `stdlib::load_stdlib()`, which parses
   the axiom file embedded at compile time via `include_str!`.  The result is a
   `HashMap<String, Formula>` that maps axiom names such as `"identity"` and
   `"and_comm"` to their formulas.  A `ParseError` here is also propagated as
   `AlifError::Parse` (via the `From` impl).

3. **Process items in declaration order** — iterates over the `Vec<Item>`
   returned by the parser:
   - `Item::Axiom { name, formula }` — inserts the axiom into the running
     `HashMap` so it is visible to all subsequent theorems.
   - `Item::Theorem(thm)` — calls `checker::check(&thm, &axioms)`.  On failure
     the `CheckError` is propagated as `AlifError::Check`.

4. **Return** — if every theorem passes, returns `Ok(())`.

**Error cases:**

| Situation | Error variant |
|---|---|
| Unrecognised character in source | `AlifError::Parse` |
| Grammar error (e.g. missing `qed`) | `AlifError::Parse` |
| Stdlib source is malformed (build-time bug) | `AlifError::Parse` |
| A `have` step derives the wrong formula | `AlifError::Check` |
| An `exact` step does not match the conclusion | `AlifError::Check` |
| An unknown hypothesis name is referenced | `AlifError::Check` |
| Rule applied with wrong arity or wrong premise shape | `AlifError::Check` |

---

### `parse_source`

```rust
pub fn parse_source(source: &str) -> Result<Vec<Item>, ParseError>
```

Re-exported from `alif::parser`.  Lex and parse an Alif source string, returning
the sequence of top-level items (axiom declarations and theorem blocks).

The function first calls `lexer::lex(source)`.  Any lexer error — an
unrecognised byte in the source — is converted to a `ParseError`.  Then a
`Parser` struct is initialised with the token stream.  Items are parsed
one-by-one with `parse_item()` until the token stream is exhausted.  Any
grammar error stops parsing immediately and returns a `ParseError`.

The returned `Vec<Item>` preserves declaration order, which matters because
axioms declared earlier in the file are visible to theorems declared later.

**Error cases:**

- `ParseError` if any character in `source` is not part of a valid Alif token.
- `ParseError` if the grammar is violated (wrong keyword order, missing
  punctuation, empty sequent, etc.).

---

### `check`

```rust
pub fn check(
    theorem: &Theorem,
    axioms: &HashMap<String, Formula>,
) -> Result<(), CheckError>
```

Re-exported from `alif::checker`.  Verify that the proof steps in `theorem` are
logically valid given the named axioms in `axioms`.

The checker maintains an *environment* (`HashMap<String, Formula>`) that grows
as `Assume` and `Have` steps are processed.  For each step:

- **`Assume { name, formula }`** — inserts `(name, formula)` into the
  environment unconditionally.  No justification is required; `assume` is the
  mechanism for introducing hypotheses.
- **`Have { name, formula, justification }`** — resolves the justification to a
  derived formula, checks that the derived formula equals the declared `formula`
  (syntactic equality via `PartialEq`), then inserts `(name, formula)` into the
  environment.
- **`Exact { justification }`** — resolves the justification to a formula,
  checks that it equals `theorem.conclusion`, then records it as the last formula.

After all steps are processed, a final check confirms that the last formula
equals the conclusion.  This double-check exists because `exact` is the intended
terminator, but a proof consisting solely of `assume` steps (which would be
vacuously accepted by the step loop) is caught here.

**Resolving a `Justification`:**

- `Justification::Axiom(name)` — look up `name` first in the current environment,
  then in `axioms`.  If not found, return a `CheckError`.
- `Justification::Rule(rule_name, args)` — call `Rule::from_name(rule_name)`,
  then look up each argument name in the environment, collect the formulas as
  premises, and call `apply_rule`.

**Error cases:**

- `CheckError` if a name is not in scope and not in `axioms`.
- `CheckError` if `apply_rule` fails (wrong arity, wrong premise shape).
- `CheckError` if the derived formula does not match the declared formula in
  a `have` step.
- `CheckError` if the final formula does not match the conclusion.
- `CheckError` if the proof has no steps.

---

## Types

### `Formula`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Formula { … }
```

Located in `alif::term`.  Represents a logical formula in propositional or
first-order logic.  The type is recursive; sub-formulas are heap-allocated with
`Box<Formula>`.

**Derives:** `Debug`, `Clone`, `PartialEq`.
**Display:** implemented; see the format column below.

| Variant | Fields | Meaning | `Display` output |
|---|---|---|---|
| `Var(String)` | inner name string | Propositional variable or predicate atom, e.g. `A`, `human(socrates)` | `A`, `human(socrates)` |
| `And(Box<Formula>, Box<Formula>)` | left, right | Conjunction `A AND B` | `(A AND B)` |
| `Or(Box<Formula>, Box<Formula>)` | left, right | Disjunction `A OR B` | `(A OR B)` |
| `Not(Box<Formula>)` | inner | Negation `NOT A` | `NOT A` |
| `Implies(Box<Formula>, Box<Formula>)` | antecedent, consequent | Implication `A => B` | `(A => B)` |
| `Forall(String, Box<Formula>)` | variable name, body | Universal quantification `forall X: F` | `forall X: F` |
| `Exists(String, Box<Formula>)` | variable name, body | Existential quantification `exists X: F` | `exists X: F` |

`Display` adds parentheses around binary connectives to make the structure
unambiguous.  `Var` and the prefix/quantifier forms are not parenthesised.

**Usage example:**

```rust
use alif::term::Formula;

let a = Formula::Var("A".to_string());
let b = Formula::Var("B".to_string());
let conj = Formula::And(Box::new(a.clone()), Box::new(b.clone()));
println!("{}", conj); // prints: (A AND B)

let imp = Formula::Implies(Box::new(a.clone()), Box::new(b.clone()));
println!("{}", imp);  // prints: (A => B)

let forall_x = Formula::Forall(
    "X".to_string(),
    Box::new(Formula::Var("X".to_string())),
);
println!("{}", forall_x); // prints: forall X: X
```

---

### `ProofStep`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ProofStep { … }
```

Located in `alif::term`.  Represents a single step in a proof block.

**Derives:** `Debug`, `Clone`, `PartialEq`.

| Variant | Fields | Meaning |
|---|---|---|
| `Assume { name: String, formula: Formula }` | `name` — identifier to bind in scope; `formula` — the formula asserted without justification | Introduces a hypothesis |
| `Have { name: String, formula: Formula, justification: Justification }` | `name` — identifier; `formula` — expected derived formula; `justification` — how it is derived | Derives a new formula from existing ones |
| `Exact { justification: Justification }` | `justification` — final justification | Concludes the proof |

**Usage example:**

```rust
use alif::term::{Formula, Justification, ProofStep};

let step = ProofStep::Have {
    name: "ab".to_string(),
    formula: Formula::And(
        Box::new(Formula::Var("A".to_string())),
        Box::new(Formula::Var("B".to_string())),
    ),
    justification: Justification::Rule(
        "AndIntro".to_string(),
        vec!["ha".to_string(), "hb".to_string()],
    ),
};
```

---

### `Justification`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Justification { … }
```

Located in `alif::term`.  Records how a step in the proof is justified.

**Derives:** `Debug`, `Clone`, `PartialEq`.

| Variant | Fields | Meaning |
|---|---|---|
| `Axiom(String)` | name string | A bare name reference; resolved first in the local environment, then in the global axiom map |
| `Rule(String, Vec<String>)` | rule name, list of argument names | Application of a named inference rule to hypothesis names already in scope |

`Justification::Axiom` is used both for actual axiom references (e.g.
`exact identity`) and for direct hypothesis references (e.g. `exact h`).  The
checker resolves the name against the environment first, then falls back to the
axiom map, so there is no ambiguity.

---

### `Theorem`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Theorem { … }
```

Located in `alif::term`.  Holds the complete representation of a theorem
declaration together with its proof.

**Derives:** `Debug`, `Clone`, `PartialEq`.

| Field | Type | Description |
|---|---|---|
| `name` | `String` | The theorem's identifier as written in the source |
| `hypotheses` | `Vec<Formula>` | The antecedents of the sequent (left side of `\|-`); empty if the theorem is stated without a sequent |
| `conclusion` | `Formula` | The formula to be proved (right side of `\|-`, or the sole formula if no `\|-` is present) |
| `steps` | `Vec<ProofStep>` | The proof steps in source order, from `proof` to `qed` |

**Usage example:**

```rust
use alif::term::{Formula, Justification, ProofStep, Theorem};
use alif::checker::check;
use std::collections::HashMap;

let thm = Theorem {
    name: "identity".to_string(),
    hypotheses: vec![Formula::Var("A".to_string())],
    conclusion: Formula::Var("A".to_string()),
    steps: vec![
        ProofStep::Assume {
            name: "h".to_string(),
            formula: Formula::Var("A".to_string()),
        },
        ProofStep::Exact {
            justification: Justification::Axiom("h".to_string()),
        },
    ],
};

assert!(check(&thm, &HashMap::new()).is_ok());
```

---

### `Item`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Item { … }
```

Located in `alif::term`.  Represents a single top-level declaration in an Alif
source file.  `parse_source` returns `Vec<Item>`.

**Derives:** `Debug`, `Clone`, `PartialEq`.

| Variant | Fields | Meaning |
|---|---|---|
| `Axiom { name: String, formula: Formula }` | `name` — axiom identifier; `formula` — the declared formula | An `axiom` declaration |
| `Theorem(Theorem)` | inner `Theorem` struct | A `theorem … proof … qed` block |

---

### `Rule`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Rule { … }
```

Located in `alif::rules`.  Enumerates all inference rules built into the Alif
proof system.

**Derives:** `Debug`, `Clone`, `PartialEq`.

| Variant | Premises | Conclusion | Notes |
|---|---|---|---|
| `AndIntro` | `A`, `B` | `A AND B` | Requires exactly 2 premises |
| `AndElimLeft` | `A AND B` | `A` | Requires exactly 1 premise; premise must be `And` |
| `AndElimRight` | `A AND B` | `B` | Requires exactly 1 premise; premise must be `And` |
| `OrIntroLeft` | `A`, `B` | `A OR B` | Requires exactly 2 premises; left injection |
| `OrIntroRight` | `B`, `A` | `A OR B` | Requires exactly 2 premises; right injection — note argument order is reversed |
| `ModusPonens` | `A => B`, `A` | `B` | Requires exactly 2 premises; tries both orderings |
| `ImpliesIntro` | `A`, `B` | `A => B` | Requires exactly 2 premises |
| `NotIntro` | `A` | `NOT A` | Requires exactly 1 premise |
| `NotElim` | `A`, `NOT A` | `⊥` | Requires exactly 2 premises; either order accepted; result is `Var("⊥")` |
| `ForallElim` | `forall X: F`, `t` | `F[X := t]` | Requires exactly 2 premises; substitutes `t` for `X` in body |
| `ForallIntro` | `X` (as `Var`), `F` | `forall X: F` | Requires exactly 2 premises; first must be a `Var` |
| `ExistsIntro` | `X` (as `Var`), `t`, `F[X:=t]` | `exists X: F` | Requires exactly 3 premises; first must be a `Var`; the third premise becomes the body |

**`Rule::from_name`**

```rust
pub fn from_name(name: &str) -> Option<Rule>
```

Parse a rule name string into a `Rule` variant.  Returns `None` if the name
does not correspond to any known rule.  Used by the checker's
`resolve_rule` helper.

```rust
assert_eq!(Rule::from_name("AndIntro"), Some(Rule::AndIntro));
assert_eq!(Rule::from_name("Bogus"),    None);
```

**`Rule::name`**

```rust
pub fn name(&self) -> &'static str
```

Return the canonical ASCII name of this rule as a static string slice.  The
names round-trip through `from_name`:

```rust
let r = Rule::ModusPonens;
assert_eq!(Rule::from_name(r.name()), Some(r));
```

---

## Error types

### `AlifError`

```rust
#[derive(Debug)]
pub enum AlifError {
    Parse(ParseError),
    Check(CheckError),
}
```

Top-level error type returned by `verify_source`.

**`Display`:**

Delegates to the inner error's `Display` implementation, so the output is
identical to printing the inner `ParseError` or `CheckError` directly.

**`Error` trait:** implemented.  `source()` returns `Some` of the inner error.

**`From` impls:**

```rust
impl From<ParseError> for AlifError   // AlifError::Parse(e)
impl From<CheckError> for AlifError   // AlifError::Check(e)
```

These `From` impls allow the `?` operator to propagate both error types from
within `verify_source` without explicit wrapping.

---

### `CheckError`

```rust
#[derive(Debug)]
pub struct CheckError {
    pub step_index: usize,
    pub step: Box<ProofStep>,
    pub message: String,
}
```

Produced when `checker::check` finds an invalid proof step.

| Field | Type | Description |
|---|---|---|
| `step_index` | `usize` | Zero-based index of the failing step within `theorem.steps` |
| `step` | `Box<ProofStep>` | A clone of the failing step, boxed to keep the `CheckError` struct small (satisfies the `clippy::result_large_err` lint) |
| `message` | `String` | Human-readable explanation of the failure |

**`Display`:** `"proof error at step {step_index}: {message}"`

**`Error` trait:** implemented (no source).

---

### `ParseError`

```rust
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub offset: Option<usize>,
}
```

Produced by the lexer or parser when the source text is malformed.

| Field | Type | Description |
|---|---|---|
| `message` | `String` | Human-readable description of the failure |
| `offset` | `Option<usize>` | Byte offset in the source where the error occurred, or `None` if not available (grammar errors do not always carry an offset) |

**`Display`:**
- With offset: `"parse error at offset {offset}: {message}"`
- Without offset: `"parse error: {message}"`

**`Error` trait:** implemented (no source).

---

### `RuleError`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct RuleError {
    pub rule: String,
    pub reason: String,
}
```

Produced by `rules::apply_rule` when premises do not satisfy a rule's
requirements.  `RuleError` values are converted to `CheckError` inside
`checker::check` and are not exposed through `AlifError` directly.

| Field | Type | Description |
|---|---|---|
| `rule` | `String` | The name of the rule that failed |
| `reason` | `String` | Why the rule could not be applied |

**`Display`:** `"rule '{rule}' failed: {reason}"`

**`Error` trait:** implemented.

---

## Modules

### `alif::checker`

Implements the core proof-checking algorithm.  Exports the `check` function
(re-exported at the crate root).  Internally maintains an environment
`HashMap<String, Formula>` that accumulates named formulas as the step list is
traversed.  The private helpers `resolve_justification` and `resolve_rule`
separate name-lookup logic from the main step dispatch.  The private function
`formulas_match` performs plain structural equality (`a == b`) with no
alpha-equivalence or unification.

### `alif::error`

Defines all four error types: `AlifError`, `CheckError`, `ParseError`, and
`RuleError`.  Implements `std::fmt::Display` and `std::error::Error` for each.
Also provides `From<ParseError> for AlifError` and `From<CheckError> for
AlifError` to support the `?` operator in `verify_source`.

### `alif::lexer`

Implements the tokenizer using the `logos` 0.14 crate.  The `Token` enum is
annotated with `#[derive(Logos)]`; whitespace and line comments (`-- …`) are
declared as skip rules.  Multi-character tokens (`|-`, `=>`, `:=`) are listed
before single-character tokens so the longest match wins.  Keyword tokens are
declared before `Ident` so `logos` matches them with priority.  The public `lex`
function returns `Vec<(Token, String)>` — each pair holds the token variant and
the owned source slice, which the parser needs to recover identifier text.

### `alif::parser`

Implements a hand-rolled recursive-descent parser.  The private `Parser` struct
wraps the token stream with a `pos: usize` cursor and three primitive methods:
`peek`, `advance`, and `expect`.  Operator precedence for formulas is encoded
directly in the call hierarchy: `parse_implies_expr` calls `parse_or_expr` which
calls `parse_and_expr` which calls `parse_not_expr` which calls `parse_atom`.
Sequent parsing (`parse_sequent`) handles both the `H1, H2 |- C` form and the
bare `C` form.  Justification parsing (`parse_justification`) distinguishes
between a bare name (`Axiom`) and a rule application (`Rule`) by checking for a
`(` immediately after the identifier.  The chumsky dependency in `Cargo.toml` is
reserved for future use but is not used by this module.

### `alif::rules`

Provides the complete set of inference rules.  `apply_rule` is a pure function:
it takes a `&Rule` and a `&[Formula]` slice and returns a `Result<Formula,
RuleError>`.  The `substitute` function performs capture-avoiding substitution
for free occurrences of a variable; it stops recursing into a quantifier that
binds the same variable name (syntactic capture avoidance — no renaming is
performed).

### `alif::stdlib`

Embeds the file `stdlib/logic.alif` into the binary at compile time using
`include_str!("../stdlib/logic.alif")`.  The `load_stdlib` function parses this
constant string on every call to `verify_source`.  The current standard library
defines four axioms: `identity`, `and_comm`, `or_comm`, and `ex_falso`.

### `alif::term`

Defines the core AST data types: `Formula`, `ProofStep`, `Justification`,
`Theorem`, and `Item`.  All types derive `Debug`, `Clone`, and `PartialEq`.
`Formula` additionally implements `std::fmt::Display`.  No methods other than
trait impls are defined in this module; behaviour lives in `checker` and `rules`.

---

## `stdlib::load_stdlib`

```rust
pub fn load_stdlib() -> Result<HashMap<String, Formula>, ParseError>
```

Located in `alif::stdlib`.  Parse the embedded standard axiom library and return
a map from axiom names to their `Formula` values.

The stdlib source is the compile-time constant `STDLIB_SOURCE`, embedded via
`include_str!`.  The function calls `parse_source(STDLIB_SOURCE)`, iterates the
resulting `Item` list, and collects only the `Item::Axiom` variants.  Any
`Item::Theorem` in the stdlib is silently ignored (the current stdlib contains
none).

A `ParseError` returned from this function indicates a malformed `stdlib/logic.alif`
— this is a build-time defect rather than a user error.

Currently registered axioms:

| Name | Formula |
|---|---|
| `identity` | `A` |
| `and_comm` | `A AND B` |
| `or_comm` | `A OR B` |
| `ex_falso` | `NOT A` |

---

## `rules::apply_rule`

```rust
pub fn apply_rule(rule: &Rule, premises: &[Formula]) -> Result<Formula, RuleError>
```

Apply `rule` to `premises` and return the derived formula.

Each rule enforces its own arity and structural constraints on the premises and
returns a `RuleError` if they are not met.  The function is pure and stateless:
it does not consult any environment or axiom map.  The checker is responsible for
resolving hypothesis names to `Formula` values before calling `apply_rule`.

See the `Rule` enum table above for the precise requirements of each rule.

---

## `rules::substitute`

```rust
pub fn substitute(formula: &Formula, var: &str, term: &Formula) -> Formula
```

Substitute all free occurrences of variable `var` with `term` inside `formula`.
Returns a new `Formula`; the input is not modified.

Traversal is structural:
- `Var(name)` — if `name == var`, return `term.clone()`; otherwise return the
  original.
- `And`, `Or`, `Not`, `Implies` — recurse into sub-formulas.
- `Forall(x, body)` / `Exists(x, body)` — if `x == var`, the variable is bound
  here; do not substitute inside `body` (capture avoidance).  Otherwise recurse
  into `body`.

No renaming of bound variables is performed.  If the term being substituted
contains a variable that clashes with a bound variable in the formula, the
substitution will be semantically incorrect (variable capture).  Alif's proof
system currently relies on the user's proofs being written without such clashes.

---

## Complete usage example

```rust
use std::fs;
use alif::{verify_source, AlifError};

fn main() {
    let source = match fs::read_to_string("my_proof.alif") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not read file: {}", e);
            std::process::exit(2);
        }
    };

    match verify_source(&source) {
        Ok(()) => {
            println!("✓ QED");
        }
        Err(AlifError::Parse(e)) => {
            // e.message contains a human-readable description
            // e.offset contains the byte position, if available
            eprintln!("parse error: {}", e);
            std::process::exit(2);
        }
        Err(AlifError::Check(e)) => {
            // e.step_index is zero-based
            // e.step is a Box<ProofStep> clone of the failing step
            // e.message describes the failure
            eprintln!("proof error at step {}: {}", e.step_index, e.message);
            eprintln!("failing step: {:?}", e.step);
            std::process::exit(1);
        }
    }
}
```

For programmatic use of the AST (e.g. building a proof editor or a tactic
layer), call `parse_source` directly to obtain the `Vec<Item>` and inspect or
transform the tree before passing individual `Theorem` values to `check`:

```rust
use alif::{parse_source, AlifError};
use alif::checker::check;
use alif::stdlib::load_stdlib;
use alif::term::Item;

fn check_all(source: &str) -> Result<(), AlifError> {
    let items = parse_source(source)?;
    let mut axioms = load_stdlib()?;

    for item in &items {
        match item {
            Item::Axiom { name, formula } => {
                axioms.insert(name.clone(), formula.clone());
            }
            Item::Theorem(thm) => {
                println!("checking theorem `{}`…", thm.name);
                check(thm, &axioms)?;
                println!("  ok ({} steps)", thm.steps.len());
            }
        }
    }
    Ok(())
}
```
