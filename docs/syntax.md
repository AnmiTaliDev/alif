# Syntax

This document defines the lexical rules, the grammar and the scoping rules of `.alif` files. The meaning of proof steps and rules is described in [inference-rules.md](inference-rules.md).

## Source files

A source file is UTF-8 text. The extension `.alif` is a convention and is not checked. Whitespace separates tokens. Indentation and line breaks have no meaning.

A comment starts with `--` and runs to the end of the line.

```
-- a comment
axiom a: A  -- another comment
```

Outside comments and string literals only ASCII characters are accepted. Any other character is reported as an unrecognised token.

## Tokens

| Kind | Tokens |
|------|--------|
| Keywords | `axiom` `theorem` `import` `proof` `assume` `have` `exact` `qed` `forall` `exists` |
| Connective keywords | `AND` `OR` `NOT` `FALSE` |
| Operators | `\|-` `=>` `<=>` `:=` `=` |
| Punctuation | `:` `,` `(` `)` |
| Identifier | an ASCII letter or `_`, followed by ASCII letters, digits or `_` |
| String | `"` followed by any characters except `"` and a line break, followed by `"` |

Keywords are case sensitive and cannot be used as identifiers. A word that only begins with a keyword, such as `ANDx` or `proofs`, is an identifier.

## Grammar

```
file          = { item } ;
item          = axiom | theorem | import ;

axiom         = "axiom" ident ":" formula ;
import        = "import" string ;
theorem       = "theorem" ident ":" sequent "proof" { step } "qed" ;

sequent       = [ formula { "," formula } ] "|-" formula
              | formula ;

step          = assume | have | exact ;
assume        = "assume" ident ":" formula ;
have          = "have" ident ":" formula ":=" justification ;
exact         = "exact" justification ;
justification = ident [ "(" [ term { "," term } ] ")" ] ;

formula       = iff ;
iff           = implies { "<=>" implies } ;
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

A sequent without `|-` consists of a single formula and has no hypotheses. A sequent with `|-` may have an empty hypothesis list.

The parser reads steps until it meets a token that does not start a step, and then requires `qed`.

## Precedence

From the tightest to the loosest binding:

| Level | Construct | Associativity |
|-------|-----------|---------------|
| 1 | `=` between two terms | not associative |
| 2 | `NOT` | prefix |
| 3 | `AND` | left |
| 4 | `OR` | left |
| 5 | `=>` | right |
| 6 | `<=>` | left |

`A => B => C` is `A => (B => C)`. `A <=> B <=> C` is `(A <=> B) <=> C`.

The body of `forall` and `exists` extends as far to the right as possible. `forall X: P(X) AND Q(X)` quantifies over the whole conjunction. Use parentheses to limit the body: `(forall X: P(X)) AND Q`.

## Formulas

| Form | Meaning |
|------|---------|
| `A` | propositional atom: an identifier without arguments |
| `P(t1, ..., tn)` | predicate atom with one or more terms |
| `s = t` | equality between two terms |
| `FALSE` | the false formula |
| `NOT F` | negation |
| `F AND G` | conjunction |
| `F OR G` | disjunction |
| `F => G` | implication |
| `F <=> G` | equivalence |
| `forall X: F` | universal quantification |
| `exists X: F` | existential quantification |

A term is an identifier or a function application `f(t1, ..., tn)` with one or more arguments. Empty argument lists are not allowed in formulas and terms.

Variables and constants are not declared. A name in term position is a variable when an enclosing quantifier binds it. Otherwise it is a free name, which the checker treats like a constant. A propositional atom and a term name are unrelated even when they share an identifier: in `forall X: X` the atom `X` is not affected by the quantifier.

Two formulas are equal when they are equal up to renaming of bound variables. `forall X: P(X)` and `forall Y: P(Y)` are equal. Free names must be identical. Every comparison in the checker uses this equality.

## Items

### axiom

```
axiom human_socrates: human(socrates)
```

Declares a formula that holds without proof. Later proofs refer to it by name. Free names that occur in an axiom are fixed and cannot be generalised with `ForallIntro` or used as the witness of `ExistsElim`.

### theorem

```
theorem and_comm: A AND B |- B AND A
proof
  assume h: A AND B
  have a: A := AndElimLeft(h)
  have b: B := AndElimRight(h)
  have r: B AND A := AndIntro(b, a)
  exact r
qed
```

Declares a sequent and proves it. After the proof is checked, the theorem can be applied in later items. Applying a theorem is described in [inference-rules.md](inference-rules.md#applying-theorems).

### import

```
import "lib/common.alif"
```

Loads another file and checks it in place. The path is relative to the directory of the importing file. Rules:

- Each file is loaded once. A second import of the same file has no effect.
- An import cycle is an error.
- The items of the imported file become available to the items that follow the import.
- `import` is not available when the source is given as a string or read from standard input, because there is no directory to resolve the path against.

## Proof steps

| Step | Effect |
|------|--------|
| `assume h: F` | adds `F` under the name `h`, as an assumption |
| `have h: F := J` | derives `F` by justification `J` and adds it under the name `h` |
| `exact J` | derives the conclusion by `J` and ends the proof |

A proof must contain at least one step, must end with `exact`, and must not contain a step after `exact`. A name can be defined only once in a proof.

## Justifications

A justification is either a bare name or a name with an argument list.

| Form | Resolution, in this order |
|------|---------------------------|
| `name` | a fact of the proof, an axiom, a rule applied to no arguments, a theorem without hypotheses |
| `name(args)` | a built-in rule, then a theorem |

Arguments are separated by commas. Depending on the rule, an argument is the name of a fact or a term. Facts are looked up among the names of the proof first and among the axioms second.

## Names

Axioms and theorems share one namespace that spans the standard library, imported files and the current file.

- A name can be declared once. A second declaration is a load error.
- Names from the standard library can be redeclared once. The new declaration replaces the library entry for the rest of the run. See [stdlib.md](stdlib.md).
- The 21 rule names cannot be used for axioms or theorems.
- An item can use only items declared before it.
- A name introduced by `assume` or `have` is visible from the step after its definition to the end of the proof. It takes precedence over an axiom with the same name.
