# Inference rules

This document describes how the checker treats a proof and defines the 21 built-in rules. Syntax is defined in [syntax.md](syntax.md).

## Proof model

Every name defined in a proof stands for a fact. A fact has three parts:

- a formula,
- a set of dependencies: the names of the assumptions the formula was derived from,
- a flag that says whether the fact is an assumption.

| Step | Resulting fact |
|------|----------------|
| `assume h: F` | formula `F`, dependencies `{h}`, assumption |
| `have h: F := J` | formula `F`, dependencies taken from `J`, not an assumption |
| axiom used in a justification | the axiom formula, no dependencies, not an assumption |

A rule computes the dependencies of its result from the dependencies of its arguments. Most rules take the union. The rules `ImpliesIntro`, `NotIntro`, `OrElim` and `ExistsElim` remove assumptions from the result. This is called discharging an assumption.

`have h: F := J` succeeds when the formula derived by `J` equals `F`. `have h: F := x`, where `x` is a name, copies the fact `x` under a new name. The copy is not an assumption, so it cannot be discharged. The original can.

`exact J` succeeds when both conditions hold:

1. The formula derived by `J` equals the conclusion of the theorem.
2. Every name in the dependencies of that fact is an assumption whose formula equals one of the hypotheses of the theorem.

The second condition is what ties a proof to its statement. An assumption that is not a hypothesis has to be discharged before `exact`. Without this condition `assume h: B` followed by `exact h` would prove any `B`.

Formulas are compared up to renaming of bound variables.

## Argument kinds

| Kind | Meaning |
|------|---------|
| fact | a name of a fact of the proof or of an axiom. A proof name takes precedence over an axiom name. |
| assumption | a name introduced by `assume`. Names introduced by `have` are rejected, even when they are copies of assumptions. |
| term | an identifier or a function application, such as `a` or `f(a, b)` |
| variable | a plain identifier |

## Goal-directed rules

The result of some rules is not determined by their arguments. For these rules the formula declared in `have` or the conclusion of the theorem in `exact` is the goal. The rule checks that the goal is reachable and returns it.

| Rule | What the goal supplies |
|------|------------------------|
| `OrIntroLeft`, `OrIntroRight` | the other disjunct |
| `FalseElim` | the derived formula |
| `ExistsIntro` | the quantified formula |
| `EqRefl` | the term |
| `EqSubst` | the formula after replacement |

## Summary

| Rule | Arguments | Result |
|------|-----------|--------|
| `AndIntro` | fact `A`, fact `B` | `A AND B` |
| `AndElimLeft` | fact `A AND B` | `A` |
| `AndElimRight` | fact `A AND B` | `B` |
| `OrIntroLeft` | fact `A` | goal `A OR B` |
| `OrIntroRight` | fact `B` | goal `A OR B` |
| `OrElim` | fact `P OR Q`, assumption `P`, fact `C`, assumption `Q`, fact `C` | `C` |
| `ModusPonens` | fact `A => B`, fact `A` | `B` |
| `ImpliesIntro` | assumption `A`, fact `B` | `A => B` |
| `NotIntro` | assumption `A`, fact `FALSE` | `NOT A` |
| `NotElim` | fact `A`, fact `NOT A` | `FALSE` |
| `FalseElim` | fact `FALSE` | goal |
| `IffIntro` | fact `A => B`, fact `B => A` | `A <=> B` |
| `IffElim` | fact `A <=> B`, fact `A` or `B` | the other side |
| `ForallIntro` | fact `F`, variable `X` | `forall X: F` |
| `ForallElim` | fact `forall X: F`, term `t` | `F[X := t]` |
| `ExistsIntro` | fact `F[X := t]`, term `t` | goal `exists X: F` |
| `ExistsElim` | fact `exists X: F`, assumption `F[X := w]`, fact `C`, variable `w` | `C` |
| `EqRefl` | none | goal `t = t` |
| `EqSym` | fact `s = t` | `t = s` |
| `EqTrans` | fact `s = t`, fact `t = u` | `s = u` |
| `EqSubst` | fact `s = t`, fact `F` | goal |

A wrong number of arguments is an error: ``rule `AndIntro` failed: expects 2 arguments, got 1``.

## Conjunction

`AndIntro(a, b)` derives `A AND B` from facts `a: A` and `b: B`.

