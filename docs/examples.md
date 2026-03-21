# Alif — Worked Examples

This document walks through nine complete examples, from the simplest possible
one-step proof to multi-theorem files and deliberate errors.  For each example
the complete `.alif` source is shown, followed by the CLI invocation and expected
output, and then a detailed explanation of what the checker does at every step.

---

## Introduction

Every Alif proof file is verified by running:

```sh
cargo run --example alif -- verify <file.alif>
```

or, after `cargo build --release`, by:

```sh
./target/release/examples/alif verify <file.alif>
```

A successful run prints `✓ QED` to stdout and exits with code 0.  A proof error
prints a message to stderr and exits with code 1.  A parse error prints to
stderr and exits with code 2.

The standard library axioms (`identity`, `and_comm`, `or_comm`, `ex_falso`) are
loaded automatically on every run.

---

## Example 1: Identity (`A |- A`)

### Source

```
-- identity.alif
-- The simplest possible proof: A implies itself.
theorem identity:
  A |- A
proof
  assume h: A
  exact h
qed
```

### CLI invocation

```sh
alif verify identity.alif
```

### Expected output

```
✓ QED
```

### Step-by-step walkthrough

**Before the proof begins:**

The checker initialises an empty environment (`{}`).  The theorem's hypotheses
list contains one formula, `A`, but it is not placed into the environment
automatically — the proof author must introduce it explicitly.

The conclusion to be proved is `A`.

---

**Step 0 — `assume h: A`**

The checker processes `ProofStep::Assume { name: "h", formula: Var("A") }`.

No justification is required for `assume`.  The formula `A` is inserted into the
environment unconditionally:

```
env: { "h" → Var("A") }
last_formula: Some(Var("A"))
```

---

**Step 1 — `exact h`**

The checker processes `ProofStep::Exact { justification: Axiom("h") }`.

The justification is `Axiom("h")`.  The checker resolves `"h"`:
1. Looks up `"h"` in the environment → found: `Var("A")`.

The derived formula is `Var("A")`.

