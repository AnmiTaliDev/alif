# Rust API

The crate `alif` is a library with the name `alif`. Add it as a path or git dependency:

```toml
[dependencies]
alif = { path = "path/to/alif" }
```

## Entry points

### `verify_source`

```rust
pub fn verify_source(source: &str) -> Result<(), AlifError>
```

Parses `source`, loads the standard library and checks every theorem in order. Returns `Ok(())` when all items succeed, and the first error otherwise. `import` items are rejected with a load error, because a string has no directory.

```rust
use alif::verify_source;

let source = "theorem id: A |- A\nproof\n  assume h: A\n  exact h\nqed\n";
match verify_source(source) {
    Ok(()) => println!("verified"),
    Err(e) => eprintln!("{}", e),
}
```

### `verify_file`

```rust
pub fn verify_file(path: &Path) -> Result<(), AlifError>
```

Reads the file at `path` and verifies it. `import` items are resolved relative to the directory of the file that contains them. Each file is loaded once per call. An unreadable file or a failed import is a load error.

```rust
use std::path::Path;
use alif::verify_file;

verify_file(Path::new("examples/and_comm.alif")).unwrap();
```

Both functions build a fresh context for each call, so calls do not share declarations.

### `platform_warning`

```rust
pub fn platform_warning() -> Option<&'static str>
```

Returns the warning text for macOS and Windows and `None` elsewhere. The library functions do not print it. The CLI and the C functions do.

## Errors

```rust
pub enum AlifError {
    Parse(ParseError),
    Check(CheckError),
    Load(LoadError),
}
```

`AlifError` and the three inner types implement `Display` and `std::error::Error`. The `Display` output is the message format described in [errors.md](errors.md). `From` conversions exist from each inner type to `AlifError`.

| Type | Fields |
|------|--------|
| `ParseError` | `message: String`, `offset: Option<usize>`, `location: Option<Location>` |
| `CheckError` | `theorem: String`, `step_index: usize` (zero based), `offset: usize`, `message: String`, `location: Option<Location>` |
| `LoadError` | `message: String`, `location: Option<Location>` |
| `Location` | `file: Option<String>`, `line: usize`, `column: usize` (both one based) |

`offset` is a byte offset into the source text. The verification functions fill in `location`. Functions that work on a single piece of text, such as `parse_source` and `check`, leave it as `None`. `ParseError::locate`, `CheckError::locate` and `Location::new` compute a location from a source string and a file label.

```rust
use alif::{verify_source, AlifError};

let source = "theorem t: A |- B\nproof\n  assume h: A\n  exact h\nqed";
match verify_source(source) {
    Ok(()) => {}
    Err(AlifError::Parse(e)) => println!("syntax: {}", e.message),
    Err(AlifError::Check(e)) => {
        let at = e.location.as_ref().unwrap();
        println!("theorem {} step {} at {}:{}", e.theorem, e.step_index + 1, at.line, at.column);
    }
    Err(AlifError::Load(e)) => println!("load: {}", e.message),
}
```

The code above prints `theorem t step 2 at 4:3`.

## Syntax tree

```rust
pub enum Term {
    Name(String),
    App(String, Vec<Term>),
}

pub enum Formula {
    Atom(String, Vec<Term>),
    Eq(Term, Term),
    Bottom,
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Not(Box<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    Iff(Box<Formula>, Box<Formula>),
    Forall(String, Box<Formula>),
    Exists(String, Box<Formula>),
}

pub enum Justification {
    Ref(String),
    Rule(String, Vec<Term>),
}

pub enum ProofStep {
    Assume { name: String, formula: Formula, offset: usize },
    Have { name: String, formula: Formula, justification: Justification, offset: usize },
    Exact { justification: Justification, offset: usize },
}

pub struct Theorem {
    pub name: String,
    pub hypotheses: Vec<Formula>,
    pub conclusion: Formula,
    pub steps: Vec<ProofStep>,
    pub offset: usize,
}

pub enum Item {
    Axiom { name: String, formula: Formula, offset: usize },
    Theorem(Theorem),
    Import { path: String, offset: usize },
}
```

A propositional atom is `Formula::Atom(name, vec![])`. A predicate application has a non-empty argument list. `offset` fields are byte offsets of the first token of the construct. `ProofStep::offset()` returns the offset of any step.

All of these types implement `Debug`, `Clone` and `PartialEq`. `Term` and `Formula` implement `Display` with parenthesised binary connectives, for example `(A AND B)`. `Display` is meant for messages. Quantifiers are printed without surrounding parentheses, so the text is not guaranteed to parse back to the same tree.

`PartialEq` on `Formula` is structural. Use `alif::formula_ops::alpha_eq` to compare up to renaming of bound variables.

## Parsing and checking

```rust
pub fn parse_source(source: &str) -> Result<Vec<Item>, ParseError>
pub fn check(theorem: &Theorem, ctx: &Context) -> Result<(), CheckError>
```

`parse_source` parses a source string without checking it. `check` checks one theorem against a context.

```rust
pub struct Context {
    pub axioms: HashMap<String, Formula>,
    pub theorems: HashMap<String, Sequent>,
}

pub struct Sequent {
    pub hypotheses: Vec<Formula>,
    pub conclusion: Formula,
}
```

`Context` implements `Default` and has `contains(&self, name: &str) -> bool`. `check` does not modify the context. To check a file item by item, add each axiom and each checked theorem to the context yourself. `alif::stdlib::load_stdlib() -> Result<Context, AlifError>` returns a context with the standard library.

```rust
use alif::{check, parse_source, stdlib::load_stdlib, Item, Sequent};

let mut context = load_stdlib()?;
for item in parse_source(source)? {
    match item {
        Item::Axiom { name, formula, .. } => {
            context.axioms.insert(name, formula);
        }
        Item::Theorem(theorem) => {
            check(&theorem, &context)?;
            context.theorems.insert(
                theorem.name.clone(),
                Sequent {
                    hypotheses: theorem.hypotheses,
                    conclusion: theorem.conclusion,
                },
            );
        }
        Item::Import { .. } => {}
    }
}
```

This loop does not enforce the naming rules of `verify_source`: duplicate names, built-in rule names and imports. Use `verify_source` or `verify_file` when those rules matter.

## Lower-level modules

These modules are public. Their items can change more often than the entry points above.

| Module | Contents |
|--------|----------|
| `lexer` | `Token`, `Lexeme { token, text, offset }`, `lex(&str) -> Result<Vec<Lexeme>, ParseError>` |
| `rules` | `Rule` (21 variants, `from_name`, `name`, `all`), `Fact`, `Env`, `apply_rule`, `lookup_fact` |
| `formula_ops` | `free_vars`, `term_free_vars`, `substitute`, `alpha_eq`, `matches_all`, `replaces` |
| `verify` | `verify_source`, `verify_file` |
| `ffi` | the C functions, see [ffi.md](ffi.md) |

`formula_ops::substitute(formula, var, term)` is capture-avoiding. `alpha_eq(a, b)` compares up to renaming. `matches_all(&[(pattern, target)])` tests whether one replacement of propositional atoms makes every pattern equal to its target. `replaces(source, target, from, to)` tests whether `target` is `source` with some occurrences of `from` replaced by `to`.
