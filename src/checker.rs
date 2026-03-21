// SPDX-License-Identifier: GPL-3.0-only

//! Proof checker: walks proof steps and verifies each against inference rules.

use std::collections::HashMap;

use crate::error::{CheckError, RuleError};
use crate::rules::{apply_rule, Rule};
use crate::term::{Formula, Justification, ProofStep, Theorem};

/// Check that `theorem`'s proof is valid given the provided `axioms`.
///
/// Each proof step is verified in order. The environment (set of named formulas
/// in scope) grows as `assume` and `have` steps are processed.
///
/// # Errors
///
/// Returns a [`CheckError`] identifying the step index, the offending step, and
/// a human-readable explanation of the failure.
pub fn check(
    theorem: &Theorem,
    axioms: &HashMap<String, Formula>,
) -> Result<(), CheckError> {
    // Start with the theorem's declared hypotheses pre-loaded into scope.
    let mut env: HashMap<String, Formula> = HashMap::new();
    for (i, hyp) in theorem.hypotheses.iter().enumerate() {
        env.insert(format!("_hyp{}", i), hyp.clone());
    }

    let mut last_formula: Option<Formula> = None;

    for (idx, step) in theorem.steps.iter().enumerate() {
        let make_err = |msg: String| CheckError {
            step_index: idx,
            step: Box::new(step.clone()),
            message: msg,
        };

        match step {
            ProofStep::Assume { name, formula } => {
                env.insert(name.clone(), formula.clone());
                last_formula = Some(formula.clone());
            }

            ProofStep::Have { name, formula, justification } => {
                let derived = resolve_justification(justification, &env, axioms)
                    .map_err(|e| make_err(e.to_string()))?;

                if !formulas_match(&derived, formula) {
                    return Err(make_err(format!(
                        "derived formula `{}` does not match declared formula `{}`",
                        derived, formula
                    )));
                }

                env.insert(name.clone(), formula.clone());
                last_formula = Some(formula.clone());
            }

            ProofStep::Exact { justification } => {
                let derived = match justification {
                    Justification::Axiom(name) => {
                        // First check if it's a name in the environment
                        if let Some(f) = env.get(name) {
                            f.clone()
                        } else if let Some(f) = axioms.get(name) {
                            f.clone()
                        } else {
                            return Err(make_err(format!(
                                "unknown name `{}`: not in scope and not a known axiom",
                                name
                            )));
                        }
                    }
                    Justification::Rule(rule_name, args) => {
                        resolve_rule(rule_name, args, &env)
                            .map_err(|e| make_err(e.to_string()))?
                    }
                };

                if !formulas_match(&derived, &theorem.conclusion) {
                    return Err(make_err(format!(
                        "`exact` produced `{}` but the theorem's conclusion is `{}`",
                        derived, theorem.conclusion
                    )));
                }

                last_formula = Some(derived);
            }
        }
    }

    // Final check: the last formula must equal the conclusion.
    match &last_formula {
        Some(f) if formulas_match(f, &theorem.conclusion) => Ok(()),
        Some(f) => Err(CheckError {
            step_index: theorem.steps.len(),
            step: Box::new(theorem.steps.last().cloned().unwrap_or(ProofStep::Exact {
                justification: Justification::Axiom("_".to_string()),
            })),
            message: format!(
                "proof ends with `{}` but conclusion is `{}`",
                f, theorem.conclusion
            ),
        }),
        None => Err(CheckError {
            step_index: 0,
            step: Box::new(ProofStep::Exact {
                justification: Justification::Axiom("_".to_string()),
            }),
            message: "proof has no steps".to_string(),
        }),
    }
}

/// Resolve a [`Justification`] to a [`Formula`].
fn resolve_justification(
    justification: &Justification,
    env: &HashMap<String, Formula>,
    axioms: &HashMap<String, Formula>,
) -> Result<Formula, RuleError> {
    match justification {
        Justification::Axiom(name) => {
            env.get(name)
                .or_else(|| axioms.get(name))
                .cloned()
                .ok_or_else(|| RuleError {
                    rule: name.clone(),
                    reason: format!("name `{}` is not in scope and is not a known axiom", name),
                })
        }
        Justification::Rule(rule_name, args) => resolve_rule(rule_name, args, env),
    }
}

/// Look up rule by name and apply it to the listed hypothesis names.
fn resolve_rule(
    rule_name: &str,
    args: &[String],
    env: &HashMap<String, Formula>,
) -> Result<Formula, RuleError> {
    let rule = Rule::from_name(rule_name).ok_or_else(|| RuleError {
        rule: rule_name.to_string(),
        reason: "unknown rule name".to_string(),
    })?;

    let mut premises = Vec::with_capacity(args.len());
    for arg in args {
        let f = env.get(arg).ok_or_else(|| RuleError {
            rule: rule_name.to_string(),
            reason: format!("hypothesis `{}` is not in scope", arg),
        })?;
        premises.push(f.clone());
    }

    apply_rule(&rule, &premises)
}

/// Check whether two formulas are structurally equal.
///
/// This is syntactic equality: no alpha-equivalence or unification.
fn formulas_match(a: &Formula, b: &Formula) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::{Formula, Justification, ProofStep, Theorem};

    fn var(s: &str) -> Formula { Formula::Var(s.to_string()) }

    #[test]
    fn valid_assume_exact_proof() {
        // theorem id: A |- A
        // proof assume h: A; exact h; qed
        let thm = Theorem {
            name: "id".to_string(),
            hypotheses: vec![var("A")],
            conclusion: var("A"),
            steps: vec![
                ProofStep::Assume { name: "h".to_string(), formula: var("A") },
                ProofStep::Exact { justification: Justification::Axiom("h".to_string()) },
            ],
        };
        assert!(check(&thm, &HashMap::new()).is_ok());
    }

    #[test]
    fn invalid_exact_wrong_conclusion() {
        let thm = Theorem {
            name: "bad".to_string(),
            hypotheses: vec![var("A")],
            conclusion: var("B"),
            steps: vec![
                ProofStep::Assume { name: "h".to_string(), formula: var("A") },
                ProofStep::Exact { justification: Justification::Axiom("h".to_string()) },
            ],
        };
        assert!(check(&thm, &HashMap::new()).is_err());
    }

    #[test]
    fn and_intro_proof() {
        // theorem: A, B |- A AND B
        let thm = Theorem {
            name: "and_i".to_string(),
            hypotheses: vec![var("A"), var("B")],
            conclusion: Formula::And(Box::new(var("A")), Box::new(var("B"))),
            steps: vec![
                ProofStep::Assume { name: "ha".to_string(), formula: var("A") },
                ProofStep::Assume { name: "hb".to_string(), formula: var("B") },
                ProofStep::Have {
                    name: "c".to_string(),
                    formula: Formula::And(Box::new(var("A")), Box::new(var("B"))),
                    justification: Justification::Rule(
                        "AndIntro".to_string(),
                        vec!["ha".to_string(), "hb".to_string()],
                    ),
                },
                ProofStep::Exact { justification: Justification::Axiom("c".to_string()) },
            ],
        };
        assert!(check(&thm, &HashMap::new()).is_ok());
    }

    #[test]
    fn unknown_hypothesis_returns_error() {
        let thm = Theorem {
            name: "bad".to_string(),
            hypotheses: vec![],
            conclusion: var("A"),
            steps: vec![
                ProofStep::Exact { justification: Justification::Axiom("nonexistent".to_string()) },
            ],
        };
        assert!(check(&thm, &HashMap::new()).is_err());
    }
}
