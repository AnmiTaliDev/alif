# Rust interface

The library crate is named `alif`.

```toml
[dependencies]
alif = { path = "path/to/alif" }
```

## Verifying

```rust
pub fn verify_source(source: &str) -> Result<(), AlifError>
pub fn verify_file(path: &Path) -> Result<(), AlifError>
```

Both load the standard library, process the declarations in order, and return the first error. Each call starts from a fresh context.

- `verify_source` works on a string. An `import` in it is a load error.
- `verify_file` reads the file and resolves `import` relative to the directory of the file that contains it. Each file is loaded once per call.

```rust
use std::path::Path;

let source = "theorem id: A |- A\nproof\n  assume h: A\n  exact h\nqed\n";
alif::verify_source(source).unwrap();
alif::verify_file(Path::new("examples/and_comm.alif")).unwrap();
```

```rust
pub fn platform_warning() -> Option<&'static str>
```

Returns the warning text on macOS and Windows and `None` elsewhere. The library functions above do not print it. The command line tool and the C functions do.

## Errors

```rust
pub enum AlifError {
    Parse(ParseError),
    Check(CheckError),
    Load(LoadError),
}
```

All four types implement `Debug`, `Display` and `std::error::Error`, and the inner types convert into `AlifError` with `From`. `Display` produces the diagnostic text from [errors.md](errors.md).

| Type | Fields |
|------|--------|
| `ParseError` | `message: String`, `offset: Option<usize>`, `location: Option<Location>` |
| `CheckError` | `theorem: String`, `step_index: usize`, `offset: usize`, `message: String`, `location: Option<Location>` |
| `LoadError` | `message: String`, `location: Option<Location>` |
| `Location` | `file: Option<String>`, `line: usize`, `column: usize` |

`step_index` counts from 0. `line` and `column` count from 1. `offset` is a byte offset into the source text.

`verify_source` and `verify_file` fill in `location`. Lower-level functions leave it `None`. `Location::new(file, source, offset)`, `ParseError::locate(file, source)` and `CheckError::locate(file, source)` compute it from a source text.

```rust
use alif::AlifError;

let source = "theorem t: A |- B\nproof\n  assume h: A\n  exact h\nqed";
match alif::verify_source(source) {
    Ok(()) => {}
    Err(AlifError::Parse(e)) => println!("syntax: {}", e.message),
    Err(AlifError::Check(e)) => {
        let at = e.location.as_ref().unwrap();
        println!("{} step {} at {}:{}", e.theorem, e.step_index + 1, at.line, at.column);
    }
    Err(AlifError::Load(e)) => println!("load: {}", e.message),
}
```

This prints `t step 2 at 4:3`.

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

A propositional letter is `Atom(name, vec![])`. `offset` is the byte offset of the first token of the construct, and `ProofStep::offset()` returns it for any step.

The types derive `Debug`, `Clone` and `PartialEq`. `Term` and `Formula` also implement `Display`, which is meant for messages: binary connectives are parenthesised, quantifiers are not, so the text may not parse back to the same tree. `==` on formulas is structural. Use `alif::formula_ops::alpha_eq` to compare up to renaming of bound variables.

## Parsing and checking separately

```rust
pub fn parse_source(source: &str) -> Result<Vec<Item>, ParseError>
pub fn check(theorem: &Theorem, ctx: &Context) -> Result<(), CheckError>
```

`parse_source` only parses. `check` verifies one theorem against a context and does not change it.

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

`Context` implements `Default` and has `contains(&self, name: &str) -> bool`. `alif::stdlib::load_stdlib()` returns a context that holds the standard library.

A caller that checks declarations one by one has to add each axiom and each accepted theorem to the context:

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

This loop does not apply the naming rules (no duplicates, no rule names) and does not resolve imports. `verify_source` and `verify_file` do.

## Other public modules

These can change more often than the functions above.

| Module | Items |
|--------|-------|
| `lexer` | `Token`, `Lexeme { token, text, offset }`, `lex` |
| `rules` | `Rule` with `from_name`, `name` and `all`; `Fact`; `Env`; `apply_rule`; `lookup_fact` |
| `formula_ops` | `free_vars`, `term_free_vars`, `substitute`, `alpha_eq`, `matches_all`, `replaces` |
| `verify` | `verify_source`, `verify_file` |
| `ffi` | the C functions, see [ffi.md](ffi.md) |

`substitute(formula, var, term)` avoids capture. `matches_all(&[(pattern, target)])` tests whether one assignment to the propositional letters makes each pattern equal to its target. `replaces(source, target, from, to)` tests whether `target` is `source` with some occurrences of `from` replaced by `to`.
