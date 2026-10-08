# Inference rules

This document defines how a proof is checked and what each of the 21 built-in rules does. File syntax is in [syntax.md](syntax.md).

## Concepts

### Facts

Every step of a proof defines a name for a fact. A fact has:

- a formula,
- dependencies: the set of assumptions it rests on,
- a mark that tells whether it is an assumption.

| Step | Fact |
|------|------|
| `assume h: F` | formula `F`, dependencies `{h}`, marked as an assumption |
| `have h: F := J` | formula `F`, dependencies of `J`, not marked |

An axiom used as an argument or as a justification is a fact with no dependencies. A name defined in the proof wins over an axiom of the same name.

`have h: F := x`, where `x` is a name, copies the fact `x`. The copy is not marked as an assumption. It cannot be discharged, but `x` still can.

### Dependencies

A rule computes the dependencies of its result from its arguments. Unless a rule says otherwise, they are the union of the dependencies of all fact arguments. `ImpliesIntro`, `NotIntro`, `OrElim` and `ExistsElim` remove an assumption from the set. Removing it is called discharging.

### Acceptance

`have h: F := J` is accepted when the formula derived by `J` equals `F`.

`exact J` is accepted when:

1. the formula derived by `J` equals the conclusion of the theorem, and
2. every assumption in the dependencies of the result has a formula that equals one of the hypotheses of the theorem.

The second condition connects the proof to the statement. If it were absent, `assume h: B` followed by `exact h` would prove any `B`.

### Goals

For some rules the arguments do not determine the result. These rules use the declared formula of `have`, or the conclusion in the case of `exact`, as a goal. They check that the goal is reachable and return it: `OrIntroLeft`, `OrIntroRight`, `FalseElim`, `ExistsIntro`, `EqRefl`, `EqSubst`.

### Argument kinds

| Kind | Accepts |
|------|---------|
| fact | name of a step of the proof, or of an axiom |
| assumption | name of a step made by `assume`. A step made by `have` is rejected, even if it copies an assumption. |
| term | identifier or function application |
| variable | identifier |

A wrong number of arguments fails the rule.

## Rule reference

In the entries, `A`, `B`, `C`, `F`, `P`, `Q` stand for arbitrary formulas.

### Conjunction

**AndIntro(a, b)**
Needs `a: A` and `b: B`. Gives `A AND B`.

**AndElimLeft(h)**
Needs `h: A AND B`. Gives `A`.

**AndElimRight(h)**
Needs `h: A AND B`. Gives `B`.

### Disjunction

**OrIntroLeft(a)**
Needs `a: A`. The goal has to be `A OR B` for some `B`. Gives the goal.

**OrIntroRight(b)**
Needs `b: B`. The goal has to be `A OR B` for some `A`. Gives the goal.

**OrElim(o, x, cx, y, cy)**
Proof by cases.

- `o: P OR Q`
- `x` is an assumption with formula `P`
- `cx: C`
- `y` is an assumption with formula `Q`
- `cy: C`

The formulas of `cx` and `cy` have to be equal. Gives `C`. Dependencies: those of `o`, those of `cx` without `x`, those of `cy` without `y`.

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

### Implication

**ModusPonens(i, a)**
Needs `i: A => B` and `a: A`, in either order. Gives `B`.

**ImpliesIntro(x, y)**
`x` is an assumption with formula `A`. `y: B`. Gives `A => B`. Dependencies: those of `y` without `x`. It is not required that `y` depends on `x`.

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

### Negation and falsity

**NotElim(a, n)**
Needs a fact and its negation, in either order. Gives `FALSE`.

**NotIntro(x, c)**
`x` is an assumption with formula `A`. `c: FALSE`. Gives `NOT A`. Dependencies: those of `c` without `x`.

**FalseElim(f)**
Needs `f: FALSE`. Gives the goal, which can be any formula.

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

The rules include no classical principle such as double negation elimination or excluded middle. A proof that needs one can take it as an axiom or a hypothesis.

### Equivalence

**IffIntro(f, b)**
Needs `f: P => Q` and `b: Q => P`. Gives `P <=> Q`.

