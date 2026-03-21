# Alif — Internal Architecture

This document describes the internal design of the Alif formal proof verifier:
how the source text is transformed into a verified proof, what data structures
are used at each stage, and why certain design decisions were made.

---

## Pipeline overview

```
source text  (&str)
      │
      ▼
  [Lexer]  src/lexer.rs
      │  Vec<(Token, String)>
      ▼
  [Parser]  src/parser.rs
      │  Vec<Item>
      │       ├── Item::Axiom { name, formula }
      │       └── Item::Theorem(Theorem { name, hypotheses, conclusion, steps })
      ▼
  [Stdlib]  src/stdlib.rs
      │  HashMap<String, Formula>   (standard axioms merged with user axioms)
      ▼
  [Checker]  src/checker.rs
      │  Result<(), CheckError>
      ▼
  ✓ QED   or   ✗ proof error at step N: …
```

The entry point `verify_source` (in `src/lib.rs`) orchestrates all four stages.
Each stage is independent of the others and can be called directly; the pipeline
is not fused or interleaved.

---

## Lexer — `src/lexer.rs`

### Overview

The lexer converts raw source text into a flat sequence of `(Token, String)`
pairs.  The `String` in each pair is the exact source slice that produced the
token, which the parser needs to recover identifier text.

### Token enum

```rust
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"--[^\n]*")]
pub enum Token { … }
```

The `Token` enum is the single source of truth for the lexical grammar.  It
contains 21 variants grouped into four categories:

| Category | Variants |
|---|---|
| Keywords | `Axiom`, `Theorem`, `Proof`, `Assume`, `Have`, `Exact`, `Qed`, `Forall`, `Exists` |
| Logical connectives | `And`, `Or`, `Not` |
| Multi-character punctuation | `Turnstile` (`\|-`), `Implies` (`=>`), `ColonEq` (`:=`) |
| Single-character punctuation | `Colon`, `Comma`, `LParen`, `RParen` |
| Identifiers | `Ident` |

### Skip rules

Two `#[logos(skip …)]` attributes are declared at the enum level:

- `r"[ \t\r\n]+"` — skips whitespace (spaces, tabs, carriage returns, newlines).
- `r"--[^\n]*"` — skips line comments from `--` to end-of-line.

These patterns are applied between every token match; no explicit whitespace or
comment tokens are produced.

### Priority and two-character tokens

`logos` 0.14 resolves conflicts between overlapping patterns by a combination of
literal length and declaration order.  The multi-character tokens (`|-`, `=>`,
`:=`) are declared **before** their single-character prefixes (`:`, `|` is not a
token but if it were, the same rule applies).  This ensures that `|-` is matched
as a `Turnstile` and not as an error followed by `-`.  Similarly, keywords are
declared before `Ident` so that `axiom`, `AND`, etc. are matched as keyword
tokens rather than identifiers, even though they satisfy the identifier regex
`[A-Za-z_][A-Za-z0-9_]*`.

The `lex` function iterates the `logos` spanned iterator, collects `Ok` tokens
into a `Vec`, and returns an error string on the first unrecognised character.

---

## Parser — `src/parser.rs`

### Design

The parser is a hand-rolled recursive-descent parser.  It does not use
`chumsky`, `pest`, or any other parser-combinator library.  The design was
chosen for simplicity and debuggability: the entire parser fits in one file,
there are no combinators to trace through, and error messages can be tailored
precisely.

### `Parser` struct

```rust
struct Parser {
    tokens: Vec<(Token, String)>,
    pos: usize,
}
```

`tokens` is the complete token stream produced by the lexer.  `pos` is the
current cursor position (a zero-based index).  Three primitive methods drive
every production:

- `peek()` — return the token at `pos` without advancing.
- `advance()` — increment `pos`.
- `expect(expected, desc)` — if the current token matches `expected`, advance and
  return `Ok(())`; otherwise return a `ParseError` with a human-readable
  description.
- `expect_ident()` — specialisation of `expect` that returns the identifier
  string.

### Operator precedence

Formula parsing uses the standard descent-by-precedence technique.  Each
precedence level is a separate method that calls the next-higher-precedence
method:

```
parse_formula
  └─ parse_implies_expr    (=>)    left-associative loop
       └─ parse_or_expr    (OR)    left-associative loop
            └─ parse_and_expr (AND)  left-associative loop
                 └─ parse_not_expr  (NOT)  right-recursive
                      └─ parse_atom
```