`AndElimLeft(h)` and `AndElimRight(h)` derive the left and the right part of a fact `h: A AND B`.

The dependencies of the result are the union of the dependencies of the arguments.

## Disjunction

`OrIntroLeft(a)` derives a goal of the form `A OR B` from `a: A`. The left side of the goal has to equal the formula of `a`. `B` is free.

`OrIntroRight(b)` derives a goal of the form `A OR B` from `b: B`. The right side of the goal has to equal the formula of `b`.

`OrElim(o, x, cx, y, cy)` is proof by cases. The arguments are:

- `o`: a fact `P OR Q`,
- `x`: an assumption with formula `P`,
- `cx`: a fact `C` derived from `x`,
- `y`: an assumption with formula `Q`,
- `cy`: a fact `C` derived from `y`.

The formulas of `cx` and `cy` have to be equal. The result is `C`. Its dependencies are the dependencies of `o`, the dependencies of `cx` without `x`, and the dependencies of `cy` without `y`.

```
theorem or_to_implication:
  A OR B, NOT A |- B
proof
  assume o: A OR B
  assume na: NOT A
  assume x: A
  have f: FALSE := NotElim(x, na)
  have bx: B := FalseElim(f)
  assume y: B
  have result: B := OrElim(o, x, bx, y, y)
  exact result
qed
```

## Implication

`ModusPonens(i, a)` derives `B` from `i: A => B` and `a: A`. The two arguments can be given in either order.

`ImpliesIntro(x, y)` derives `A => B`, where `A` is the formula of the assumption `x` and `B` is the formula of the fact `y`. The result depends on the dependencies of `y` without `x`.

```
theorem const_fn:
  |- A => B => A
proof
  assume a: A
  assume b: B
  have inner: B => A := ImpliesIntro(b, a)
  have outer: A => B => A := ImpliesIntro(a, inner)
  exact outer
qed
```

`ImpliesIntro` does not require that `y` depends on `x`. If it does not, the implication is derived with a vacuous antecedent, which is valid.

## Negation and falsity

`NotElim(a, n)` derives `FALSE` from a fact and its negation. The arguments can be given in either order.

`NotIntro(x, c)` derives `NOT A` from the assumption `x: A` and a fact `c` whose formula is `FALSE`. The result depends on the dependencies of `c` without `x`.

`FalseElim(f)` derives the goal from a fact `f: FALSE`. The goal can be any formula.

```
theorem modus_tollens: A => B, NOT B |- NOT A
proof
  assume i: A => B
  assume nb: NOT B
  assume a: A
  have b: B := ModusPonens(i, a)
  have f: FALSE := NotElim(b, nb)
  have r: NOT A := NotIntro(a, f)
  exact r
qed
```

The built-in rules contain no classical principle such as double negation elimination or the law of excluded middle. A proof that needs one has to take it as an axiom or as a hypothesis.

## Equivalence

`IffIntro(f, b)` derives `P <=> Q` from `f: P => Q` and `b: Q => P`.

`IffElim(i, x)` takes `i: P <=> Q`. When `x` equals `P` the result is `Q`. Otherwise, when `x` equals `Q`, the result is `P`.

## Universal quantifier

