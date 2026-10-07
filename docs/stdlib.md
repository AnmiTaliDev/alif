# Standard library

The standard library is a set of theorems in `stdlib/logic.alif`. They are available in every file without an `import`.

## Loading

The file is embedded into the library at build time with `include_str!`. Each call of `verify_source` or `verify_file` parses and checks it before the user input is processed. A failure in the library is reported with the file name `<stdlib>`. The library declares no axioms, so everything in it is proved by the same checker that checks user files.

## Theorems

Propositional letters in these statements are placeholders. Any formula can take their place, as described in [inference-rules.md](inference-rules.md#applying-theorems).

| Name | Statement | Use |
|------|-----------|-----|
| `identity` | `\|- A => A` | `identity` or `identity()` |
| `and_comm` | `A AND B \|- B AND A` | `and_comm(h)` |
| `and_assoc` | `(A AND B) AND C \|- A AND (B AND C)` | `and_assoc(h)` |
| `or_comm` | `A OR B \|- B OR A` | `or_comm(h)` |
| `ex_falso` | `FALSE \|- A` | `ex_falso(f)` |
| `not_not_intro` | `A \|- NOT NOT A` | `not_not_intro(a)` |
| `modus_tollens` | `A => B, NOT B \|- NOT A` | `modus_tollens(i, nb)` |
| `contraposition` | `A => B \|- NOT B => NOT A` | `contraposition(i)` |
| `iff_comm` | `A <=> B \|- B <=> A` | `iff_comm(h)` |

Example:

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

## Redefining names

A user file can declare an axiom or a theorem with the name of a library entry. The declaration replaces the library entry for the rest of the run, and a second declaration of the same name is an error. The replacement affects the whole run, including imported files that are processed afterwards.

```
axiom identity: P(a)

theorem uses_shadow:
  |- P(a)
proof
  exact identity
qed
```

## Source

The library source is ordinary Alif code and serves as a set of proofs to read. Each proof uses only the built-in rules.
