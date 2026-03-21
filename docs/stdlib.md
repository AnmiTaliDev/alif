# Standard Library Reference

This document describes the Alif standard library: what it contains, where it
lives, how it is loaded, and how to use its axioms in proofs.

---

## Table of Contents

1. [Introduction](#introduction)
2. [How the Standard Library Is Loaded](#how-the-standard-library-is-loaded)
3. [Using Standard Library Axioms in Proofs](#using-standard-library-axioms-in-proofs)
4. [Schematic Variables](#schematic-variables)
5. [Axiom Reference](#axiom-reference)
   - [identity](#identity)
   - [and\_comm](#and_comm)
   - [or\_comm](#or_comm)
   - [ex\_falso](#ex_falso)
6. [Interaction with User-Defined Axioms](#interaction-with-user-defined-axioms)
7. [Extending the Standard Library](#extending-the-standard-library)
8. [Source Listing](#source-listing)

---

## Introduction

The Alif **standard library** (`stdlib`) is a small, fixed set of logical
axioms that are available in every Alif proof file without any import statement.
These axioms represent foundational logical schemas — patterns that hold for
any formula substituted for their schematic variables.

The standard library is intentionally minimal. Its purpose is not to serve as
a comprehensive logic framework but to provide a few universally-useful starting
points so that trivial proofs do not require the user to redeclare basic facts.
Complex results must still be proved step by step using the twelve inference
rules.

The standard library currently contains **four axioms**:

| Name | Formula | Informal reading |
|---|---|---|
| `identity` | `A` | Any atom is itself. |
| `and_comm` | `A AND B` | A conjunction is available as a schema. |
| `or_comm` | `A OR B` | A disjunction is available as a schema. |
| `ex_falso` | `NOT A` | A negation is available as a schema. |

---

## How the Standard Library Is Loaded

The standard library lives at:

```
stdlib/logic.alif
```

relative to the project root. It is **embedded into the compiled binary** at
build time using Rust's `include_str!` macro in `src/stdlib.rs`:

```rust
const STDLIB_SOURCE: &str = include_str!("../stdlib/logic.alif");
```

When `verify_source` is called (the main public entry point for verification),
it:

1. Parses the user's source string.
2. Calls `stdlib::load_stdlib()`, which parses the embedded `STDLIB_SOURCE`
   string using the same `parse_source` function used for user files.
3. Collects all `axiom` declarations from the stdlib parse result into a
   `HashMap<String, Formula>`.
4. Passes this axiom map to the checker for every theorem in the user's file.

Because the stdlib is parsed at `verify_source` call time (not at compile time),
any parse error in the embedded `stdlib/logic.alif` would surface as a
`ParseError` at runtime — but since the file is under version control and
compiled into the binary, this represents a build-time error that would be
caught during testing long before release.

The loading sequence means that **stdlib axioms are always in scope**, regardless
of position in the user file. They cannot be shadowed or overridden by user
declarations.

---

## Using Standard Library Axioms in Proofs

Standard library axioms are referenced by name — exactly like user-defined
axioms. There is no import statement, no namespace prefix, and no special
syntax.

To use a stdlib axiom in a proof, simply provide its name as a justification:

```
exact identity
```

or bind it with `have`:

```
have h: A := identity
```

The checker looks up the name first in the local environment (formulas
introduced by `assume` and `have` in the current proof), then in the axiom map
(which contains both stdlib and user-defined axioms). If the name is found in
the axiom map, the formula stored there is returned as the derived formula.

**The declared type annotation must match the axiom's formula exactly** (by
structural equality). Because stdlib axioms are schematic (see next section),
their formula is a simple `Var` atom or a two-atom connective. Whether a
particular usage is semantically appropriate is the author's responsibility.

---

## Schematic Variables

The four stdlib axioms use **schematic variables**: the identifiers `A` and `B`
in the axiom source text are parsed as `Formula::Var("A")` and
`Formula::Var("B")` — ordinary atoms, not universally quantified variables.

This means the axioms do not automatically instantiate. `identity` does not
mean "for all formulas F, F holds"; it means literally the atom `A`. If you
write:

```
exact identity
```

inside a theorem whose conclusion is `A`, the checker accepts it (because the
axiom's stored formula is the atom `A` and the conclusion is also the atom `A`).
But if the conclusion is `P`, it will fail because `P ≠ A`.

Consequently, these axioms are most useful when your theorem's conclusion
literally mentions the atoms `A` or `B`, or when you bind the axiom to a
`have` step with an explicit type annotation that matches the axiom's formula.

---

## Axiom Reference

### `identity`

**Source text:**

```
axiom identity: A
```

**Stored formula:** `Formula::Var("A")`

**Formal meaning:**

The atom `A` is asserted. This represents the schema that any proposition is
itself — but in Alif's non-schematic model, `identity` is simply the atom `A`.

**Usage example:**

```
-- Prove that A holds by citing the identity axiom directly.
theorem use_identity:
  A |- A
proof
  assume h: A
  exact h
qed
```

Because `identity` stores only the atom `A`, it is most directly useful when
the conclusion is exactly the atom `A`:

```
theorem identity_demo:
  A
proof
  exact identity
qed
```

Note: this theorem has no hypotheses and a conclusion of `A`. The proof
consists of a single step citing the stdlib axiom `identity`, whose formula
is `A`.

---

### `and_comm`

**Source text:**

```
axiom and_comm: A AND B
```

**Stored formula:** `Formula::And(Var("A"), Var("B"))`

**Formal meaning:**

The conjunction `A AND B` is asserted as a schema. In practice this axiom is
available as a pre-established fact of the form `A AND B` that you can
reference by name.

**Usage example:**

```
-- Use and_comm as a starting formula in a step.
theorem and_comm_usage:
  A AND B
proof
  exact and_comm
qed
```

For the more common goal of proving commutativity (that `A AND B |- B AND A`),
use `AndElimLeft`, `AndElimRight`, and `AndIntro` directly:

```
theorem and_comm_proof:
  A AND B |- B AND A
proof
  assume h: A AND B
  have b: B := AndElimRight(h)
  have a: A := AndElimLeft(h)
  have result: B AND A := AndIntro(b, a)
  exact result
qed
```

---

### `or_comm`

**Source text:**

```
axiom or_comm: A OR B
```

**Stored formula:** `Formula::Or(Var("A"), Var("B"))`

**Formal meaning:**

The disjunction `A OR B` is asserted as a schema.

**Usage example:**

```
-- Cite or_comm as a pre-established disjunction.
theorem or_comm_usage:
  A OR B
proof
  exact or_comm
qed
```

---

### `ex_falso`

**Source text:**

```
axiom ex_falso: NOT A
```

**Stored formula:** `Formula::Not(Var("A"))`

**Formal meaning:**

The negation `NOT A` is asserted as a schema. The name `ex_falso` alludes to
the classical *ex falso quodlibet* principle ("from falsehood, anything
follows"), but in Alif this axiom simply stores the atom `NOT A`. Its usefulness
is in providing a ready-made negation formula for proofs that require one as a
starting point.

**Usage example:**

```
-- Use ex_falso as a negation premise.
theorem ex_falso_usage:
  NOT A
proof
  exact ex_falso
qed
```

---

## Interaction with User-Defined Axioms

User-defined axioms (declared with `axiom name: formula` in the user's file)
are **additive**: they are inserted into the same axiom map as stdlib axioms.
Both are available simultaneously throughout the file.

The processing order is:

1. Stdlib axioms are loaded first into the map.
2. User axioms are inserted as they are encountered during the top-to-bottom
   pass over the user's file.
3. If a user axiom has the same name as a stdlib axiom, the user axiom
   **overwrites** the stdlib entry for that name (last write wins in the
   `HashMap::insert` semantics).

This means a user can deliberately shadow a stdlib axiom by declaring an axiom
with the same name. This is generally inadvisable and may produce confusing
errors, but it is not prohibited.

Theorems can only reference axioms that have been declared **before** them in
the file. A theorem declared on line 10 cannot reference a user axiom declared
on line 20, even though both are in the same file.

---

## Extending the Standard Library

Users **cannot currently extend the standard library at runtime**. The only
way to add axioms to the stdlib is to edit `stdlib/logic.alif` and recompile
the project.

As an alternative, place user-specific axioms at the top of your `.alif` file:

```
axiom my_law: P OR NOT P    -- law of excluded middle
axiom mp_schema: A => B     -- modus ponens schema
```

These will be available to all theorems that follow in the same file.

---

## Source Listing

The complete source of `stdlib/logic.alif` as embedded in the binary:

```
axiom identity: A
axiom and_comm: A AND B
axiom or_comm: A OR B
axiom ex_falso: NOT A
```

**File:** `stdlib/logic.alif`
**Lines:** 4
**Axioms:** 4

Each line declares exactly one axiom. The file uses no comments, no theorems,
and no proof blocks — only bare `axiom` declarations. The file must remain
parseable by `parse_source`; any syntax error would cause all `verify_source`
calls to return a `ParseError` regardless of the user's input.
