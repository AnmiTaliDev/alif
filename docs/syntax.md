# Alif Language Syntax Reference

This document is the authoritative reference for the Alif proof language. It
covers every lexical element, every grammatical construct, operator precedence,
scoping rules, and the complete set of built-in inference rules. Readers who want
to understand what a `.alif` file means, or who are implementing tooling for the
language, should read this document in full.

---

## Table of Contents

1. [File Structure](#file-structure)
2. [Lexical Elements](#lexical-elements)
   - [Comments](#comments)
   - [Whitespace](#whitespace)
   - [Keywords](#keywords)
   - [Logical Connective Keywords](#logical-connective-keywords)
   - [Operators and Punctuation](#operators-and-punctuation)
   - [Identifiers](#identifiers)
3. [Formula Grammar](#formula-grammar)
   - [EBNF](#ebnf)
   - [Operator Precedence](#operator-precedence)
   - [Precedence Examples](#precedence-examples)
   - [Predicate Application Syntax](#predicate-application-syntax)
4. [Top-Level Items](#top-level-items)
   - [Axiom Declarations](#axiom-declarations)
   - [Theorem Declarations](#theorem-declarations)
5. [Sequents](#sequents)
6. [Proof Blocks](#proof-blocks)
   - [`assume`](#assume)
   - [`have`](#have)
   - [`exact`](#exact)
7. [Justifications](#justifications)
   - [Bare Name (Environment and Axiom Lookup)](#bare-name-environment-and-axiom-lookup)
   - [Rule Application](#rule-application)
8. [Scoping Rules](#scoping-rules)
9. [Built-in Inference Rules](#built-in-inference-rules)
10. [Complete Annotated Example](#complete-annotated-example)

---

## File Structure

An Alif source file (conventionally named with the `.alif` extension) is a
sequence of zero or more *top-level items*. There are exactly two kinds of
top-level item:

- An **axiom declaration** introduces a named formula that is accepted without
  proof. Axioms declared in a file are available to all subsequent theorems in
  that same file.
- A **theorem declaration** states a sequent (hypotheses and a conclusion) and
  supplies a proof block that justifies the conclusion from the hypotheses.

Items are processed in the order they appear. An axiom declared on line 10 cannot
be referenced by a theorem declared on line 5. Apart from this ordering
constraint, there is no notion of modules, imports, or namespaces.

The standard library (`stdlib/logic.alif`) is loaded automatically before any
item in the user file is processed; its axioms are available everywhere.

```
-- File: example.alif
--
-- Any number of axiom and theorem declarations, in order.

axiom my_axiom: A => A

theorem my_theorem: A |- A
proof
  assume h: A
  exact h
qed
```

---

## Lexical Elements

The lexer processes the source text left-to-right, producing a flat token stream.
Whitespace and comments are discarded before the token stream reaches the parser.

### Comments

Alif supports **line comments only**. A line comment begins with the two-character
sequence `--` and extends to the end of the current line (i.e., up to but not
including the newline character). There are no block comments.

```
-- This is a line comment.
axiom id: A   -- This comment follows a token on the same line.
```

Attempting to nest or close comments with any other delimiter is not special;
the characters are simply part of the comment text.

### Whitespace

Whitespace characters — space (U+0020), horizontal tab (U+0009), carriage return
(U+000D), and newline (U+000A) — are insignificant and are discarded by the
lexer. Whitespace may appear freely between any two tokens. There are no
indentation requirements; proof steps may be indented by any amount (or not at
all) purely for readability.

### Keywords

The following nine identifiers are reserved as keywords. They cannot be used as
user-defined names in any context.

| Keyword   | Role                                                     |
|-----------|----------------------------------------------------------|
| `axiom`   | Begins an axiom declaration.                             |
| `theorem` | Begins a theorem declaration.                            |
| `proof`   | Begins the proof block of a theorem.                     |
| `qed`     | Ends the proof block of a theorem.                       |
| `assume`  | Introduces a hypothesis into the current proof scope.    |
| `have`    | Derives a new named formula from existing scope entries. |
| `exact`   | Closes the proof by citing the final derived formula.    |
| `forall`  | Universal quantifier in a formula.                       |
| `exists`  | Existential quantifier in a formula.                     |

All keywords are lowercase and are case-sensitive. The string `Theorem` is a
valid identifier, not the keyword `theorem`.

### Logical Connective Keywords

Three additional uppercase tokens function as infix and prefix operators within
formulas. They are recognised before identifiers by the lexer and therefore
cannot be used as user-defined names.

| Token | Arity  | Meaning                     |
|-------|--------|-----------------------------|
| `AND` | Binary | Logical conjunction (∧).    |
| `OR`  | Binary | Logical disjunction (∨).    |
| `NOT` | Unary  | Logical negation (¬), prefix.|

The implication operator `=>` is a punctuation token rather than a keyword; see
the [Operators and Punctuation](#operators-and-punctuation) table.

### Operators and Punctuation

| Token | Name              | Role                                                       |
|-------|-------------------|------------------------------------------------------------|
| `\|-`  | Turnstile         | Separates the hypothesis list from the conclusion in a sequent. |
| `=>`  | Implication arrow | Infix binary implication operator in formulas.             |
| `:=`  | Justification     | Separates the declared formula from its justification in `have`. |
| `:`   | Colon             | Separates a name from its associated formula.              |
| `,`   | Comma             | Separates hypotheses in a sequent; separates rule arguments. |
| `(`   | Left parenthesis  | Groups a sub-formula; begins a predicate argument list.    |
| `)`   | Right parenthesis | Closes a grouped sub-formula or argument list.             |

The two-character tokens `|-`, `=>`, and `:=` are matched as single units before
their individual characters are considered. For example, writing `:-` produces a
colon token followed by a minus sign, which would be a lex error (minus is not a
recognised character).

### Identifiers

An identifier is any sequence of characters matching the regular expression:

```
[A-Za-z_][A-Za-z0-9_]*
```

That is, an identifier begins with an ASCII letter (upper or lowercase) or an
underscore, followed by zero or more ASCII letters, decimal digits, or
underscores. Identifiers are case-sensitive: `Foo`, `foo`, and `FOO` are three
distinct identifiers.

All nine keywords and the three connective keywords (`AND`, `OR`, `NOT`) are
reserved and will never be produced as an `Ident` token by the lexer, regardless
of case matching. This means that while `Not` and `not` are valid identifiers,
`NOT` is always a connective keyword token.

Examples of valid identifiers: `A`, `B`, `x`, `socrates`, `human_1`, `AndIntro`,
`_tmp`, `mortal`.

Examples of invalid identifiers: `1foo` (starts with digit), `foo-bar` (hyphen
not allowed), `@name` (at-sign not allowed).

---

## Formula Grammar

### EBNF

The complete grammar for formulas, in Extended Backus-Naur Form, is:

```ebnf
formula      ::= implies-expr

implies-expr ::= or-expr ( '=>' or-expr )*

or-expr      ::= and-expr ( 'OR' and-expr )*

and-expr     ::= not-expr ( 'AND' not-expr )*

not-expr     ::= 'NOT' not-expr
             |   atom

atom         ::= 'forall' ident ':' formula
             |   'exists' ident ':' formula
             |   '(' formula ')'
             |   ident [ '(' ident { ',' ident } ')' ]
```

The grammar is unambiguous. Each level of the hierarchy corresponds directly to a
precedence class, with tighter-binding operators parsed deeper in the recursion.

### Operator Precedence

The table below lists all formula operators from **lowest precedence** (loosest
binding, outermost in the parse tree) to **highest precedence** (tightest
binding, innermost in the parse tree).

| Precedence | Operator | Associativity | Description          |
|------------|----------|---------------|----------------------|
| 1 (lowest) | `=>`     | Left          | Implication          |
| 2          | `OR`     | Left          | Disjunction          |
| 3          | `AND`    | Left          | Conjunction          |
| 4          | `NOT`    | Right (prefix)| Negation             |
| 5 (highest)| atoms    | —             | Variables, quantifiers, parenthesised groups |

Quantifiers (`forall`, `exists`) are parsed at the atom level but consume the
*entire remaining formula* as their body (by calling `parse_formula` recursively),
which means their scope extends as far right as possible. A quantifier body is
effectively the lowest-precedence sub-expression that starts after the colon.

### Precedence Examples

```
-- AND binds tighter than OR:
A AND B OR C          -- parsed as (A AND B) OR C

-- NOT binds tighter than AND:
NOT A AND B           -- parsed as (NOT A) AND B

-- NOT is right-associative:
NOT NOT A             -- parsed as NOT (NOT A)

-- => is left-associative:
A => B => C           -- parsed as (A => B) => C

-- Parentheses override precedence:
A AND (B OR C)        -- parsed as A AND (B OR C)

-- Quantifier body extends to end of formula:
forall X: X AND Y     -- body is (X AND Y), not just X
forall X: X => Y      -- body is (X => Y)

-- Multiple quantifiers:
forall X: exists Y: X AND Y
-- outer body: exists Y: X AND Y
-- inner body: X AND Y
```

### Predicate Application Syntax

An atom identifier may be followed by a parenthesised, comma-separated list of
identifier arguments to represent a predicate or function application:

```
f(x)            -- unary predicate f applied to x
rel(x, y)       -- binary predicate rel applied to x and y
mortal(socrates) -- the atom "mortal applied to socrates"
```

This syntax is **purely notational**. Internally the verifier stores the entire
application as a single flat `Var` string. For example, `mortal(socrates)` is
stored as the atom named `mortal(socrates)`, and `rel(x,y)` is stored as the atom
named `rel(x,y)` (arguments joined by commas, no spaces). There is no
type-checking, arity-checking, or signature mechanism — it is syntactic sugar over
flat propositional variables.

Consequence: the atoms `mortal(socrates)` and `mortal( socrates )` parse
identically (whitespace inside the argument list is insignificant); both produce
the Var `mortal(socrates)`.

An empty argument list — `f()` — is treated as a bare atom `f` with no
arguments: the parentheses are consumed but produce no change to the name.

---

## Top-Level Items

### Axiom Declarations

**Grammar:**

```ebnf
axiom-decl ::= 'axiom' ident ':' formula
```

**Syntax:**

```
axiom <name>: <formula>
```

An axiom declaration introduces a named formula into the global axiom environment.
The formula is accepted as true without any proof obligation. Axioms declared in a
`.alif` file are added to the same axiom map as the standard library axioms and
are available to all subsequent items in the file.

**Examples:**

```
axiom reflexivity: A => A
axiom explosion:   NOT A => A => B
axiom conjunction: A AND B
axiom all_human:   forall X: human(X) => mortal(X)
```

Axiom names follow the same identifier rules as all other names. An axiom with
the same name as a standard library axiom will shadow the standard library entry
for the rest of the file.

### Theorem Declarations

**Grammar:**

```ebnf
theorem-decl ::= 'theorem' ident ':' sequent
                 'proof'
                 proof-step*
                 'qed'
```

**Syntax:**

```
theorem <name>: <sequent>
proof
  <steps>
qed
```

A theorem declaration states a logical claim (the sequent) and supplies a proof
(the sequence of steps between `proof` and `qed`). The verifier checks that the
proof is valid. If any step fails, the entire theorem fails and verification stops
with an error message identifying the step index and the reason for failure.

---

## Sequents

A sequent expresses that a conclusion follows from a set of hypotheses:

```
hyp1, hyp2, ..., hypN |- conclusion
```

**Syntax:**

```ebnf
sequent ::= formula-list '|-' formula
          | formula
```

where:

```ebnf
formula-list ::= formula { ',' formula }
```

**Parts:**

- The formulas to the left of `|-` are the **hypotheses** (antecedents). They
  represent assumptions that are given to the theorem as premises. The verifier
  does not require that hypotheses be proved before the theorem; they are
  additional axioms scoped to that theorem's proof.
- The formula to the right of `|-` is the **conclusion** (consequent). This is
  the formula that the proof must ultimately establish.

**No-hypothesis shorthand:**

When a theorem has no hypotheses, the `|-` and the left-hand formula list may be
omitted. The single formula becomes the conclusion and the hypothesis list is
empty:

```
theorem tautology: A => A
-- is equivalent to:
-- theorem tautology: |- A => A
```

This shorthand is only valid when there is exactly one formula before `proof`. If
there are two or more formulas before `proof` and no `|-`, the parser reports an
error.

**Examples:**

```
-- No hypotheses:
theorem taut: A => A

-- One hypothesis:
theorem id: A |- A

-- Two hypotheses:
theorem mp: A => B, A |- B

-- Three hypotheses:
theorem trans: A => B, B => C, A |- C

-- Using first-order predicates as hypotheses:
theorem socrates_mortal:
  mortal(socrates) |- mortal(socrates)
```

---

## Proof Blocks

A proof block is delimited by the `proof` and `qed` keywords and contains an
ordered sequence of proof steps. There are three kinds of proof step: `assume`,
`have`, and `exact`.

The checker processes steps in order, maintaining a mutable **environment**: a map
from names to formulas. Initially the environment is empty (the theorem's
hypotheses are not pre-loaded; they must be introduced explicitly with `assume`).
Each step may extend the environment. The proof succeeds if and only if the
formula produced by the final step structurally equals the theorem's declared
conclusion.

### `assume`

**Grammar:**

```ebnf
assume-step ::= 'assume' ident ':' formula
```

**Syntax:**

```
assume <name>: <formula>
```

An `assume` step introduces a hypothesis into scope by binding a name to a
formula. The formula is accepted without any justification — this is the mechanism
by which the theorem's hypotheses (from the sequent) are made available to the
proof. You may introduce any formula you like with `assume`, including formulas
not listed in the sequent, though doing so makes the proof potentially unsound
from a logical standpoint (the checker does not enforce that `assume` steps
correspond to sequent hypotheses).

After an `assume` step, the name `<name>` is bound to `<formula>` in the
environment and is available to all subsequent steps.

**Examples:**

```
assume h:  A AND B
assume ha: A
assume bc: B => C
assume hf: mortal(socrates)
```

### `have`

**Grammar:**

```ebnf
have-step ::= 'have' ident ':' formula ':=' justification
```

**Syntax:**

```
have <name>: <formula> := <justification>
```

A `have` step derives a new formula from existing in-scope names (or axioms) and
binds it to a new name. The verifier:

1. Resolves the justification, producing a derived formula.
2. Checks that the derived formula is **structurally equal** (identical AST) to
   the formula you declared after the colon. If they differ, the step fails with a
   message identifying both the expected formula and what the justification
   actually produced.
3. If they match, binds `<name>` to `<formula>` in the environment.

The name `<name>` must not already be bound in the environment; shadowing is not
permitted (the verifier simply inserts the new binding, which in the current
implementation will overwrite an existing binding without error — rely on unique
names as a matter of style).

**Examples:**

```
have a:      A             := AndElimLeft(h)
have b:      B             := AndElimRight(h)
have ab:     A AND B       := AndIntro(ha, hb)
have goal:   B AND A       := AndIntro(b, a)
have result: B             := ModusPonens(imp, ha)
```

### `exact`

**Grammar:**

```ebnf
exact-step ::= 'exact' justification
```

**Syntax:**

```
exact <justification>
```

An `exact` step closes the proof. It resolves the justification, producing a
formula, and then checks that the formula is structurally equal to the theorem's
declared conclusion. If they match, the proof step succeeds. The verifier then
performs a final check that the last-produced formula in the proof equals the
conclusion; if there is no `exact` step the proof fails.

There may be at most one meaningful `exact` step per theorem because it terminates
proof checking for that theorem. Additional steps after `exact` would never be
reached. A proof with no steps at all fails with "proof has no steps".

**Examples:**

```
exact h                    -- cite a hypothesis by name
exact result               -- cite a previously derived have-step
exact AndIntro(ha, hb)     -- apply a rule directly in the exact position
```

---

## Justifications

A justification is the right-hand side of a `have` step (after `:=`) or the
argument of an `exact` step. It specifies where a formula comes from.

**Grammar:**

```ebnf
justification ::= ident
               |  ident '(' ident { ',' ident } ')'
```

There are two forms:

### Bare Name (Environment and Axiom Lookup)

A bare identifier is looked up in two places, in order:

1. **Environment** — the map of names bound by prior `assume` and `have` steps in
   the current proof. If found, the associated formula is returned.
2. **Axiom map** — the combined map of standard library axioms and user-declared
   `axiom` items. If found, the associated formula is returned.

If the name is not found in either location, the step fails with:

```
unknown name `<name>`: not in scope and not a known axiom
```

**Examples:**

```
exact h          -- h was bound by a preceding assume step
exact identity   -- identity is a standard library axiom
have x: A := h  -- h is an in-scope hypothesis
```

### Rule Application

An identifier followed by a parenthesised, comma-separated list of argument names
is a rule application:

```
RuleName(arg1, arg2, ...)
```

The verifier:

1. Looks up `RuleName` in the built-in rule registry. If not found, fails with
   `unknown rule name`.
2. For each argument name, looks it up in the current environment (not in the
   axiom map). If any argument name is not in scope, fails with `hypothesis
   '<name>' is not in scope`.
3. Collects the formulas bound to the argument names as the ordered list of
   premises.
4. Applies the rule to the premises. If the rule's arity or structural requirements
   are not satisfied, fails with a rule-specific error message.
5. Returns the derived formula.

Rule names are case-sensitive. `andintro` is not a valid rule name; the correct
spelling is `AndIntro`.

**Examples:**

```
have ab: A AND B := AndIntro(ha, hb)
have a:  A       := AndElimLeft(h)
have c:  C       := ModusPonens(bc, b)
exact    AndIntro(b, a)
```

---

## Scoping Rules

The following objects are in scope at any point during proof checking:

1. **Standard library axioms** — always available, loaded before the file is
   processed.
2. **File-level axioms** — every `axiom` declaration that appears **before** the
   current theorem in the source file.
3. **Environment entries** — every `assume` and `have` step that has been
   processed **before** the current step in the current proof block. Steps are
   strictly ordered; a later step cannot reference the name introduced by an
   earlier step that has not yet been reached.

Scope is **flat within a proof**: there is no nesting, no `let` blocks, and no
way to remove a name from scope once it has been introduced. If two steps
introduce the same name, the later binding silently overwrites the earlier one in
the environment map (this is an implementation detail; write proofs with unique
step names to avoid confusion).

Scope does **not** persist between theorems. The environment is created fresh for
each theorem and discarded when the theorem's proof ends. Axiom declarations,
however, persist and accumulate across the entire file.

---

## Built-in Inference Rules

The twelve built-in rules are described below. All argument positions expect names
of formulas currently in the proof environment. The notation `F[X := t]` means
the formula `F` with all free occurrences of variable `X` replaced by the term
(formula) `t`.

| Rule           | Required arguments         | Derived formula     | Notes                                                                   |
|----------------|----------------------------|---------------------|-------------------------------------------------------------------------|
| `AndIntro`     | `a : A`, `b : B`           | `A AND B`           | Exactly 2 arguments.                                                    |
| `AndElimLeft`  | `h : A AND B`              | `A`                 | Exactly 1 argument; must be a conjunction.                              |
| `AndElimRight` | `h : A AND B`              | `B`                 | Exactly 1 argument; must be a conjunction.                              |
| `OrIntroLeft`  | `a : A`, `b : B`           | `A OR B`            | Exactly 2 arguments; `a` supplies the left disjunct.                   |
| `OrIntroRight` | `b : B`, `a : A`           | `A OR B`            | Exactly 2 arguments; first arg is the right disjunct, second the left. |
| `ModusPonens`  | `imp : A => B`, `a : A`    | `B`                 | 2 arguments; accepts either argument order (implication first or second).|
| `ImpliesIntro` | `a : A`, `b : B`           | `A => B`            | Exactly 2 arguments; packages two in-scope formulas into an implication. |
| `NotIntro`     | `a : A`                    | `NOT A`             | Exactly 1 argument.                                                     |
| `NotElim`      | `a : A`, `na : NOT A`      | `⊥`                 | 2 arguments forming a contradiction; either order accepted.             |
| `ForallIntro`  | `x : X` (bare Var), `f : F`| `forall X: F`       | 2 arguments; first must be a bare propositional variable (Var).         |
| `ForallElim`   | `fa : forall X: F`, `t : T`| `F[X := t]`         | 2 arguments; first must be a universal quantification.                  |
| `ExistsIntro`  | `x : X`, `t : T`, `f : F[X := T]` | `exists X: F` | Exactly 3 arguments: variable, witness, already-substituted body.    |

**Note on `NotElim`:** The derived formula is the bottom constant `⊥` (Unicode
U+22A5), represented internally as `Var("⊥")`. A proof that concludes with `⊥`
can be used with `exact` only if the theorem's declared conclusion is also `⊥`.

**Note on `ForallIntro`:** The first argument must resolve to a bare
`Formula::Var` in the environment; the variable name stored in that `Var` becomes
the bound variable of the quantifier.

**Note on `ExistsIntro`:** Three arguments are required. The first is the
quantifier variable (a Var), the second is the witness term, and the third is the
body formula `F[X := t]` (already substituted). The rule wraps the third
argument in `exists X: …` using the variable name from the first argument.

---

## Complete Annotated Example

The following file illustrates all major constructs in one place:

```
-- File: full_example.alif
-- Demonstrates axioms, theorem with multiple hypotheses, all three step kinds,
-- predicate application, and a first-order quantifier.

-- 1. A user axiom, available to all subsequent theorems in this file.
axiom refl: A => A

-- 2. A propositional theorem: commutativity of AND.
--    Sequent: one hypothesis (A AND B), conclusion (B AND A).
theorem and_comm:
  A AND B |- B AND A
proof
  -- Introduce the sole hypothesis by name.
  assume h: A AND B

  -- Derive B from h using the right-elimination rule.
  -- The verifier checks: AndElimRight(h) produces B. Declared type is B. OK.
  have b: B := AndElimRight(h)

  -- Derive A from h using the left-elimination rule.
  have a: A := AndElimLeft(h)

  -- Combine b and a into B AND A via conjunction introduction.
  have result: B AND A := AndIntro(b, a)

  -- Close the proof: result holds B AND A, which equals the conclusion.
  exact result
qed

-- 3. A theorem with three hypotheses demonstrating modus ponens chaining.
theorem transitivity:
  A => B, B => C, A |- C
proof
  assume ab: A => B
  assume bc: B => C
  assume ha: A
  -- ModusPonens accepts either order for the implication and its antecedent.
  have b: B := ModusPonens(ab, ha)
  have c: C := ModusPonens(bc, b)
  exact c
qed

-- 4. A first-order theorem using predicate application.
--    Given mortal(socrates), conclude mortal(socrates).
theorem socrates:
  mortal(socrates) |- mortal(socrates)
proof
  -- mortal(socrates) is stored internally as Var("mortal(socrates)").
  assume h: mortal(socrates)
  exact h
qed

-- 5. No-hypothesis shorthand: the sequent has no |- separator.
--    The single formula A => A becomes the conclusion; hypotheses list is empty.
theorem identity_impl: A => A
proof
  -- We cannot assume A here (no hypothesis was declared in the sequent),
  -- but we can use the refl axiom declared above.
  exact refl
qed
```

Each step in the proof of `and_comm` is verified as follows:

| Step                               | Environment after step               | Check performed                                 |
|------------------------------------|--------------------------------------|-------------------------------------------------|
| `assume h: A AND B`                | `{h => A AND B}`                    | None; formula accepted unconditionally.         |
| `have b: B := AndElimRight(h)`     | `{h => A AND B, b => B}`            | `AndElimRight(A AND B)` produces `B`; equals declared `B`. |
| `have a: A := AndElimLeft(h)`      | `{h => …, b => B, a => A}`          | `AndElimLeft(A AND B)` produces `A`; equals declared `A`. |
| `have result: B AND A := AndIntro(b, a)` | `{…, result => B AND A}`    | `AndIntro(B, A)` produces `B AND A`; equals declared `B AND A`. |
| `exact result`                     | unchanged                            | `result` holds `B AND A`; equals conclusion `B AND A`. |
