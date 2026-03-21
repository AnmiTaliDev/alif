# Inference Rules Reference

This document is the complete, authoritative reference for all twelve inference
rules built into the Alif proof verifier. It covers formal notation, prose
descriptions, arity constraints, error conditions, Alif syntax examples, and
self-contained demonstration theorems for every rule.

---

## Table of Contents

1. [Introduction](#introduction)
2. [How to Reference Rules in Proofs](#how-to-reference-rules-in-proofs)
3. [Substitution: F\[X := t\]](#substitution-fx--t)
4. [Rule Errors](#rule-errors)
5. [AndIntro](#1-andintro)
6. [AndElimLeft](#2-andelimleft)
7. [AndElimRight](#3-andelimright)
8. [OrIntroLeft](#4-orintroleft)
9. [OrIntroRight](#5-orIntroright)
10. [ModusPonens](#6-modusponens)
11. [ImpliesIntro](#7-impliesintro)
12. [NotIntro](#8-notintro)
13. [NotElim](#9-notelim)
14. [ForallIntro](#10-forallintro)
15. [ForallElim](#11-forallelim)
16. [ExistsIntro](#12-existsintro)

---

## Introduction

An **inference rule** is a function from a finite list of already-established
formulas (the *premises*) to a new formula (the *conclusion*). Inference rules
are the computational core of the Alif checker: every `have` step and every
`exact` step that uses the `RuleName(arg, ...)` form is checked by invoking
one of the twelve rules described here.

The checker operates as follows:

1. Parse the source file into an AST (`parse_source`).
2. Load the standard library axioms (`stdlib::load_stdlib`).
3. For each theorem, initialize an environment `Γ` containing the
   theorem's declared hypotheses.
4. Process each proof step in order:
   - `assume name: F` — add `name ↦ F` to `Γ` unconditionally.
   - `have name: F := RuleName(h1, h2, …)` — look up `h1`, `h2`, … in `Γ`,
     call `apply_rule(RuleName, [F(h1), F(h2), …])`, verify that the result
     equals `F`, then add `name ↦ F` to `Γ`.
   - `exact justification` — resolve the justification, verify that the result
     equals the theorem's declared conclusion.
5. After all steps, verify that the last formula produced equals the conclusion.

If any step fails, a `CheckError` (wrapping the zero-based step index, a copy
of the failing step, and a human-readable message) is returned and verification
halts. No step after the failing one is ever executed.

---

## How to Reference Rules in Proofs

Rule names are **case-sensitive PascalCase strings**. The exact spelling must
match one of the twelve canonical names:

```
AndIntro  AndElimLeft  AndElimRight
OrIntroLeft  OrIntroRight
ModusPonens  ImpliesIntro
NotIntro  NotElim
ForallIntro  ForallElim
ExistsIntro
```

A rule is invoked in a `have` step or an `exact` step by writing the rule name
followed by a parenthesised, comma-separated list of *hypothesis names*:

```
have c: A AND B := AndIntro(ha, hb)
exact ModusPonens(impl, ha)
```

Arguments are always **names** bound in the current environment — i.e., names
introduced by earlier `assume` or `have` steps, or by the theorem's own
hypotheses. You cannot pass a formula literal as an argument; if you need an
intermediate formula as a rule argument you must first bind it with `have` or
`assume`.

Misspelling a rule name (e.g. `andintro`, `And_Intro`) causes a `CheckError`
with the message `unknown rule name` rather than a `ParseError`, because the
parser treats the rule name as an ordinary identifier.

---

## Substitution: F\[X := t\]

Two rules — `ForallElim` and `ExistsIntro` — involve the concept of
**substitution**. The notation `F[X := t]` means: produce a new formula by
replacing every *free* occurrence of the variable `X` in formula `F` with the
formula `t`.

### Definition of "free occurrence"

A variable `X` occurs *free* in `F` if it is not bound by an enclosing
quantifier. Specifically:

| Formula form | Free occurrences of `X` |
|---|---|
| `X` (the atom itself) | Yes — the atom `X` is a free occurrence. |
| `Y` (different atom) | No. |
| `A AND B`, `A OR B`, `A => B` | Free occurrences in `A` plus free occurrences in `B`. |
| `NOT A` | Free occurrences in `A`. |
| `forall X: F` | None — `X` is bound here, the whole subformula is left unchanged. |
| `forall Y: F` (Y ≠ X) | Free occurrences in `F`. |
| `exists X: F` | None — same reasoning as `forall`. |
| `exists Y: F` (Y ≠ X) | Free occurrences in `F`. |

### Capture avoidance

The substitution in Alif is **capture-avoiding with respect to binders**. When
`substitute(F, X, t)` recurses into a `forall X: body` or `exists X: body`
sub-formula, it stops and returns that sub-formula unchanged — even if `t`
itself contains `X`. This means the substitution never "captures" a free
variable in `t` by pushing it under a binder.

Concretely:

```
F  = forall Y: X AND Y
X  := Y           -- t = Y
result = forall Y: Y AND Y   -- Y in t is captured under forall Y
```

The current implementation does **not** perform alpha-renaming to prevent the
capture shown above. It simply leaves `forall X: ...` nodes unchanged when the
quantifier variable equals the substitution variable. Authors should choose
distinct variable names to avoid unexpected capture.

### Examples

| F | X | t | F[X := t] |
|---|---|---|---|
| `X` | `X` | `socrates` | `socrates` |
| `X AND Y` | `X` | `a` | `a AND Y` |
| `X AND Y` | `Y` | `b` | `X AND b` |
| `forall X: X` | `X` | `a` | `forall X: X` (binder shadows `X`) |
| `forall Y: X AND Y` | `X` | `a` | `forall Y: a AND Y` |
| `NOT X` | `X` | `P AND Q` | `NOT (P AND Q)` |

---

## Rule Errors

Every rule is a pure function: `apply_rule(rule, premises) -> Result<Formula, RuleError>`.

A `RuleError` has two fields:

| Field | Type | Meaning |
|---|---|---|
| `rule` | `String` | The canonical name of the rule that failed (e.g. `"AndIntro"`). |
| `reason` | `String` | A human-readable explanation of why the rule could not be applied. |

Its `Display` implementation formats as:

```
rule `<rule>` failed: <reason>
```

When a rule error occurs inside a `have` or `exact` step, the checker wraps it
in a `CheckError` whose `message` is the `RuleError`'s `Display` string. The
`CheckError` additionally carries the zero-based step index and a copy of the
failing step. See [errors.md](errors.md) for the full error hierarchy.

The table below lists every error message string that can appear as the `reason`
field, organized by rule. The exact strings are significant — they are produced
literally by `apply_rule` and appear verbatim in checker output.

| Rule | reason string |
|---|---|
| `AndIntro` | `AndIntro requires exactly 2 premises` |
| `AndElimLeft` | `AndElimLeft requires exactly 1 premise` |
| `AndElimLeft` | `premise must be a conjunction (A AND B)` |
| `AndElimRight` | `AndElimRight requires exactly 1 premise` |
| `AndElimRight` | `premise must be a conjunction (A AND B)` |
| `OrIntroLeft` | `OrIntroLeft requires 2 arguments: the left formula and the right formula` |
| `OrIntroRight` | `OrIntroRight requires 2 arguments: the left formula and the right formula` |
| `ModusPonens` | `ModusPonens requires exactly 2 premises: (A => B) and A` |
| `ModusPonens` | `second premise does not match the antecedent of the implication` |
| `ModusPonens` | `first premise does not match the antecedent of the implication` |
| `ModusPonens` | `one premise must be an implication (A => B)` |
| `ImpliesIntro` | `ImpliesIntro requires 2 premises: the hypothesis A and the conclusion B` |
| `NotElim` | `NotElim requires exactly 2 premises: A and NOT A (or vice versa)` |
| `NotElim` | `premises must be a formula and its negation` |
| `NotIntro` | `NotIntro requires 1 premise: the formula A to be negated` |
| `ForallElim` | `ForallElim requires 2 premises: (forall X: F) and the term to substitute` |
| `ForallElim` | `first premise must be a universal quantification (forall X: F)` |
| `ForallIntro` | `ForallIntro requires 2 premises: the variable name (as Var) and the formula` |
| `ForallIntro` | `first premise must be a variable name` |
| `ExistsIntro` | `ExistsIntro requires 3 premises: the variable, the witness term, and the formula F[X:=t]` |
| `ExistsIntro` | `first premise must be a variable name` |

In addition, any rule application where an argument name is not in scope
produces the error (through `resolve_rule`):

```
rule `<RuleName>` failed: hypothesis `<name>` is not in scope
```

And if the rule name string does not match any known rule:

```
rule `<name>` failed: unknown rule name
```

---

## 1. AndIntro

### Formal notation

```
    Γ ⊢ A    Γ ⊢ B
    ─────────────── AndIntro
      Γ ⊢ A ∧ B
```

### Description

From a proof of `A` and a proof of `B`, produce a proof of their conjunction
`A AND B`. This is the standard introduction rule for the logical connective
AND (∧). It takes exactly two premises in any order; the left premise becomes
the left conjunct and the right premise becomes the right conjunct.

### Required premises

**Exactly 2.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | Any formula `A` | Becomes the left conjunct. |
| 2 | Any formula `B` | Becomes the right conjunct. |

No structural constraint is placed on either premise. Any formula is acceptable.

### Conclusion

`A AND B` — a `Formula::And` node whose left child is the first premise and
whose right child is the second premise.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 2 | `AndIntro requires exactly 2 premises` |

### Alif syntax example

```
have c: A AND B := AndIntro(ha, hb)
```

Here `ha` must be bound to formula `A` and `hb` to formula `B` in the current
environment. The checker verifies that the result `A AND B` equals the declared
type annotation `A AND B`.

### Minimal complete demonstration

```
-- Prove that from A and B we can derive A AND B.
theorem and_introduction:
  A, B |- A AND B
proof
  assume ha: A
  assume hb: B
  have c: A AND B := AndIntro(ha, hb)
  exact c
qed
```

---

## 2. AndElimLeft

### Formal notation

```
    Γ ⊢ A ∧ B
    ─────────── AndElimLeft
       Γ ⊢ A
```

### Description

From a proof of the conjunction `A AND B`, extract a proof of the left conjunct
`A`. This is the left projection of the conjunction elimination rule. It
requires the premise to be a conjunction; any other shape is rejected.

### Required premises

**Exactly 1.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | `A AND B` (a conjunction) | The left child `A` is returned as the conclusion. |

### Conclusion

`A` — the left child of the conjunction premise.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 1 | `AndElimLeft requires exactly 1 premise` |
| Premise is not a conjunction | `premise must be a conjunction (A AND B)` |

### Alif syntax example

```
have a: A := AndElimLeft(h)
```

`h` must be bound to a formula of the form `A AND B`. The checker verifies
that the extracted left child equals the declared annotation `A`.

### Minimal complete demonstration

```
-- Extract the left component of a conjunction.
theorem and_elim_left_demo:
  A AND B |- A
proof
  assume h: A AND B
  have a: A := AndElimLeft(h)
  exact a
qed
```

---

## 3. AndElimRight

### Formal notation

```
    Γ ⊢ A ∧ B
    ─────────── AndElimRight
       Γ ⊢ B
```

### Description

From a proof of the conjunction `A AND B`, extract a proof of the right
conjunct `B`. This is the right projection of the conjunction elimination rule.
Symmetric to `AndElimLeft`.

### Required premises

**Exactly 1.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | `A AND B` (a conjunction) | The right child `B` is returned as the conclusion. |

### Conclusion

`B` — the right child of the conjunction premise.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 1 | `AndElimRight requires exactly 1 premise` |
| Premise is not a conjunction | `premise must be a conjunction (A AND B)` |

### Alif syntax example

```
have b: B := AndElimRight(h)
```

`h` must be bound to a formula of the form `A AND B`. The checker verifies
that the extracted right child equals the declared annotation `B`.

### Minimal complete demonstration

```
-- Extract the right component of a conjunction.
theorem and_elim_right_demo:
  A AND B |- B
proof
  assume h: A AND B
  have b: B := AndElimRight(h)
  exact b
qed
```

---

## 4. OrIntroLeft

### Formal notation

```
    Γ ⊢ A
    ─────────── OrIntroLeft  (B arbitrary)
    Γ ⊢ A ∨ B
```

### Description

Given a proof of formula `A`, construct a proof of the disjunction `A OR B`
for any formula `B`. This is the left injection into a disjunction.

**Important implementation note:** unlike the classical rule that takes only
one premise and a target right formula, the Alif implementation requires **two
premises**: the left formula `A` and the right formula `B`. Both must already
be in scope as named formulas. The result is `A OR B` with `A` on the left.

### Required premises

**Exactly 2.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | Any formula `A` | Becomes the left disjunct. |
| 2 | Any formula `B` | Becomes the right disjunct. |

### Conclusion

`A OR B` — a `Formula::Or` node with the first premise on the left and the
second premise on the right.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 2 | `OrIntroLeft requires 2 arguments: the left formula and the right formula` |

### Alif syntax example

```
have disj: A OR B := OrIntroLeft(ha, hb)
```

`ha` is bound to `A` and `hb` is bound to `B`. The result is `A OR B`.

### Minimal complete demonstration

```
-- Construct a disjunction by left injection.
theorem or_intro_left_demo:
  A, B |- A OR B
proof
  assume ha: A
  assume hb: B
  have disj: A OR B := OrIntroLeft(ha, hb)
  exact disj
qed
```

---

## 5. OrIntroRight

### Formal notation

```
    Γ ⊢ B
    ─────────── OrIntroRight  (A arbitrary)
    Γ ⊢ A ∨ B
```

### Description

Given a proof of formula `B`, construct a proof of the disjunction `A OR B`.
This is the right injection into a disjunction.

**Important implementation note:** like `OrIntroLeft`, this rule takes **two
premises**. However, the argument order is **reversed**: the first argument is
the *right* disjunct `B` (the one already proved) and the second argument is
the *left* disjunct `A`. The result places the second argument on the left and
the first argument on the right, producing `A OR B`.

Concretely: `OrIntroRight(hb, ha)` where `hb : B` and `ha : A` yields `A OR B`.

### Required premises

**Exactly 2.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | Any formula `B` | The proved right disjunct — goes to the **right** of OR. |
| 2 | Any formula `A` | The arbitrary left disjunct — goes to the **left** of OR. |

### Conclusion

`A OR B` — `Formula::Or(premises[1], premises[0])`. Note the swap.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 2 | `OrIntroRight requires 2 arguments: the left formula and the right formula` |

### Alif syntax example

```
have disj: A OR B := OrIntroRight(hb, ha)
```

`hb` is bound to `B`, `ha` is bound to `A`. The result is `A OR B`.

### Minimal complete demonstration

```
-- Construct a disjunction by right injection.
theorem or_intro_right_demo:
  A, B |- A OR B
proof
  assume hb: B
  assume ha: A
  have disj: A OR B := OrIntroRight(hb, ha)
  exact disj
qed
```

---

## 6. ModusPonens

### Formal notation

```
    Γ ⊢ A ⇒ B    Γ ⊢ A
    ─────────────────── ModusPonens
           Γ ⊢ B
```

### Description

From a proof of the implication `A => B` and a proof of the antecedent `A`,
derive the consequent `B`. This is the classical *modus ponens* (MP) rule,
also called *implication elimination*.

**Order flexibility:** Alif's `ModusPonens` implementation tries both orderings:
if the first premise is the implication, the second must be the antecedent; if
neither premise is an implication, an error is produced. Both orderings
`ModusPonens(impl, ha)` and `ModusPonens(ha, impl)` are accepted as long as one
of the two premises is an `A => B` formula whose antecedent matches the other.

### Required premises

**Exactly 2.**

### Premise shape constraints

The rule accepts either order:

**Order A (first premise is the implication):**

| # | Must be | Notes |
|---|---|---|
| 1 | `A => B` (an implication) | The antecedent `A` must equal premise 2. |
| 2 | `A` | Must equal the antecedent of the implication in premise 1. |

**Order B (second premise is the implication):**

| # | Must be | Notes |
|---|---|---|
| 1 | `A` | Must equal the antecedent of the implication in premise 2. |
| 2 | `A => B` (an implication) | The antecedent `A` must equal premise 1. |

### Conclusion

`B` — the consequent of the matched implication.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 2 | `ModusPonens requires exactly 2 premises: (A => B) and A` |
| First premise is implication but second ≠ antecedent | `second premise does not match the antecedent of the implication` |
| Second premise is implication but first ≠ antecedent | `first premise does not match the antecedent of the implication` |
| Neither premise is an implication | `one premise must be an implication (A => B)` |

### Alif syntax example

```
have b: B := ModusPonens(impl, ha)
```

`impl` is bound to `A => B` and `ha` is bound to `A`. The result is `B`.

### Minimal complete demonstration

```
-- Transitivity of implication using ModusPonens twice.
theorem impl_transitivity:
  A => B, B => C, A |- C
proof
  assume ab: A => B
  assume bc: B => C
  assume ha: A
  have b:  B := ModusPonens(ab, ha)
  have c:  C := ModusPonens(bc, b)
  exact c
qed
```

---

## 7. ImpliesIntro

### Formal notation

```
    Γ, A ⊢ B
    ────────── ImpliesIntro
    Γ ⊢ A ⇒ B
```

### Description

From a proof of `A` (the hypothesis) and a proof of `B` (the conclusion),
construct the implication `A => B`. This is the introduction rule for
implication.

**Semantic note:** in a full natural deduction system, this rule would
*discharge* the assumption `A` from the context and package the sub-proof into
an implication. In Alif's flat proof model there is no sub-proof structure, so
`ImpliesIntro` is instead a combinator: it simply combines two formulas already
in scope into an implication. Both premises remain in the environment; neither
is discharged.

### Required premises

**Exactly 2.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | Any formula `A` | Becomes the antecedent (left side) of `=>`. |
| 2 | Any formula `B` | Becomes the consequent (right side) of `=>`. |

### Conclusion

`A => B` — a `Formula::Implies` node.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 2 | `ImpliesIntro requires 2 premises: the hypothesis A and the conclusion B` |

### Alif syntax example

```
have impl: A => B := ImpliesIntro(ha, hb)
```

`ha` is bound to `A` and `hb` is bound to `B`. The result is `A => B`.

### Minimal complete demonstration

```
-- Build an implication from two independently-known formulas.
theorem implies_intro_demo:
  A, B |- A => B
proof
  assume ha: A
  assume hb: B
  have impl: A => B := ImpliesIntro(ha, hb)
  exact impl
qed
```

---

## 8. NotIntro

### Formal notation

```
    Γ ⊢ A
    ─────── NotIntro
    Γ ⊢ ¬A
```

### Description

From a proof of formula `A`, construct its negation `NOT A`. This is the
introduction rule for negation.

**Semantic note:** in classical or intuitionistic logic, negation introduction
typically requires a sub-proof showing that `A` leads to contradiction. In
Alif's flat model, `NotIntro` is a primitive combinator that simply wraps any
formula in a `NOT` constructor. Authors should use this rule with care and
ensure that the formula being negated genuinely represents an absurdity in the
context of the proof.

### Required premises

**Exactly 1.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | Any formula `A` | Becomes the operand of `NOT`. |

### Conclusion

`NOT A` — a `Formula::Not` node wrapping the premise.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 1 | `NotIntro requires 1 premise: the formula A to be negated` |

### Alif syntax example

```
have neg: NOT A := NotIntro(ha)
```

`ha` is bound to formula `A`. The result is `NOT A`.

### Minimal complete demonstration

```
-- Wrap a formula in negation.
theorem not_intro_demo:
  A |- NOT A => NOT A
proof
  assume ha: A
  have na: NOT A := NotIntro(ha)
  have result: NOT A => NOT A := ImpliesIntro(na, na)
  exact result
qed
```

---

## 9. NotElim

### Formal notation

```
    Γ ⊢ A    Γ ⊢ ¬A
    ───────────────── NotElim
          Γ ⊢ ⊥
```

### Description

From a proof of `A` and a proof of its negation `NOT A` (or vice versa),
derive the contradiction symbol `⊥` (bottom, the Unicode character U+22A5,
represented as the atom `Formula::Var("⊥")`). This is the *ex contradictione*
rule, also known as contradiction elimination or the law of non-contradiction.

The result `⊥` can subsequently be used with `exact` to close any goal, or can
be passed to further rules. Note that Alif does not automatically derive any
arbitrary formula from `⊥`; `⊥` is just a distinguished atom.

**Order flexibility:** the rule accepts the formula and its negation in either
order — `NotElim(ha, hna)` and `NotElim(hna, ha)` are both valid.

### Required premises

**Exactly 2.**

### Premise shape constraints

The rule accepts either of these two orderings:

| Ordering | Premise 1 | Premise 2 |
|---|---|---|
| A | Any formula `A` | `NOT A` (negation of premise 1) |
| B | `NOT A` | Any formula `A` (negation of premise 1) |

### Conclusion

`⊥` — the atom `Formula::Var("⊥")` (U+22A5 ASSERTION / BOTTOM symbol).

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 2 | `NotElim requires exactly 2 premises: A and NOT A (or vice versa)` |
| Premises are not a formula and its negation | `premises must be a formula and its negation` |

### Alif syntax example

```
have bot: ⊥ := NotElim(ha, hna)
```

`ha` is bound to `A` and `hna` is bound to `NOT A`. The result is `⊥`.

Because `⊥` is a valid identifier character sequence in Alif (it is treated as
an `Ident` by the lexer if present as a UTF-8 identifier), you can write the
bottom symbol directly in formula annotations:

```
have bot: ⊥ := NotElim(ha, hna)
```

### Minimal complete demonstration

```
-- From A and NOT A, derive bottom.
theorem contradiction_demo:
  A, NOT A |- ⊥
proof
  assume ha: A
  assume hna: NOT A
  have bot: ⊥ := NotElim(ha, hna)
  exact bot
qed
```

---

## 10. ForallIntro

### Formal notation

```
    Γ ⊢ F  (X not free in any hypothesis of Γ)
    ──────────────────────────────────────────── ForallIntro
                  Γ ⊢ ∀X. F
```

### Description

Given a variable name `X` (provided as a `Formula::Var` atom) and a formula
`F`, construct the universally quantified formula `forall X: F`. This is the
introduction rule for universal quantification.

**Semantic note:** in a complete first-order logic, `ForallIntro` is only
sound when the variable `X` does not appear free in any undischarged hypothesis.
Alif's flat proof model does not enforce this side condition mechanically —
authors are responsible for ensuring that `X` is genuinely arbitrary in the
context in which `ForallIntro` is applied.

**Argument note:** the first argument to `ForallIntro` must be a name bound to
a `Formula::Var` atom — i.e., a step like `assume X: X` or `have X: X :=
...` must appear before `ForallIntro` is called. The string stored in the `Var`
becomes the bound variable of the `forall`.

### Required premises

**Exactly 2.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | `Formula::Var(name)` — a simple atom | The string `name` becomes the bound variable in `forall name: ...`. |
| 2 | Any formula `F` | The body of the universal quantification. |

### Conclusion

`forall X: F` — a `Formula::Forall(X, F)` node.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 2 | `ForallIntro requires 2 premises: the variable name (as Var) and the formula` |
| First premise is not a `Var` atom | `first premise must be a variable name` |

### Alif syntax example

```
assume vx: X
have px: P := ...  -- some proof of P that holds for X
have fa: forall X: P := ForallIntro(vx, px)
```

### Minimal complete demonstration

```
-- Universally quantify a formula over X.
theorem forall_intro_demo:
  X, P |- forall X: P
proof
  assume vx: X
  assume px: P
  have fa: forall X: P := ForallIntro(vx, px)
  exact fa
qed
```

---

## 11. ForallElim

### Formal notation

```
    Γ ⊢ ∀X. F    Γ ⊢ t
    ─────────────────── ForallElim
         Γ ⊢ F[X := t]
```

### Description

Given a proof of `forall X: F` and a term `t` (provided as any formula in
scope), derive `F[X := t]` — the body `F` with every free occurrence of `X`
replaced by `t`. This is the elimination rule for universal quantification,
also called universal instantiation.

The substitution `F[X := t]` is described in full in the
[Substitution section](#substitution-fx--t) above. The checker computes the
result of the substitution and verifies that it equals the type annotation on
the `have` step.

### Required premises

**Exactly 2.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | `forall X: F` (a universal quantification) | The bound variable `X` and body `F` are extracted. |
| 2 | Any formula `t` | Used as the substituted term. |

### Conclusion

`F[X := t]` — the result of substituting `t` for every free occurrence of `X`
in `F`.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 2 | `ForallElim requires 2 premises: (forall X: F) and the term to substitute` |
| First premise is not a `forall` | `first premise must be a universal quantification (forall X: F)` |

### Alif syntax example

```
-- h is bound to: forall X: mortal(X)
-- t  is bound to: socrates
have ms: mortal(socrates) := ForallElim(h, t)
```

Here `F = mortal(X)`, `X = X`, `t = socrates`, so `F[X := socrates] = mortal(socrates)`.

### Minimal complete demonstration

```
-- Instantiate a universal statement at a concrete term.
theorem forall_elim_demo:
  forall X: mortal(X), socrates |- mortal(socrates)
proof
  assume hall: forall X: mortal(X)
  assume t: socrates
  have ms: mortal(socrates) := ForallElim(hall, t)
  exact ms
qed
```

The classic Socrates syllogism can be derived by combining `ForallElim` with
`ModusPonens`:

```
-- All humans are mortal; Socrates is human; therefore Socrates is mortal.
theorem socrates:
  forall X: human(X) => mortal(X), human(socrates) |- mortal(socrates)
proof
  assume all_hm: forall X: human(X) => mortal(X)
  assume hs: human(socrates)
  assume wit: socrates
  have impl: human(socrates) => mortal(socrates) := ForallElim(all_hm, wit)
  have ms: mortal(socrates) := ModusPonens(impl, hs)
  exact ms
qed
```

---

## 12. ExistsIntro

### Formal notation

```
         Γ ⊢ F[X := t]
    ──────────────────────── ExistsIntro
         Γ ⊢ ∃X. F
```

### Description

Given a variable name `X`, a witness term `t`, and a proof of `F[X := t]`
(the formula `F` instantiated at the witness `t`), construct the existentially
quantified statement `exists X: F`. This is the introduction rule for
existential quantification, also called existential generalization.

In other words: to prove "there exists an `X` such that `F(X)` holds", provide
a concrete witness `t` for which you already hold a proof of `F(t)`.

**Argument count:** `ExistsIntro` requires **three** arguments, which is unique
among the twelve rules:

1. The bound variable name (as a `Var` atom).
2. The witness term `t` (any formula; the checker does not use it for
   computation but requires it to be present for explicitness).
3. The proof of `F[X := t]` (the instantiated body).

The rule extracts the variable name from argument 1 and the body from argument
3 to construct `exists X: body`.

### Required premises

**Exactly 3.**

### Premise shape constraints

| # | Must be | Notes |
|---|---|---|
| 1 | `Formula::Var(name)` — a simple atom | The string `name` becomes the bound variable of `exists`. |
| 2 | Any formula `t` | The witness term. Present for documentation; not used in output computation. |
| 3 | Any formula `F[X := t]` | The instantiated body. Becomes the body of the existential. |

### Conclusion

`exists X: F[X := t]` — a `Formula::Exists(X, F[X := t])` node, where `X` is
the variable name from premise 1 and `F[X := t]` is premise 3.

Note: the checker does not verify that premise 3 is actually equal to the
result of substituting premise 2 into some formula. The body of the existential
is literally taken as premise 3. Authors must ensure this is semantically
correct.

### Error conditions

| Condition | `reason` string |
|---|---|
| Number of premises ≠ 3 | `ExistsIntro requires 3 premises: the variable, the witness term, and the formula F[X:=t]` |
| First premise is not a `Var` atom | `first premise must be a variable name` |

### Alif syntax example

```
assume vx: X
assume wit: socrates
have hs: human(socrates) := ...  -- proof of F[X := socrates]
have ex: exists X: human(socrates) := ExistsIntro(vx, wit, hs)
```

### Minimal complete demonstration

```
-- From a concrete fact, derive the existential statement.
theorem exists_intro_demo:
  human(socrates) |- exists X: human(socrates)
proof
  assume hs: human(socrates)
  assume vx: X
  assume wit: socrates
  have ex: exists X: human(socrates) := ExistsIntro(vx, wit, hs)
  exact ex
qed
```
