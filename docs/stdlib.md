# Standard library

The standard library is the file `stdlib/logic.alif`. Its theorems can be used in every file without an `import`.

## How it is loaded

The file is compiled into the library with `include_str!`. Each verification run parses and checks it first, with the same code that checks user files, and only then reads the user input. The library contains theorems only, and no axioms. If it fails to check, the diagnostic names the file `<stdlib>`.

## Theorems

Propositional letters in the statements are placeholders. Any formula can take their place, see [inference-rules.md](inference-rules.md#applying-theorems).

| Name | Statement | Call |
|------|-----------|------|
| `identity` | `\|- A => A` | `identity` |
| `and_comm` | `A AND B \|- B AND A` | `and_comm(h)` |
| `and_assoc` | `(A AND B) AND C \|- A AND (B AND C)` | `and_assoc(h)` |
| `or_comm` | `A OR B \|- B OR A` | `or_comm(h)` |
| `ex_falso` | `FALSE \|- A` | `ex_falso(f)` |
| `not_not_intro` | `A \|- NOT NOT A` | `not_not_intro(a)` |
| `modus_tollens` | `A => B, NOT B \|- NOT A` | `modus_tollens(i, nb)` |
| `contraposition` | `A => B \|- NOT B => NOT A` | `contraposition(i)` |
| `iff_comm` | `A <=> B \|- B <=> A` | `iff_comm(h)` |

Example use:

```
theorem dni:
  |- A => NOT NOT A
proof
  assume a: A
  have nn: NOT NOT A := not_not_intro(a)
  have r: A => NOT NOT A := ImpliesIntro(a, nn)
  exact r
qed
```

## Redeclaring library names

A user file may declare an axiom or theorem with the name of a library entry. The declaration replaces the library entry for the rest of the run, including files imported afterwards. Declaring the same name a second time is an error.

```
axiom identity: P(a)

theorem uses_new_identity:
  |- P(a)
proof
  exact identity
qed
```

## Reading the source

`stdlib/logic.alif` is written in plain Alif and uses only built-in rules. It can be read as a set of short proofs.
