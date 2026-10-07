# Architecture

Alif is a library with a thin command line front end. The library parses source text, builds a context of axioms and theorems, and checks each theorem proof step by step.

## Pipeline

```
source text
    |  lexer::lex
    v
lexemes (token, text, byte offset)
    |  parser::parse_source
    v
items: axiom | theorem | import
    |  verify::Verifier::process
    v
for each item, in order:
    axiom    -> declare name, store formula in the context
    import   -> load another file through the same Verifier
    theorem  -> declare name, checker::check, store sequent in the context
```

The `Verifier` holds the context, the set of files already loaded and the stack of files being loaded. It starts with the standard library, which it processes with the same code as user files.

## Modules

| Module | Responsibility |
|--------|----------------|
| `lexer` | Splits text into tokens with the `logos` crate. Records the byte offset of each token. |
| `parser` | Recursive descent parser. Produces `Item` values with offsets. |
| `term` | Syntax tree types and their `Display` implementations. |
| `context` | `Context` (axioms and theorems by name) and `Sequent`. |
| `formula_ops` | Free variables, capture-avoiding substitution, comparison up to renaming, pattern matching for theorem application, replacement check for equality. |
| `rules` | The 21 built-in rules and the `Fact` type. |
| `checker` | Walks the steps of one proof, resolves justifications, applies theorems. |
| `verify` | Processes the items of a file, resolves imports, enforces naming rules. |
| `stdlib` | Loads `stdlib/logic.alif` into a context. |
| `error` | Error types and source locations. |
| `ffi` | C functions and the per-thread error message. |
| `main.rs` | Command line interface. |

`lib.rs` declares the modules and re-exports the entry points.

## Data model

`Formula` and `Term` are plain recursive enums. A propositional letter is an atom with no arguments. Names in term position are not classified as variables or constants. A name is bound when a quantifier binds it and free otherwise.

All formula comparisons in the checker go through `formula_ops::alpha_eq`, so bound variable names never matter. Pattern matching and alpha equivalence share one matcher. The matcher keeps a stack that pairs each bound variable on the left with the bound variable on the right, and it uses the stack to decide whether two names correspond.

## Checking a proof

`checker::check` keeps an environment that maps names to facts. A fact has a formula, a set of dependencies and an assumption flag. The set of dependencies is a `BTreeSet<String>` of assumption names.

For every step:

- `assume` adds a fact with itself as the only dependency.
- `have` derives a fact through `derive`, compares its formula with the declared one, and stores the fact with the declared formula and the derived dependencies.
- `exact` derives a fact, compares it with the conclusion, and requires that every dependency is a hypothesis of the theorem. It has to be the last step.

`derive` resolves a justification. A bare name is looked up as a proof fact, then as an axiom, then as a rule or theorem without arguments. A name with arguments is looked up as a rule, then as a theorem.

## Soundness

The checker is meant to accept a proof only when the conclusion follows from the hypotheses and axioms by the rules of natural deduction. These design points carry that property:

- **Dependencies.** A fact derived from an assumption records that assumption. Only discharging rules remove an entry, and each of them requires an assumption as its argument. `exact` accepts only dependencies that are hypotheses.
- **Fixed names.** `ForallIntro` and `ExistsElim` reject variables that are free in an assumption the fact depends on or in any axiom. This stops generalisation over a name that a hypothesis or an axiom talks about.
- **Capture avoidance.** Substitution renames a bound variable when the inserted term would be captured.
- **Theorem application.** A theorem is applied only through one-way matching. Placeholders can be replaced only by formulas that contain no name captured by a quantifier at the placeholder position.
- **Proof shape.** A proof has to end with `exact`, names are not redefined, and nothing follows `exact`.

The checker has no proof search, no unification and no type inference. A rule either accepts its arguments or fails with a reason.

The test suite contains proofs that earlier designs accepted and the current checker rejects. They are in `tests/fixtures/invalid_proof`.

## Imports and names

`Verifier::load` canonicalises a path to detect cycles and repeated loads. The directory used to resolve nested imports is the directory of the path as it was written, so that error messages keep the user's relative paths.

Axioms and theorems share one name table. The names of the standard library are recorded as replaceable. The first user declaration of such a name removes the library entries with that name, and the name is no longer replaceable afterwards. The names of the 21 rules are reserved.

## Errors

`ParseError`, `CheckError` and `LoadError` carry a byte offset or a location. The functions that know the source text and the file label, namely `Verifier::process` and `Verifier::load`, convert offsets into `Location` values. Lower-level functions leave `location` empty.

## C interface

`ffi` wraps `verify_source` and `verify_file`. The message of the last failure is stored in a thread-local `Option<CString>` that every call replaces. The return value is derived from the error variant: proof errors give 1, other errors give 2.

## Tests

| Location | Content |
|----------|---------|
| unit tests in `src/` | Lexer, parser, formula operations, every rule, the checker and the verifier |
| `tests/fixtures.rs` | Runs every file in `examples/` and in `tests/fixtures/` and checks the error kind |
| `tests/cli.rs` | Runs the executable and checks output and exit status |

The fixture directories are `valid`, `invalid_proof`, `invalid_syntax` and `invalid_load`. A new file placed in one of them is picked up without changes to the test code.
