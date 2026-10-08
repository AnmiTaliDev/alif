# Examples

The files in `examples/` are run by the test suite. Each can be checked with `alif verify`.

## A first proof

`examples/and_comm.alif`:

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

The sequent has one hypothesis, `A AND B`, and the conclusion `B AND A`. The checker proceeds as follows.

| Step | Fact | Depends on |
|------|------|------------|
| `assume h` | `A AND B` | `h` |
| `have b` | `B` | `h` |
| `have a` | `A` | `h` |
| `have result` | `B AND A` | `h` |
| `exact result` | accepted: the formula is the conclusion, and `h` is a hypothesis | |

```
$ alif verify examples/and_comm.alif
✓ QED
```

## Predicates and quantifiers

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

`ForallElim(all, socrates)` puts the term `socrates` in place of `X` inside `human(X) => mortal(X)`.

`examples/quantifiers.alif` has two more proofs. The first generalises a name:

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

`x` occurs in no hypothesis, so `ForallIntro(p, x)` is allowed. It produces `forall x: P(x)`, which equals the declared `forall X: P(X)`.

The second uses an existential hypothesis:

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

`w` is the existential body for the fresh name `c`. `ExistsElim` discharges `w`, so `result` depends on `ex` only.

## Axioms

An axiom can be used directly as an argument or as a justification.

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

There are no hypotheses. `ImpliesIntro(b, a)` discharges `b`, leaving `inner` dependent on `a`. `ImpliesIntro(a, inner)` discharges `a`. `outer` depends on nothing, and `exact` accepts it.

## Cases

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

`x` and `y` are the assumptions of the two cases. They are not hypotheses, and `OrElim` discharges both. The first case reaches `B` through a contradiction. In the second case `y` is `B` already and serves as the derived fact.

## Equality

`examples/equality.alif` proves reflexivity, symmetry, transitivity and substitution. The last one:

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

## Using theorems

`examples/lemmas.alif` applies `and_comm` from the standard library twice:

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

The first call assigns `A := P` and `B := Q`. The second assigns `A := Q` and `B := P`.

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

Each input below is a complete file named `bad.alif`.

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

The only hypothesis is `A`.

### An assumption that stays open

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

`ImpliesIntro` discharged `a` only.

### Generalising a fixed name

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

The hypothesis speaks about one particular `x`.

### A theorem used on the wrong formula

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

For `h: X AND Y` the theorem gives `Y AND X`.

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

Replacing `X` by `Y` renames the bound `Y`, so the result differs from the declared formula. The statement does not follow from the hypothesis.

### A derivation that gives another formula

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

### No `exact`

```
theorem t: A |- A
proof
  assume h: A
qed
```

```
bad.alif:3:3: proof error in theorem `t`, step 1: proof must end with `exact`
```

### No `qed`

```
theorem t: A |- A
proof
  assume h: A
  exact h
```

```
bad.alif:5:1: parse error: expected `qed`, found end of input
```