`parse_not_expr` recurses into itself to handle `NOT NOT A` correctly.
`parse_atom` handles `forall`, `exists`, parenthesised sub-expressions, and
identifiers.

### Sequent parsing

`parse_sequent` collects formulas separated by commas until it sees a
`Turnstile` (`|-`) or the `Proof` keyword.  If a `Turnstile` is found, the
formulas collected so far become the hypotheses and the formula after `|-`
becomes the conclusion.  If no `Turnstile` is found and exactly one formula was
collected, it becomes the conclusion with an empty hypothesis list (the
no-sequent shorthand).

### Justification parsing

`parse_justification` reads one identifier.  If the next token is `(`, it parses
a comma-separated list of identifier arguments until `)` and constructs
`Justification::Rule(name, args)`.  Otherwise it constructs
`Justification::Axiom(name)`.  This single lookahead character is sufficient to
distinguish the two cases.

### Predicate application syntax

In `parse_atom`, after consuming an identifier, the parser checks for a `(`
immediately following.  If one is found, it reads comma-separated identifier
arguments and joins them into the flat string `"f(x,y)"` using
`format!("{}({})", name, args.join(","))`.  This string is stored as a
`Formula::Var`.  There is no separate predicate node in the AST.

---

## Term representation — `src/term.rs`

### `Formula` enum design

```rust
pub enum Formula {
    Var(String),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Not(Box<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    Forall(String, Box<Formula>),
    Exists(String, Box<Formula>),
}
```

`Var(String)` is used for both plain propositional variables (`A`, `B`) and
predicate atoms with arguments (`human(socrates)`, `rel(x,y)`).  The argument
list is flattened into the string at parse time, which means the verifier treats
`human(socrates)` as an opaque atom — exactly the same as any other `Var`.

`Box<Formula>` is used wherever a `Formula` appears recursively (both children of
binary connectives, the body of unary `Not`, and the bodies of quantifiers).
Without boxing, `Formula` would have infinite size.

`String` is used for the bound variable name in `Forall` and `Exists`, and as
the only payload of `Var`.  All heap allocations are therefore either `Box` or
`String`.

### Substitution algorithm

`rules::substitute(formula, var, term)` performs structural recursion.  The
interesting cases are the quantifier arms:

```rust
Formula::Forall(x, body) => {
    if x == var { formula.clone() }    // bound: stop here
    else { Formula::Forall(x.clone(), Box::new(substitute(body, var, term))) }
}
```

When the quantifier binds the same name as `var`, the variable is shadowed and
no substitution occurs in the body.  This is syntactic capture avoidance: it
prevents the substitution from reaching into a scope where `var` has a different
binding.  However, it does not rename bound variables to avoid capture of free
variables in `term`.  The user is responsible for writing proofs that do not
exhibit variable capture.

---

## Checker — `src/checker.rs`

### Environment

The checker maintains a local `HashMap<String, Formula>` called the environment.
It is initialised empty.  As proof steps are processed:

- `Assume { name, formula }` inserts `(name, formula)` unconditionally.
- `Have { name, formula, … }` inserts `(name, formula)` after verifying the
  derived result matches `formula`.

The declared theorem hypotheses are **not** pre-loaded into the environment
automatically.  The proof author must introduce them with explicit `assume`
steps.  (Note: the checker does pre-insert `_hyp0`, `_hyp1`, … keys for the
hypotheses, but these synthetic names are not accessible from the proof syntax;
they serve as a foothold for future features.)

### Step-by-step walk

```rust
for (idx, step) in theorem.steps.iter().enumerate() {
    let make_err = |msg: String| CheckError { step_index: idx, step: Box::new(step.clone()), message: msg };
    match step { … }
}
```

The `make_err` closure captures `idx` and a clone of `step`, so every error
automatically carries the correct step index and a copy of the offending step.
This pattern means the `CheckError` construction is never duplicated.

### `Exact` step

When processing `Exact { justification }`, the checker resolves the justification
to a formula and then tests:

```rust
if !formulas_match(&derived, &theorem.conclusion) { return Err(…); }
```

`formulas_match` is simply `a == b` — structural equality via `PartialEq`.

### Final formula check

After the loop, the checker performs a second equality test:

