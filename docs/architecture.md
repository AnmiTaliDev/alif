# Architecture

Alif is a Rust library with a small command line front end. The library parses text into declarations and checks each theorem against a context of axioms and earlier theorems.

## Flow

```
text
  -> lexer::lex            tokens with byte offsets
  -> parser::parse_source  declarations
  -> verify::Verifier      one declaration at a time:
        axiom    store the formula
        import   load the file with the same Verifier
        theorem  checker::check, then store the sequent
```

`Verifier` owns the context, the set of files already loaded and the stack of files being loaded. It begins with the standard library, which goes through the same code path as a user file.

## Modules

| Module | Role |
|--------|------|
| `lexer` | tokenizer built on `logos`; records the byte offset of every token |
| `parser` | recursive descent parser producing `Item` values |
| `term` | syntax tree types and `Display` |
| `context` | `Context` (axioms and theorems by name) and `Sequent` |
| `formula_ops` | free variables, substitution, equality up to renaming, pattern matching, replacement check |
| `rules` | the 21 rules and the `Fact` type |
| `checker` | checks the steps of one proof, resolves justifications, applies theorems |
| `verify` | processes declarations, resolves imports, enforces naming rules |
| `stdlib` | loads `stdlib/logic.alif` |
| `error` | error types and locations |
| `ffi` | C functions |
| `main.rs` | command line tool |

## Data

`Formula` and `Term` are recursive enums. A propositional letter is an atom without arguments. A name in a term is not classified in the tree; whether it is bound follows from the quantifiers around it.

Formulas are compared with `formula_ops::alpha_eq`. It and the pattern matcher for theorems are one implementation. The matcher keeps a stack of pairs, each pairing a variable bound on the left with the variable bound at the same place on the right, and uses it to decide whether two names correspond. In pattern mode a propositional letter on the left matches any formula on the right, and the match is recorded.

## Checking a proof

`checker::check` keeps an environment from names to facts. A fact is a formula, a `BTreeSet<String>` of assumption names and an assumption flag.

- `assume` stores a fact whose dependency set is its own name.
- `have` derives a fact, compares its formula with the declared one, and stores the declared formula with the derived dependencies.
- `exact` derives a fact, compares it with the conclusion, and requires every dependency to be a hypothesis. It also has to be the last step.

`derive` resolves a justification. A bare name is tried as a fact of the proof, an axiom, and then a rule or theorem with no arguments. A name with arguments is tried as a rule, then as a theorem. Arguments that are facts go through `rules::lookup_fact`, which gives proof names precedence over axioms.

## Why the checker is sound

The checker accepts a proof only when the conclusion follows from the hypotheses and axioms by natural deduction. The following properties carry this.

- Dependencies. A derived fact records the assumptions it rests on. Only discharging rules remove an entry, and they require an assumption as the argument. `exact` accepts only hypotheses.
- Fixed names. `ForallIntro` and `ExistsElim` reject a variable that is free in an assumption the fact depends on, or in any axiom.
- Capture avoidance. Substitution renames a bound variable when the inserted term would be captured.
- Theorem application. A theorem is applied by one-way matching, and a placeholder never receives a formula with a free variable that a quantifier in the pattern binds.
- Proof shape. A proof has to end with `exact`, names are defined once, and nothing follows `exact`.

There is no search, unification or type inference. A rule accepts its arguments or fails with a reason.

Inputs that earlier designs accepted and the current checker rejects are in `tests/fixtures/invalid_proof`.

## Imports and names

`Verifier::load` canonicalises a path to detect cycles and repeated loads. The directory used for nested imports is taken from the path as written, so that diagnostics keep the paths the user wrote.

Axioms and theorems share one name table. At start, names that came from the standard library are marked replaceable. The first user declaration of such a name removes the library entries with that name and clears the mark. The rule names are reserved.

## Errors

`ParseError`, `CheckError` and `LoadError` carry byte offsets. `Verifier::process` and `Verifier::load` know the text and the file label, and convert offsets to `Location` values. Lower-level functions leave `location` empty.

## C interface

`ffi` wraps `verify_source` and `verify_file`. The message of the last failure lives in a thread-local `Option<CString>` that each call overwrites. A proof error maps to 1 and every other error to 2.

## Tests

| Location | Content |
|----------|---------|
| unit tests in `src/` | lexer, parser, formula operations, every rule, checker, verifier, FFI |
| `tests/fixtures.rs` | runs every file under `examples/` and `tests/fixtures/` and checks the kind of error |
| `tests/cli.rs` | runs the executable and checks output and exit status |

The fixture directories are `valid`, `invalid_proof`, `invalid_syntax` and `invalid_load`. A new file in one of them is picked up with no change to the test code.