The checker tests: `Var("A") == Var("A")` (the theorem's conclusion).  They are
equal — the step passes.

```
last_formula: Some(Var("A"))
```

---

**Final check:**

`last_formula` is `Some(Var("A"))` which equals the conclusion `Var("A")`.
The checker returns `Ok(())`.

---

## Example 2: AND commutativity (`A AND B |- B AND A`)

### Source

```
-- and_comm.alif
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

### CLI invocation

```sh
alif verify and_comm.alif
```

### Expected output

```
✓ QED
```

### Step-by-step checker state

| Step | Instruction | Action | Environment after step | `last_formula` |
|---|---|---|---|---|
| 0 | `assume h: A AND B` | Insert `h → (A AND B)` | `{ h → (A AND B) }` | `(A AND B)` |
| 1 | `have b: B := AndElimRight(h)` | Resolve `AndElimRight` on `[h]`: premise `(A AND B)`, extract right → `B`; check `B == B` ✓; insert `b → B` | `{ h → (A AND B), b → B }` | `B` |
| 2 | `have a: A := AndElimLeft(h)` | Resolve `AndElimLeft` on `[h]`: premise `(A AND B)`, extract left → `A`; check `A == A` ✓; insert `a → A` | `{ h, b, a → … }` | `A` |
| 3 | `have result: B AND A := AndIntro(b, a)` | Resolve `AndIntro` on `[b, a]`: premises `B`, `A`; construct `(B AND A)`; check `(B AND A) == (B AND A)` ✓; insert `result → (B AND A)` | `{ h, b, a, result → … }` | `(B AND A)` |
| 4 | `exact result` | Resolve `Axiom("result")` → `(B AND A)` from env; check `(B AND A) == (B AND A)` (conclusion) ✓ | unchanged | `(B AND A)` |

Final check: `last_formula = (B AND A)` equals conclusion `(B AND A)` — `Ok(())`.

---

## Example 3: Modus Ponens chain (`A=>B, B=>C, A |- C`)

This example demonstrates chaining two modus ponens applications to implement
transitivity of implication.

### Source

```
-- transitivity.alif
-- Transitivity of implication: A=>B, B=>C, A |- C
theorem transitivity:
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

### CLI invocation

```sh
alif verify transitivity.alif
```

### Expected output

```
✓ QED
```

### Explanation

After three `assume` steps, the environment contains:
```
ab → (A => B)
bc → (B => C)
ha → A
```

**`have b: B := ModusPonens(ab, ha)`**

The rule `ModusPonens` tries both orderings of its two premises.  With `ab =
(A => B)` and `ha = A`:
- `premises[0]` is `(A => B)` — an `Implies`.
- The antecedent of `(A => B)` is `A`.
- `A == A` (the value of `premises[1]`) — match.
- The consequent `B` is returned.

The checker verifies `B == B` and inserts `b → B`.

**`have c: C := ModusPonens(bc, b)`**

With `bc = (B => C)` and `b = B`:
- `premises[0]` is `(B => C)` — an `Implies`.
- The antecedent `B` matches `premises[1]` which is `B`.
- The consequent `C` is returned.

The checker verifies `C == C` and inserts `c → C`.

**`exact c`** — resolves to `C`, which equals the conclusion.

---

## Example 4: Nested AND elimination (`(A AND B) AND C |- C AND A`)

This example shows how to extract components from a deeply nested conjunction.

### Source

```
-- nested_and.alif
-- Extract the innermost and outermost components of a nested conjunction.
theorem nested_and:
  (A AND B) AND C |- C AND A
proof
  assume h: (A AND B) AND C
  have ab: A AND B := AndElimLeft(h)
  have c:  C       := AndElimRight(h)
  have a:  A       := AndElimLeft(ab)
  have r:  C AND A := AndIntro(c, a)
  exact r
qed
```

### CLI invocation

```sh
alif verify nested_and.alif
```

### Expected output

```
✓ QED
```

### Explanation

The hypothesis `h` has type `(A AND B) AND C`.

- `AndElimLeft(h)` — the first component of the outer conjunction is `A AND B`.
- `AndElimRight(h)` — the second component is `C`.
- `AndElimLeft(ab)` — the first component of the inner conjunction `A AND B` is `A`.
- `AndIntro(c, a)` — combine `C` and `A` to form `C AND A`.

The key insight is that `AndElimLeft` and `AndElimRight` work on whatever
conjunction is in scope.  By applying `AndElimLeft` twice — once on `h` to get
`A AND B`, and once on `ab` to get `A` — we can reach any component of an
arbitrarily nested conjunction.

---

## Example 5: OR introduction (`A, B |- A OR B`)

### Source

```
-- or_intro.alif
-- From A and B, derive A OR B.
theorem or_intro:
  A, B |- A OR B
proof
  assume ha: A
  assume hb: B
  have r: A OR B := OrIntroLeft(ha, hb)
  exact r
qed
```

### CLI invocation

```sh
alif verify or_intro.alif
```

### Expected output

```
✓ QED
```

### Explanation

`OrIntroLeft` takes two premises: the formula to inject on the left (`ha = A`)
and the formula to place on the right (`hb = B`).  It constructs `A OR B`.

Note that `OrIntroRight` can also be used here, but with swapped argument order:
`OrIntroRight(hb, ha)` would produce `A OR B` (with `A` derived from `hb`
becoming the right side and `ha` becoming the left side — see the rule table in
the API reference for the exact semantics).

---

## Example 6: Forall elimination (`forall X: X, t |- t`)

This example demonstrates substitution via `ForallElim`.

### Source

```
-- forall_elim.alif
-- Given a universal statement and a witness, derive the instantiated formula.
theorem forall_elim:
  forall X: X |- mortal(socrates)
proof
  assume fa: forall X: X
  assume t:  mortal(socrates)
  have r: mortal(socrates) := ForallElim(fa, t)
  exact r
qed
```

### CLI invocation

```sh
alif verify forall_elim.alif
```

### Expected output

```
✓ QED
```

### Explanation

**`assume fa: forall X: X`**

Introduces the universal formula `forall X: X` under the name `fa`.

**`assume t: mortal(socrates)`**

Introduces the term `mortal(socrates)` as a `Var("mortal(socrates)")` under the
name `t`.  This is the witness that will be substituted for `X`.

**`have r: mortal(socrates) := ForallElim(fa, t)`**

`ForallElim` is applied with two premises:
1. `fa` = `Forall("X", Var("X"))` — a universal formula.
2. `t` = `Var("mortal(socrates)")` — the term to substitute.

The rule extracts the bound variable `"X"` and the body `Var("X")`, then calls
`substitute(body, "X", &Var("mortal(socrates)"))`.

`substitute` sees `Var("X")` with `var = "X"`: since the name matches, it
returns `Var("mortal(socrates)")`.

The derived formula is `Var("mortal(socrates)")`.  The checker verifies this
equals the declared formula `Var("mortal(socrates)")` — they are identical.

---

## Example 7: Multi-theorem file (theorem 2 depends on axioms from theorem 1's scope)

This example shows how a file can contain multiple theorems and how user-declared
axioms become available to later theorems.

### Source

```
-- multi.alif
-- Declare a user axiom and then use it in a theorem.

axiom all_humans_mortal: human(X) => mortal(X)

-- This theorem uses the user axiom declared above.
theorem socrates_mortal:
  human(socrates) |- mortal(socrates)
proof
  assume h: human(socrates)
  have impl: human(X) => mortal(X) := all_humans_mortal
  have r:    mortal(socrates)      := ModusPonens(impl, h)
  exact r
qed

-- A second theorem uses only built-in rules.
theorem and_comm_again:
  A AND B |- B AND A
proof
  assume h: A AND B
  have b: B       := AndElimRight(h)
  have a: A       := AndElimLeft(h)
  have r: B AND A := AndIntro(b, a)
  exact r
qed
```

### CLI invocation

```sh
alif verify multi.alif
```

### Expected output

```
✓ QED
```

### Explanation

`verify_source` processes items in declaration order:

1. `Item::Axiom { name: "all_humans_mortal", formula: Implies(Var("human(X)"), Var("mortal(X)")) }` — inserted into the axiom map.
2. `Item::Theorem(socrates_mortal)` — checked with the axiom map that now
   contains `all_humans_mortal`.
3. `Item::Theorem(and_comm_again)` — checked independently; does not depend on
   the first theorem.

In `socrates_mortal`, the step `have impl: human(X) => mortal(X) := all_humans_mortal`
uses a `Justification::Axiom("all_humans_mortal")`.  The checker looks up
`"all_humans_mortal"` in the environment (not found), then in the axiom map
(found: `(human(X) => mortal(X))`).  The derived formula equals the declared
formula, so the step passes.

The `ModusPonens(impl, h)` step then applies with:
- `impl = (human(X) => mortal(X))`
- `h = human(socrates)`

`ModusPonens` checks whether the antecedent of `impl` equals `h`:
`human(X) == human(socrates)`?  This is a string comparison on `Var` values:
`"human(X)" != "human(socrates)"`.  They are **not** equal.

This means `ModusPonens` would fail in this exact form.  To make it work, the
axiom must be stated with the concrete value, or the proof must use `ForallElim`
to instantiate a universally quantified form.  The corrected multi-theorem file
that actually passes:

```
-- multi_correct.alif

axiom socrates_is_human: human(socrates)
axiom humans_are_mortal: human(socrates) => mortal(socrates)

theorem socrates_mortal:
  human(socrates) |- mortal(socrates)
proof
  assume h: human(socrates)
  have impl: human(socrates) => mortal(socrates) := humans_are_mortal
  have r:    mortal(socrates)                    := ModusPonens(impl, h)
  exact r
qed

theorem and_comm_again:
  A AND B |- B AND A
proof
  assume h: A AND B
  have b: B       := AndElimRight(h)
  have a: A       := AndElimLeft(h)
  have r: B AND A := AndIntro(b, a)
  exact r
qed
```

This illustrates that Alif uses syntactic equality: `human(X)` and
`human(socrates)` are distinct atoms.

---

## Example 8: A deliberately wrong proof

### Source

```
-- wrong.alif
-- Attempt to prove B from A (impossible).
theorem wrong:
  A |- B
proof
  assume h: A
  exact h
qed
```

### CLI invocation

```sh
alif verify wrong.alif
```

### Expected output (stderr, exit code 1)

```
proof error: proof error at step 1: `exact` produced `A` but the theorem's conclusion is `B`
```

### Explanation

The proof has two steps (zero-indexed):

- Step 0: `assume h: A` — succeeds; inserts `h → A`.
- Step 1: `exact h` — resolves `h` to `A`; checks `A == B` (the conclusion).
  `Var("A") != Var("B")` — the check fails.

The checker constructs:

```rust
CheckError {
    step_index: 1,
    step: Box::new(ProofStep::Exact { justification: Justification::Axiom("h") }),
    message: "`exact` produced `A` but the theorem's conclusion is `B`",
}
```

This is wrapped in `AlifError::Check` and the CLI prints:

```
proof error: proof error at step 1: `exact` produced `A` but the theorem's conclusion is `B`
```

The CLI exits with code 1.

---

## Example 9: Parser error

### Source

```
-- bad_syntax.alif
-- Missing `proof` keyword.
theorem oops:
  A |- A
  assume h: A
  exact h
qed
```

### CLI invocation

```sh
alif verify bad_syntax.alif
```

### Expected output (stderr, exit code 2)

```
parse error: parse error: expected `proof`, got Assume ("assume")
```

### Explanation

After parsing the sequent `A |- A`, the parser's `parse_theorem` method calls:

```rust
self.expect(&Token::Proof, "`proof`")?;
```

The current token is `Assume` (the word `assume`), not `Proof`.  The `expect`
method returns:

```rust
Err(ParseError {
    message: "expected `proof`, got Assume (\"assume\")",
    offset: None,
})
```

This `ParseError` propagates through `parse_source` and `verify_source`, and is
wrapped as `AlifError::Parse(e)`.  The CLI matches this variant and prints:

```
parse error: parse error: expected `proof`, got Assume ("assume")
```

Note: the `Display` of `AlifError::Parse(e)` delegates to `ParseError`'s
`Display`, which — since `offset` is `None` — produces
`"parse error: {message}"`.  The CLI then prepends its own label, giving the
double `"parse error: parse error: …"` appearance.  The CLI exits with code 2.

---

## Summary table

| Example | Sequent | Key rules used | Outcome |
|---|---|---|---|
| 1: Identity | `A |- A` | none (assume + exact) | `✓ QED` |
| 2: AND commutativity | `A AND B |- B AND A` | `AndElimRight`, `AndElimLeft`, `AndIntro` | `✓ QED` |
| 3: Modus Ponens chain | `A=>B, B=>C, A |- C` | `ModusPonens` ×2 | `✓ QED` |
| 4: Nested AND | `(A AND B) AND C |- C AND A` | `AndElimLeft` ×2, `AndElimRight`, `AndIntro` | `✓ QED` |
| 5: OR introduction | `A, B |- A OR B` | `OrIntroLeft` | `✓ QED` |
| 6: Forall elimination | `forall X: X |- mortal(socrates)` | `ForallElim` | `✓ QED` |
| 7: Multi-theorem file | multiple | `ModusPonens`, `AndIntro`, etc. | `✓ QED` |
| 8: Wrong proof | `A |- B` | — | proof error at step 1 |
| 9: Parse error | malformed | — | parse error |