**IffElim(i, x)**
Needs `i: P <=> Q`. If `x` equals `P`, gives `Q`. Otherwise, if `x` equals `Q`, gives `P`.

### Universal quantifier

**ForallIntro(h, X)**
Needs `h: F`. `X` is a variable. Gives `forall X: F`. Fails if `X` is fixed for `h`, see [Fixed names](#fixed-names).

**ForallElim(u, t)**
Needs `u: forall X: F` and a term `t`. Gives `F` with `t` in place of `X`, see [Substitution](#substitution).

### Existential quantifier

**ExistsIntro(h, t)**
Needs `h: F[X := t]` and a term `t`. The goal has to be `exists X: F`. The checker substitutes `t` for `X` in `F` and compares with the formula of `h`. Gives the goal.

**ExistsElim(e, x, c, w)**
Existential elimination.

- `e: exists X: F`
- `x` is an assumption with formula `F[X := w]`
- `c: C`
- `w` is a variable, the witness

Gives `C`. Dependencies: those of `e`, and those of `c` without `x`. The witness has to be fresh, see [Fixed names](#fixed-names).

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

### Equality

**EqRefl()**
No arguments. The goal has to be `t = t` with identical sides. Gives the goal, with no dependencies.

**EqSym(h)**
Needs `h: s = t`. Gives `t = s`.

**EqTrans(h1, h2)**
Needs `h1: s = t` and `h2: t = u`. The middle terms have to be identical. Gives `s = u`.

**EqSubst(e, h)**
Needs `e: s = t` and a fact `h`. The goal has to be the formula of `h` with some free occurrences of `s` replaced by `t`. Gives the goal. A replacement under a quantifier is rejected when the quantified variable is free in `s` or `t`. To replace in the other direction, apply `EqSym` first.

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

## Fixed names

A name that occurs free in a hypothesis or in an axiom refers to one definite object. Quantifying over it would prove statements that do not hold, so `ForallIntro` and `ExistsElim` check their variable.

A name is fixed for a fact if it is free in the formula of some assumption in the dependencies of the fact, or free in any axiom. All axioms count, not only the ones the proof uses.

- `ForallIntro(h, X)` requires that `X` is not fixed for `h`.
- `ExistsElim(e, x, c, w)` requires that `w` is not free in the formula of `e`, not free in the formula of `c`, and not fixed for the dependencies of the result.

## Substitution

`ForallElim`, `ExistsIntro` and `ExistsElim` replace a variable by a term. Only free occurrences are replaced. Occurrences bound by a quantifier inside the formula stay.

If the term contains a name that a quantifier in the formula binds, that quantifier's variable is renamed first. The new name is the old one with the suffix `_1`, or `_2` and so on, using the first suffix that clashes with no name in the formula or in the term.

Replacing `X` by `Y` in `exists Y: R(X, Y)` gives `exists Y_1: R(Y, Y_1)`. This equals `exists Z: R(Y, Z)` and differs from `exists Y: R(Y, Y)`.

## Applying theorems

A theorem declared earlier acts as a rule. For `name: H1, ..., Hn |- C`, write `name(a1, ..., an)` with one fact per hypothesis. A theorem without hypotheses is written `name()` or just `name`.

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

The statement of the theorem is a pattern in which propositional letters, meaning identifiers without arguments, are placeholders. The call is accepted when one assignment of formulas to the placeholders exists such that:

- each argument `ai` equals `Hi` after assignment,
- the declared formula equals `C` after assignment.

A placeholder gets the same formula at all its occurrences. For `and_comm: A AND B |- B AND A` and `h: P AND Q`, the assignment is `A := P`, `B := Q`, and the declared formula has to be `Q AND P`.

Everything else in the pattern has to match exactly: connectives, predicates, terms, equalities and free names. Bound variables match up to renaming. A placeholder inside a quantifier cannot receive a formula that contains the quantified variable free. Without that restriction `P |- forall X: P` could be applied with `P := Q(X)`.

The result depends on the union of the dependencies of the arguments. Axioms are not patterns: their formulas are used as written.
