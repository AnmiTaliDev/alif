# Examples

The files in `examples/` are checked by the test suite. Run any of them with `alif verify`. The outputs below are from the CLI.

## Reading a proof

```
theorem and_comm:
  A AND B |- B AND A
proof
  assume h: A AND B
  have b: B := AndElimRight(h)
  have a: A := AndElimLeft(h)
  have result: B AND A := AndIntro(b, a)
  exact result
qed
```

This is `examples/and_comm.alif`. The sequent has one hypothesis, `A AND B`, and the conclusion `B AND A`.

1. `assume h: A AND B` introduces the hypothesis under the name `h`. Dependencies of `h`: `{h}`.
2. `AndElimRight(h)` derives `B`. The step declares `B`, and the formula matches. Dependencies: `{h}`.
3. `AndElimLeft(h)` derives `A`.
4. `AndIntro(b, a)` derives `B AND A`.
5. `exact result` derives the conclusion. The only dependency is `h`, whose formula is the hypothesis, so the proof is complete.

```
$ alif verify examples/and_comm.alif
✓ QED
```

## Predicates

`examples/syllogism.alif`:

```
theorem syllogism:
  forall X: human(X) => mortal(X), human(socrates) |- mortal(socrates)
proof
  assume all: forall X: human(X) => mortal(X)
  assume h: human(socrates)
  have step: human(socrates) => mortal(socrates) := ForallElim(all, socrates)
  have m: mortal(socrates) := ModusPonens(step, h)
  exact m
qed
```

`ForallElim(all, socrates)` substitutes the term `socrates` for `X` in the body of `all`. Predicates take terms as arguments, so the substitution reaches inside `human(X)` and `mortal(X)`.

`examples/socrates.alif` is the same statement reduced to `mortal(socrates) |- mortal(socrates)`.

## Axioms

An axiom can be passed to a rule or used as a justification. It has no dependencies.

```
axiom human_socrates: human(socrates)
axiom all_mortal: forall X: human(X) => mortal(X)

theorem socrates_mortal:
  |- mortal(socrates)
proof
  have step: human(socrates) => mortal(socrates) := ForallElim(all_mortal, socrates)
  have r: mortal(socrates) := ModusPonens(step, human_socrates)
  exact r
qed
```

## Cases and contradiction

`examples/disjunction.alif`:

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

`x` and `y` are assumptions for the two cases. They are not hypotheses of the theorem. `OrElim` discharges them, so `result` depends on `o` and `na` only. In the first case `NotElim` and `FalseElim` derive `B` from the contradiction. In the second case `y` is already `B`, and `y` is passed as the derived fact.

## Discharging assumptions

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

The theorem has no hypotheses. Both assumptions are discharged by `ImpliesIntro`, so `outer` has no dependencies and `exact` succeeds. `A => B => A` is read as `A => (B => A)`.

## Quantifiers

`examples/quantifiers.alif` contains three proofs.

```
theorem forall_left:
  forall X: P(X) AND Q(X) |- forall X: P(X)
proof
  assume all: forall X: P(X) AND Q(X)
  have both: P(x) AND Q(x) := ForallElim(all, x)
  have p: P(x) := AndElimLeft(both)
  have result: forall X: P(X) := ForallIntro(p, x)
  exact result
qed
```

`x` is a name that appears in no hypothesis, so `ForallIntro(p, x)` is allowed. The result `forall x: P(x)` equals the declared `forall X: P(X)` because bound names do not matter.

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

`w` is the instance of the existential formula for the fresh name `c`. `ExistsElim` discharges `w`, and the result depends on `ex` only.

## Equality

`examples/equality.alif` proves reflexivity, symmetry, transitivity and substitution:

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

## Theorems as lemmas

`examples/lemmas.alif`:

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

`and_comm` comes from the standard library. The first call replaces `A` by `P` and `B` by `Q`. The second call replaces `A` by `Q` and `B` by `P`.

## Imports

`examples/lib/common.alif`:

```
theorem weaken:
  A, B |- A
proof
  assume a: A
  assume b: B
  exact a
qed
```

`examples/import_main.alif`:

```
import "lib/common.alif"

theorem first_of_two:
  X AND Y |- X
proof
  assume h: X AND Y
  have x: X := AndElimLeft(h)
  have y: Y := AndElimRight(h)
  have r: X := weaken(x, y)
  exact r
qed
```

The path is relative to the directory of `import_main.alif`.

## Common mistakes

Each case below is a complete file named `bad.alif`.

### An assumption that is not a hypothesis

```
theorem t: A |- B
proof
  assume h: B
  exact h
qed
```

```
bad.alif:4:3: proof error in theorem `t`, step 2: the result depends on assumption `h: B`, which is neither discharged nor a hypothesis of the theorem
```

The hypothesis is `A`. Introducing `B` as an assumption does not prove `B`.

### An assumption that is never discharged

```
theorem t: |- A => B
proof
  assume a: A
  assume b: B
  have r: A => B := ImpliesIntro(a, b)
  exact r
qed
```

```
bad.alif:6:3: proof error in theorem `t`, step 4: the result depends on assumption `b: B`, which is neither discharged nor a hypothesis of the theorem
```

`ImpliesIntro` discharges `a` only. The result still depends on `b`.

### Generalising over a fixed name

```
theorem t: P(x) |- forall x: P(x)
proof
  assume h: P(x)
  have g: forall x: P(x) := ForallIntro(h, x)
  exact g
qed
```

```
bad.alif:4:3: proof error in theorem `t`, step 2: rule `ForallIntro` failed: variable `x` occurs free in an undischarged assumption or an axiom
```

The hypothesis talks about one particular `x`, so the statement cannot be generalised.

### A theorem applied to the wrong formula

```
theorem t: X AND Y |- X AND Y
proof
  assume h: X AND Y
  have r: X AND Y := and_comm(h)
  exact r
qed
```

```
bad.alif:4:3: proof error in theorem `t`, step 2: the arguments and the declared formula are not an instance of theorem `and_comm`: (A AND B) |- (B AND A)
```

With `h: X AND Y` the theorem yields `Y AND X`.

### Variable capture

```
theorem t: forall X: exists Y: R(X, Y) |- exists Y: R(Y, Y)
proof
  assume all: forall X: exists Y: R(X, Y)
  have r: exists Y: R(Y, Y) := ForallElim(all, Y)
  exact r
qed
```

```
bad.alif:4:3: proof error in theorem `t`, step 2: derived formula `exists Y_1: R(Y, Y_1)` does not match declared formula `exists Y: R(Y, Y)`
```

Substituting `Y` into the body renames the bound `Y`. The statement of the theorem is not derivable, and the checker does not derive it.

### A derivation that does not produce the declared formula

```
theorem t: A, B |- A
proof
  assume a: A
  assume b: B
  have c: A := AndIntro(a, b)
  exact c
qed
```

```
bad.alif:5:3: proof error in theorem `t`, step 3: derived formula `(A AND B)` does not match declared formula `A`
```

### A proof without `exact`

```
theorem t: A |- A
proof
  assume h: A
qed
```

```
bad.alif:3:3: proof error in theorem `t`, step 1: proof must end with `exact`
```

### A missing `qed`

```
theorem t: A |- A
proof
  assume h: A
  exact h
```

```
bad.alif:5:1: parse error: expected `qed`, found end of input
```
