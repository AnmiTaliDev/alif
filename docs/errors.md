# Diagnostics

Processing of a file stops at the first error. There are three kinds.

| Kind | Meaning | `AlifError` variant | CLI exit status |
|------|---------|---------------------|-----------------|
| Parse error | the text is not valid syntax | `Parse` | 2 |
| Load error | a declaration or import is not acceptable | `Load` | 2 |
| Proof error | a proof step does not hold | `Check` | 1 |

## Format

```
[location: ]parse error: <message>
[location: ]error: <message>
[location: ]proof error in theorem `<name>`, step <n>: <message>
```

The first line is a parse error, the second a load error and the third a proof error. The step number starts at 1.

A location is `file:line:column`, or `line:column` when the text has no file name (standard input, a source string). Lines and columns start at 1, and a column counts characters. The location points to:

| Kind | Position |
|------|----------|
| Parse error | the offending token, or the end of the text |
| Load error | the keyword that starts the declaration or the `import` |
| Proof error | the keyword that starts the step; for a proof without steps, the `theorem` keyword |

The file name is the path as given to the command line or to `verify_file`. In an imported file it is the importing file's directory joined with the path written in `import`. In the standard library it is `<stdlib>`.

Formulas in messages are printed with parentheses around binary connectives, as `(A AND B)`.

## Parse errors

Most messages read ``expected X, found Y``. `Y` is the offending token in backticks, or `end of input`.

| Message | Cause |
|---------|-------|
| ``unrecognised token "@"`` | a character that starts no token |
| ``expected `axiom`, `theorem` or `import`, found Y`` | something else at the top level, for example a step outside a proof |
| ``expected a quoted path, found Y`` | `import` without a string |
| ``expected `:`, found Y`` | no `:` after a name or after a quantified variable |
| ``expected `:=`, found Y`` | a `have` step without a justification |
| ``expected `proof`, found Y`` | the sequent is followed by something else |
| ``expected `qed`, found Y`` | an unclosed proof, or a token that is not a step |
| ``expected `\|-`, found Y`` | several hypotheses without `\|-` |
| ``expected identifier, found Y`` | a name is required, for instance in an empty argument list `P()` |
| ``expected `,` or `)`, found Y`` | an argument list is not closed |
| ``expected `)`, found Y`` | a parenthesised formula is not closed |
| ``expected a formula, found Y`` | an operator without an operand, or no formula where one is required |

```
bad.alif:5:1: parse error: expected `qed`, found end of input
```

## Load errors

| Message | Cause |
|---------|-------|
| ``` `n` is already defined ``` | a second declaration of the name `n` |
| ``` `n` is the name of a built-in rule ``` | an axiom or theorem named like one of the 21 rules |
| ``` cannot read `path`: <system message> ``` | an unreadable file or missing import |
| ``` import cycle through `path` ``` | a file imports itself, directly or through others |
| `` `import` needs a file location; verify a file instead of a source string `` | `import` in text without a directory |

For a failed import the location is the `import` declaration. Library names can be declared once without error, see [stdlib.md](stdlib.md#redeclaring-library-names).

## Proof errors

### Shape of the proof

| Message | Cause |
|---------|-------|
| `proof has no steps` | nothing between `proof` and `qed` |
| ``proof must end with `exact` `` | the last step is `assume` or `have` |
| `` `exact` must be the last step of the proof `` | a step follows `exact` |
| ``name `n` is already defined in this proof`` | two steps define the same name |

### Formulas that do not match

| Message | Cause |
|---------|-------|
| ``derived formula `D` does not match declared formula `F` `` | the justification of a `have` derives `D`, but the step declares `F` |
| ``` `exact` produced `D` but the theorem's conclusion is `C` ``` | the final justification derives something other than the conclusion |
| ``the result depends on assumption `h: F`, which is neither discharged nor a hypothesis of the theorem`` | an assumption that is not a hypothesis was never discharged |

### Names

| Message | Cause |
|---------|-------|
| ``` `n` is not in scope and is not an axiom, a rule or a theorem ``` | a bare name that resolves to nothing |
| ``` `n` is not a rule or a theorem ``` | a name with arguments that is neither |
| ``` `n` is not in scope and is not an axiom ``` | an argument names nothing. With a rule, the message starts with ``rule `R` failed:``. |

### Theorem application

| Message |
|---------|
| ``theorem `t` expects N arguments, got M`` |
| ``arguments of theorem `t` must be names of facts in scope`` |
| ``the arguments and the declared formula are not an instance of theorem `t`: <statement>`` |

### Rule failures

The form is ``rule `R` failed: <reason>``. Reasons that apply to any rule:

| Reason | Cause |
|--------|-------|
| ``expects N arguments, got M`` | wrong argument count |
| ``argument i must be a plain name`` | a function application where a name is required |
| ``` `x` is not an assumption ``` | the argument was made by `have`, not `assume` |

Reasons that belong to one rule:

| Rule | Reason |
|------|--------|
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

## Samples

```
bad.alif:4:3: proof error in theorem `t`, step 2: the result depends on assumption `h: B`, which is neither discharged nor a hypothesis of the theorem
```

```
bad.alif:4:3: proof error in theorem `t`, step 2: rule `ForallIntro` failed: variable `x` occurs free in an undischarged assumption or an axiom
```

```
bad.alif:5:3: proof error in theorem `t`, step 3: derived formula `(A AND B)` does not match declared formula `A`
```

The inputs that produce these messages are in [examples.md](examples.md#common-mistakes).
