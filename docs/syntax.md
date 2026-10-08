# Syntax

This document defines what a valid `.alif` file looks like. What a proof step means is defined in [inference-rules.md](inference-rules.md).

## Lexical structure

A file is UTF-8 text. Whitespace separates tokens, and line breaks and indentation carry no meaning. A comment starts with `--` and ends at the end of the line.

```
axiom a: A   -- comment
```

Outside comments and string literals, only ASCII characters are allowed.

| Token class | Spelling |
|-------------|----------|
| Keyword | `axiom` `theorem` `import` `proof` `assume` `have` `exact` `qed` `forall` `exists` |
| Connective | `AND` `OR` `NOT` `FALSE` |
| Operator | `\|-` `=>` `<=>` `:=` `=` |
| Punctuation | `:` `,` `(` `)` |
| Identifier | ASCII letter or `_`, then ASCII letters, digits or `_` |
| String | `"`, any characters except `"` and line breaks, `"` |

The lexer takes the longest match. `proofs` and `ANDx` are identifiers, not keywords followed by a letter. Keywords are case sensitive and cannot be used as names.

## Terms and formulas

A term is an identifier, or a function symbol applied to one or more terms:

```
socrates
f(a, g(b))
```

A formula is one of:

| Formula | Reading |
|---------|---------|
| `A` | propositional letter, an identifier without arguments |
| `P(t1, ..., tn)` | predicate applied to one or more terms |
| `s = t` | equality of two terms |
| `FALSE` | falsity |
| `NOT F` | negation |
| `F AND G` | conjunction |
| `F OR G` | disjunction |
| `F => G` | implication |
| `F <=> G` | equivalence |
| `forall X: F` | universal quantification |
| `exists X: F` | existential quantification |
| `(F)` | grouping |

Argument lists cannot be empty in formulas and terms.

### Precedence

From tightest to loosest:

| Level | Operator | Grouping |
|-------|----------|----------|
| 1 | `=` | none |
| 2 | `NOT` | prefix |
| 3 | `AND` | left |
| 4 | `OR` | left |
| 5 | `=>` | right |
| 6 | `<=>` | left |

So `A => B => C` means `A => (B => C)`, and `A OR B AND C` means `A OR (B AND C)`.

A quantifier takes everything to its right as its body: `forall X: P(X) AND Q(X)` is `forall X: (P(X) AND Q(X))`. Use parentheses to stop the body early: `(forall X: P(X)) AND Q`.

### Variables and constants

Names are not declared. In a term, a name is a variable when an enclosing quantifier binds it, and otherwise it is free. The checker treats free names like constants. In `human(socrates)` the name `socrates` is free. In `forall X: human(X)` the name `X` is bound.

A propositional letter is not a term. In `forall X: X` the letter `X` is unrelated to the bound variable.

### Equality of formulas

Formulas are compared up to renaming of bound variables. `forall X: P(X)` equals `forall Y: P(Y)`. Free names have to be identical. All comparisons in the checker use this notion.

## Declarations

A file is a sequence of declarations.

### `axiom`

```
axiom all_mortal: forall X: human(X) => mortal(X)
```

Adds a formula that needs no proof. A proof refers to it by name. Free names inside an axiom count as fixed, see [inference-rules.md](inference-rules.md#fixed-names).

### `theorem`

```
theorem and_comm:
  A AND B |- B AND A
proof
  assume h: A AND B
  have a: A := AndElimLeft(h)
  have b: B := AndElimRight(h)
  have r: B AND A := AndIntro(b, a)
  exact r
qed
```

A theorem has a name, a sequent and a proof. The sequent is a list of hypotheses separated by commas, then `|-`, then the conclusion. The hypothesis list may be empty, and `|-` may be omitted when it is:

```
theorem t1: |- A => A
proof
  exact identity
qed

theorem t2: A => A
proof
  exact identity
qed
```

Once its proof is accepted, the theorem can be applied in later proofs, see [inference-rules.md](inference-rules.md#applying-theorems).

### `import`

```
import "lib/common.alif"
```

Loads another file in place. The path is taken relative to the directory of the file that contains the `import`.

- A file that was already loaded is not loaded again.
- A cycle of imports is an error.
- Declarations of the imported file are visible below the `import`.
- `import` is an error when the text has no directory, which is the case for source passed as a string and for standard input.

## Proofs

A proof lies between `proof` and `qed` and consists of steps:

```
assume name: formula
have name: formula := justification
exact justification
```

Conditions on the list of steps:

- there is at least one step,
- the last step is `exact`,
- no other step is `exact`,
- no name is defined twice.

A justification has two forms:

| Form | Meaning |
|------|---------|
| `name` | a step of the proof, an axiom, a rule that takes no arguments, or a theorem without hypotheses |
| `name(arg, ...)` | a rule or a theorem applied to arguments. The list may be empty. |

For a bare `name`, a step of the proof is tried first, then an axiom, then a rule or theorem. For `name(...)`, rules are tried before theorems. An argument is either a name of a step or axiom or a term, depending on the rule. A step name shadows an axiom of the same name.

## Names and scope

- Axioms and theorems share a single namespace across the standard library, imported files and the current file.
- A name can be declared once. A second declaration is an error.
- Names defined by the standard library may be declared once by the user, which replaces the library entry. See [stdlib.md](stdlib.md#redeclaring-library-names).
- The 21 rule names are reserved.
- A declaration can use only declarations above it.
- A step name is visible from the next step to the end of its proof.

## Grammar

```
file          = { declaration } ;
declaration   = axiom | theorem | import ;
axiom         = "axiom" ident ":" formula ;
import        = "import" string ;
theorem       = "theorem" ident ":" sequent "proof" { step } "qed" ;
sequent       = [ formula { "," formula } ] "|-" formula | formula ;
step          = "assume" ident ":" formula
              | "have" ident ":" formula ":=" justification
              | "exact" justification ;
justification = ident [ "(" [ term { "," term } ] ")" ] ;
formula       = implies { "<=>" implies } ;
implies       = or [ "=>" implies ] ;
or            = and { "OR" and } ;
and           = not { "AND" not } ;
not           = "NOT" not | atom ;
atom          = "forall" ident ":" formula
              | "exists" ident ":" formula
              | "(" formula ")"
              | "FALSE"
              | term [ "=" term ] ;
term          = ident [ "(" term { "," term } ")" ] ;
```

The parser reads steps until it meets a token that cannot start a step, and then requires `qed`. The conditions on the list of steps are checked later, by the proof checker.