`ForallIntro(h, X)` derives `forall X: F` from a fact `h: F`. The argument `X` has to be a plain name. The rule fails when `X` is free in an assumption that `h` depends on, or free in any axiom. See [Variable conditions](#variable-conditions).

`ForallElim(u, t)` takes `u: forall X: F` and a term `t`. The result is `F` with `t` substituted for `X`. The substitution avoids capture, see [Substitution](#substitution).

## Existential quantifier

`ExistsIntro(h, t)` derives a goal of the form `exists X: F` from `h: F[X := t]`. The checker substitutes `t` for `X` in `F` and compares the result with the formula of `h`.

`ExistsElim(e, x, c, w)` is existential elimination. The arguments are:

- `e`: a fact `exists X: F`,
- `x`: an assumption whose formula equals `F[X := w]`,
- `c`: a fact `C` derived from `x`,
- `w`: a plain name, the witness.

The result is `C`. Its dependencies are the dependencies of `e` and the dependencies of `c` without `x`. The witness has to be fresh, see [Variable conditions](#variable-conditions).

```
theorem exists_left:
  exists X: P(X) AND Q(X) |- exists X: P(X)
proof
  assume ex: exists X: P(X) AND Q(X)
  assume w: P(c) AND Q(c)
  have p: P(c) := AndElimLeft(w)
  have goal: exists X: P(X) := ExistsIntro(p, c)
  have result: exists X: P(X) := ExistsElim(ex, w, goal, c)
  exact result
qed
```

## Equality

`EqRefl()` derives a goal of the form `t = t`. The two sides have to be identical. The rule has no arguments and no dependencies.

`EqSym(h)` derives `t = s` from `h: s = t`.

`EqTrans(h1, h2)` derives `s = u` from `h1: s = t` and `h2: t = u`. The middle terms have to be identical.

`EqSubst(e, h)` takes `e: s = t` and a fact `h`. The goal has to be the formula of `h` with some free occurrences of `s` replaced by `t`. Occurrences of `s` that are not replaced stay as they are. A replacement under a quantifier fails when the quantified variable is free in `s` or in `t`. Replacement from right to left needs `EqSym` first.

```
theorem eq_substitution:
  a = b, P(a) |- P(b)
proof
  assume e: a = b
  assume p: P(a)
  have r: P(b) := EqSubst(e, p)
  exact r
qed
```

## Variable conditions

A name that occurs free in a hypothesis or in an axiom stands for a fixed object. Generalising over such a name would be unsound, so two rules check that their variable is not fixed.

A name is fixed for a fact when it is free in the formula of an assumption that the fact depends on, or free in any axiom of the run. The check covers every axiom, not only the axioms used in the proof.

`ForallIntro(h, X)` requires that `X` is not fixed for `h`.

`ExistsElim(e, x, c, w)` requires that `w` is not free in the formula of `e`, not free in the formula of `c`, and not fixed for the dependencies of the result.

Failure messages:

```
rule `ForallIntro` failed: variable `x` occurs free in an undischarged assumption or an axiom
rule `ExistsElim` failed: witness `c` is not fresh: it occurs in the existential formula, the conclusion, an undischarged assumption or an axiom
```

## Substitution

`ForallElim`, `ExistsIntro` and `ExistsElim` substitute a term for a variable. The substitution replaces free occurrences only. A bound occurrence of the same name is left alone.

When the term contains a name that a quantifier inside the formula binds, the substitution renames the bound variable first. The new name is the old name followed by `_1`, or `_2` and so on, with the first number that does not clash with any name in the formula or in the term.

Substituting `Y` for `X` in `exists Y: R(X, Y)` gives `exists Y_1: R(Y, Y_1)`. This formula equals `exists Z: R(Y, Z)` and differs from `exists Y: R(Y, Y)`.

## Applying theorems

A theorem that was declared earlier can be used as a rule. For a theorem `name: H1, ..., Hn |- C` the justification is `name(a1, ..., an)` with one fact for each hypothesis. A theorem without hypotheses is used as `name()` or as the bare name `name`.

```
theorem swap_twice:
  P AND Q |- P AND Q
proof
  assume h: P AND Q
  have swapped: Q AND P := and_comm(h)
  have back: P AND Q := and_comm(swapped)
  exact back
qed
```

The statement of the theorem is a pattern. Propositional atoms, which are identifiers without arguments, are placeholders. The check finds one replacement of every placeholder by a formula such that:

- the formula of each argument `ai` equals `Hi` after the replacement,
- the declared formula equals `C` after the replacement.

The same placeholder has to be replaced by equal formulas everywhere. In `and_comm: A AND B |- B AND A` the call `and_comm(h)` with `h: P AND Q` replaces `A` by `P` and `B` by `Q`, and the declared formula has to be `Q AND P`.

Everything else in the pattern has to match exactly: connectives, predicate names, terms, equalities and free names. Bound variables are matched up to renaming. A replacement must not contain a name that is bound by a quantifier around the placeholder in the pattern. Without this restriction `P |- forall X: P` could be applied with `Q(X)` for `P`.

The dependencies of the result are the union of the dependencies of the arguments. When the arguments do not fit, the message is:

```
the arguments and the declared formula are not an instance of theorem `and_comm`: (A AND B) |- (B AND A)
```

A theorem can be applied only after its declaration. Axioms are not patterns: their formulas are used as they are written.