```rust
match &last_formula {
    Some(f) if formulas_match(f, &theorem.conclusion) => Ok(()),
    Some(f) => Err(…),
    None => Err(CheckError { … message: "proof has no steps" … }),
}
```

This catches proofs that consist only of `assume` steps (the `last_formula`
would be the assumed formula, not necessarily the conclusion) and proofs with an
empty step list.

### `resolve_justification` and `resolve_rule`

Two private helpers separate concerns:

- `resolve_justification(justification, env, axioms)` — dispatches on the
  `Justification` variant.  For `Axiom(name)`, looks up `name` in `env` first,
  then in `axioms`.  For `Rule(rule_name, args)`, delegates to `resolve_rule`.

- `resolve_rule(rule_name, args, env)` — calls `Rule::from_name` to parse the
  rule name, looks up each argument name in `env`, collects the resulting
  `Formula` values as a `Vec`, then calls `apply_rule`.

Both helpers return `RuleError` on failure; the calling code in `check`
converts `RuleError` to `CheckError` via `.map_err(|e| make_err(e.to_string()))`.

---

## Rules — `src/rules.rs`

### `Rule` enum

The `Rule` enum is a plain enum with no payload; all rule logic lives in
`apply_rule`.  This keeps the enum lean and makes exhaustive matching in
`apply_rule` easy to audit.

### `apply_rule` dispatch

`apply_rule` is a single `match` over all 12 `Rule` variants.  Each arm:

1. Checks the arity of `premises` and returns `RuleError` if wrong.
2. Destructs the premise formula(s) to verify the structural requirement (e.g.
   `AndElimLeft` requires the premise to be `Formula::And`).
3. Constructs and returns the derived `Formula`.

The `err` closure at the top of the function captures `rule_name` so every arm
can produce a consistent `RuleError` without repeating the rule name:

```rust
let err = |reason: &str| -> RuleError {
    RuleError { rule: rule_name.clone(), reason: reason.to_string() }
};
```

### `ModusPonens` flexibility

`ModusPonens` tries both orderings of the two premises: it first checks whether
`premises[0]` is an `Implies`, and if not, checks whether `premises[1]` is an
`Implies`.  This means `ModusPonens(imp, ha)` and `ModusPonens(ha, imp)` both
work as long as one is the implication and the other is its antecedent.

### Substitution in `ForallElim`

```rust
Rule::ForallElim => {
    match &premises[0] {
        Formula::Forall(var, body) => {
            let term = &premises[1];
            Ok(substitute(body, var, term))
        }
        _ => Err(err("first premise must be a universal quantification")),
    }
}
```

`substitute` is called directly from `apply_rule`; no environment is consulted.
The second premise is the term formula as a whole; its variable name (if it is a
`Var`) is used as-is by `substitute`.

---

## Error propagation

```
RuleError
    │
    │  .map_err(|e| make_err(e.to_string()))
    ▼
CheckError  (step_index: usize, step: Box<ProofStep>, message: String)
    │
    │  From<CheckError> for AlifError
    ▼
AlifError::Check(CheckError)
```

```
ParseError  (message: String, offset: Option<usize>)
    │
    │  From<ParseError> for AlifError
    ▼
AlifError::Parse(ParseError)
```

`RuleError` does not implement `From<…> for CheckError` directly because the
conversion requires `step_index` and `step`, which are not available inside
`apply_rule`.  Instead, the checker calls `.to_string()` on the `RuleError` and
embeds the string into the `CheckError` message.  This loses the structured
`RuleError` fields but keeps the `CheckError` type simple.

### `Box<ProofStep>` in `CheckError`

The `step` field in `CheckError` is `Box<ProofStep>` rather than `ProofStep`.
Without the box, the `CheckError` struct would be large enough to trigger the
`clippy::result_large_err` lint, because `ProofStep::Have` contains a `Formula`
(which is recursive) and a `Justification` (which contains a `Vec<String>`).
Boxing the step moves it to the heap and keeps `CheckError` to three words on
the stack.

---

## FFI — `src/lib.rs`

### Why `unsafe extern "C"`

The `extern "C"` keyword selects the C calling convention, which is the only
ABI that is stable and universally supported across languages.  The `unsafe`
modifier is required because the function accepts a raw pointer (`*const c_char`)
whose validity cannot be checked by the Rust type system.

### `CStr::from_ptr` safety contract

