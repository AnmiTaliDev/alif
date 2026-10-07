# Errors

Verification stops at the first error. There are three kinds of error. They map to the three variants of `AlifError` and to the exit status of the CLI.

| Kind | Raised when | Exit status |
|------|-------------|-------------|
| Parse error | the text is not valid Alif syntax | 2 |
| Load error | a declaration or an import is not acceptable | 2 |
| Proof error | a proof step does not hold | 1 |

## Message format

Every message can start with a location. The location is `file:line:column`, or `line:column` when the source has no file name, as with standard input or a source string. Lines and columns start at 1. A column counts characters, not bytes.

| Kind | Format |
|------|--------|
| Parse error | `[location: ]parse error: <message>` |
| Load error | `[location: ]error: <message>` |
| Proof error | `[location: ]proof error in theorem `<name>`, step <n>: <message>` |

The step number `<n>` starts at 1. The location points to:

| Kind | Position |
|------|----------|
| Parse error | the offending token, or the end of the input |
| Load error | the keyword that starts the declaration or the import |
| Proof error | the keyword that starts the step. For a proof without steps, the `theorem` keyword. |

The file name is the path as it was given on the command line or to `verify_file`. For an imported file it is the directory of the importing file joined with the path written in the `import`. Errors inside the standard library use `<stdlib>`.

Formulas in messages are printed with parentheses around binary connectives: `(A AND B)`.

## Parse errors

Messages of the form `expected X, found Y` name the expected construct. `Y` is the offending token in backticks or `end of input`.

| Message | Cause |
|---------|-------|
| ``unrecognised token "@"`` | a character that no token accepts |
| ``expected `axiom`, `theorem` or `import`, found Y`` | something else at the top level, for example a stray `assume` |
| ``expected a quoted path, found Y`` | `import` without a string |
| ``expected `:`, found Y`` | a missing `:` after a name or after a quantified variable |
| ``expected `:=`, found Y`` | a `have` step without a justification |
| ``expected `proof`, found Y`` | a theorem header followed by something else |
| ``expected `qed`, found Y`` | a proof that is not closed, or a token that is not a step |
| ``expected `\|-`, found Y`` | several hypotheses without `\|-` |
| ``expected identifier, found Y`` | a name is required, for example an empty argument list `P()` |
| ``expected `,` or `)`, found Y`` | an argument list that is not closed |
| ``expected `)`, found Y`` | a parenthesised formula that is not closed |
| ``expected a formula, found Y`` | an operator without its operand, or an empty formula |

```
$ alif verify bad.alif
bad.alif:5:1: parse error: expected `qed`, found end of input
```

## Load errors

| Message | Cause |
|---------|-------|
| ``` `n` is already defined ``` | a second axiom or theorem with the name `n` |
| ``` `n` is the name of a built-in rule ``` | an axiom or a theorem named like one of the 21 rules |
| ``` cannot read `path`: <os error> ``` | an unreadable file or a missing import |
| ``` import cycle through `path` ``` | a file that imports itself, directly or through other files |
| `` `import` needs a file location; verify a file instead of a source string `` | an `import` in text that has no directory |

The location of a failed import points to the `import` item. The names of the standard library can be declared once without this error, see [stdlib.md](stdlib.md#redefining-names).

## Proof errors

### Structure of the proof

| Message | Cause |
|---------|-------|
| `proof has no steps` | nothing between `proof` and `qed` |
| ``proof must end with `exact` `` | the last step is `assume` or `have` |
| `` `exact` must be the last step of the proof `` | a step after `exact` |
| ``name `n` is already defined in this proof`` | two steps define the same name |

### Comparing formulas

| Message | Cause |
|---------|-------|
| ``derived formula `D` does not match declared formula `F` `` | the justification of `have` derives `D`, the step declares `F` |
| ``` `exact` produced `D` but the theorem's conclusion is `C` ``` | the final justification derives something else than the conclusion |
| ``the result depends on assumption `h: F`, which is neither discharged nor a hypothesis of the theorem`` | an `assume` that is not one of the hypotheses and was not discharged |

### Names

| Message | Cause |
|---------|-------|
| ``` `n` is not in scope and is not an axiom, a rule or a theorem ``` | a bare name that resolves to nothing |
| ``` `n` is not a rule or a theorem ``` | a name with arguments that is neither a rule nor a theorem |
| ``` `n` is not in scope and is not an axiom ``` | an argument of a rule or a theorem that names nothing. For a rule the message is prefixed with ``rule `R` failed:``. |

### Applying a theorem

| Message |
|---------|
| ``theorem `t` expects N arguments, got M`` |
| ``arguments of theorem `t` must be names of facts in scope`` |
| ``the arguments and the declared formula are not an instance of theorem `t`: <statement>`` |

### Rules

Rule failures have the form ``rule `R` failed: <reason>``. The reasons are:

| Rule | Reason |
|------|--------|
| any | ``expects N arguments, got M`` |
| any | ``argument i must be a plain name`` |
| any | ``` `x` is not an assumption ``` for an argument that has to be an assumption |
| `AndElimLeft`, `AndElimRight` | `argument must be a conjunction (A AND B)` |
| `OrIntroLeft` | ``the declared formula must be a disjunction whose left side is `F` `` |
| `OrIntroRight` | ``the declared formula must be a disjunction whose right side is `F` `` |
| `OrElim` | `first argument must be a disjunction (A OR B)` |
| `OrElim` | ``assumption `x` must be `P` `` |
| `OrElim` | ``both branches must derive the same formula, got `F` and `G` `` |
| `ModusPonens` | `arguments must be an implication (A => B) and a formula equal to A` |
| `NotIntro` | ``second argument must be FALSE, got `F` `` |
| `NotElim` | `arguments must be a formula and its negation` |
| `FalseElim` | ``argument must be FALSE, got `F` `` |
| `IffIntro` | `arguments must be A => B and B => A` |
| `IffElim` | `first argument must be an equivalence (A <=> B)` |
| `IffElim` | `second argument must equal one side of the equivalence` |
| `ForallIntro` | ``variable `x` occurs free in an undischarged assumption or an axiom`` |
| `ForallElim` | `first argument must be a universal formula (forall X: F)` |
| `ExistsIntro` | `the declared formula must be existential (exists X: F)` |
| `ExistsIntro` | ``expected the first argument to be `E`, got `F` `` |
| `ExistsElim` | `first argument must be an existential formula (exists X: F)` |
| `ExistsElim` | ``assumption `x` must be `E` `` |
| `ExistsElim` | ``witness `w` is not fresh: it occurs in the existential formula, the conclusion, an undischarged assumption or an axiom`` |
| `EqRefl` | `the declared formula must have the form t = t` |
| `EqSym` | `argument must be an equality (s = t)` |
| `EqTrans` | `arguments must be s = t and t = u` |
| `EqSubst` | `first argument must be an equality (s = t)` |
| `EqSubst` | ``the declared formula must be `F` with some occurrences of `s` replaced by `t` `` |

## Examples

An assumption that is not a hypothesis:

```
bad.alif:4:3: proof error in theorem `t`, step 2: the result depends on assumption `h: B`, which is neither discharged nor a hypothesis of the theorem
```

Generalising over a name that a hypothesis fixes:

```
bad.alif:4:3: proof error in theorem `t`, step 2: rule `ForallIntro` failed: variable `x` occurs free in an undischarged assumption or an axiom
```

A declared formula that differs from the derived one:

```
bad.alif:5:3: proof error in theorem `t`, step 3: derived formula `(A AND B)` does not match declared formula `A`
```

More cases with their sources are in [examples.md](examples.md#common-mistakes).