```rust
let c_str = if source.is_null() {
    return 2;
} else {
    unsafe { CStr::from_ptr(source) }
};
```

The null check before `CStr::from_ptr` is the only runtime guard.  The safety of
`CStr::from_ptr` then relies entirely on the caller's guarantee that `source`
points to a valid null-terminated sequence of bytes.  No bounds or alignment
checks are performed beyond what the CPU enforces during memory access.

### Coarse return codes

The three-code interface (0/1/2) is a deliberate trade-off.  A richer interface
would require either:

- A caller-supplied output buffer for error strings (complicates the API and
  requires the caller to manage buffer lifetimes), or
- A global error state (introduces global mutable state and breaks thread safety).

For callers that need detailed errors, the Rust API is the correct choice.

---

## Stdlib embedding — `src/stdlib.rs`

```rust
const STDLIB_SOURCE: &str = include_str!("../stdlib/logic.alif");
```

`include_str!` reads `stdlib/logic.alif` at **compile time** and embeds the
content as a `&'static str` constant.  There is no file I/O at runtime; the
library contains the stdlib text verbatim.

`load_stdlib()` calls `parse_source(STDLIB_SOURCE)` on every call to
`verify_source`.  This means the stdlib is parsed once per call.  For the
current size of `logic.alif` (four axioms), this cost is negligible.  A future
optimisation could cache the parsed result using `std::sync::OnceLock`, but this
is not done today.

---

## Design decisions

### Syntactic equality only — no unification or alpha-equivalence

The checker uses plain `PartialEq` to compare formulas.  Two formulas are
considered equal if and only if their AST trees are identical, including the
exact names of all variables.

This means `forall X: X` and `forall Y: Y` are **not** equal under the current
checker, even though they are logically equivalent (alpha-equivalent).  Proof
authors must ensure that all formulas in a proof use consistent variable names.

The rationale: unification requires a constraint solver; alpha-equivalence
requires a renaming pass and a notion of "variable identity" beyond string
equality.  Both would significantly increase the complexity of the checker.
Syntactic equality is sound (it never accepts an invalid proof) and is
sufficient for the pedagogical proofs Alif targets.

### Predicate application stored as flat `Var("f(x,y)")`

When the parser encounters `human(socrates)`, it constructs
`Formula::Var("human(socrates)")`.  The arguments are not stored separately and
the predicate is not tracked as a separate AST node.

This means the verifier cannot reason about the structure of predicate
applications.  For example, it cannot automatically conclude `human(socrates)`
from a schema `forall X: human(X)` by substituting `X := socrates` — the user
must write an explicit `ForallElim` step.

The rationale: a separate `Predicate(String, Vec<Formula>)` variant would
require changes to `substitute`, `formulas_match`, `Display`, and all rule
implementations.  More importantly, it would require defining what substitution
means for compound terms, which borders on first-order unification.  The flat
`Var` approach keeps the system simple and self-consistent.

### Hand-rolled parser instead of chumsky

`chumsky = "0.9"` appears in `Cargo.toml` and is a declared dependency, but the
parser in `src/parser.rs` does not use it.  The dependency was retained for
potential future use (e.g. better error recovery with labelled spans).

The hand-rolled recursive-descent parser was chosen because:

- It requires no dependencies at the call sites.
- Error messages can be crafted exactly; combinator libraries generate generic
  errors that require significant wrapper code to make user-friendly.
- The Alif grammar is simple enough that the entire parser fits in one file
  without becoming unwieldy.
- Debugging a failing parser is easier when the call stack directly mirrors the
  grammar productions.

### `Box<ProofStep>` in `CheckError`

`ProofStep::Have` contains a `Formula` and a `Justification`.  `Formula` is
recursive (it contains `Box<Formula>` children); its in-memory size is at least
three words (discriminant + pointer + optional second pointer).  `Justification`
contains a `String` and potentially a `Vec<String>`.  Together with the `name`
and `formula` fields of the `Have` variant, a `ProofStep` on the stack can be
dozens of bytes.

Rust's `clippy::result_large_err` lint fires when a `Result`'s `Err` variant
exceeds 128 bytes, because large `Err` variants are copied on every `?`
propagation.  Boxing the `ProofStep` reduces `CheckError` to three pointer-sized
words regardless of the size of the step, eliminating the lint and making error
propagation cheaper.
