// SPDX-License-Identifier: GPL-3.0-only

use std::collections::BTreeSet;

use crate::context::{Context, Sequent};
use crate::error::CheckError;
use crate::formula_ops::{alpha_eq, matches_all};
use crate::rules::{apply_rule, lookup_fact, Env, Fact, Rule};
use crate::term::{Formula, Justification, ProofStep, Term, Theorem};

pub fn check(theorem: &Theorem, ctx: &Context) -> Result<(), CheckError> {
    let Some(last) = theorem.steps.len().checked_sub(1) else {
        return Err(CheckError {
            theorem: theorem.name.clone(),
            step_index: 0,
            offset: theorem.offset,
            message: "proof has no steps".to_string(),
            location: None,
        });
    };

    let mut env = Env::new();
    for (index, step) in theorem.steps.iter().enumerate() {
        if let Err(message) = check_step(theorem, ctx, &mut env, step, index == last) {
            return Err(step_error(theorem, index, message));
        }
    }

    match theorem.steps[last] {
        ProofStep::Exact { .. } => Ok(()),
        _ => Err(step_error(
            theorem,
            last,
            "proof must end with `exact`".to_string(),
        )),
    }
}

fn step_error(theorem: &Theorem, index: usize, message: String) -> CheckError {
    CheckError {
        theorem: theorem.name.clone(),
        step_index: index,
        offset: theorem.steps[index].offset(),
        message,
        location: None,
    }
}

fn check_step(
    theorem: &Theorem,
    ctx: &Context,
    env: &mut Env,
    step: &ProofStep,
    is_last: bool,
) -> Result<(), String> {
    match step {
        ProofStep::Assume { name, formula, .. } => define(
            env,
            name,
            Fact {
                formula: formula.clone(),
                deps: BTreeSet::from([name.clone()]),
                assumption: true,
            },
        ),
        ProofStep::Have {
            name,
            formula,
            justification,
            ..
        } => {
            let derived = derive(justification, env, ctx, formula)?;
            if !alpha_eq(&derived.formula, formula) {
                return Err(format!(
                    "derived formula `{}` does not match declared formula `{}`",
                    derived.formula, formula
                ));
            }
            define(
                env,
                name,
                Fact {
                    formula: formula.clone(),
                    deps: derived.deps,
                    assumption: false,
                },
            )
        }
        ProofStep::Exact { justification, .. } => {
            if !is_last {
                return Err("`exact` must be the last step of the proof".to_string());
            }
            let derived = derive(justification, env, ctx, &theorem.conclusion)?;
            if !alpha_eq(&derived.formula, &theorem.conclusion) {
                return Err(format!(
                    "`exact` produced `{}` but the theorem's conclusion is `{}`",
                    derived.formula, theorem.conclusion
                ));
            }
            for dep in &derived.deps {
                let assumed = &env[dep].formula;
                if !theorem.hypotheses.iter().any(|h| alpha_eq(h, assumed)) {
                    return Err(format!(
                        "the result depends on assumption `{}: {}`, which is neither discharged nor a hypothesis of the theorem",
                        dep, assumed
                    ));
                }
            }
            Ok(())
        }
    }
}

fn define(env: &mut Env, name: &str, fact: Fact) -> Result<(), String> {
    if env.contains_key(name) {
        return Err(format!("name `{}` is already defined in this proof", name));
    }
    env.insert(name.to_string(), fact);
    Ok(())
}

fn derive(
    justification: &Justification,
    env: &Env,
    ctx: &Context,
    goal: &Formula,
) -> Result<Fact, String> {
    match justification {
        Justification::Ref(name) => {
            if let Some(fact) = lookup_fact(env, ctx, name) {
                return Ok(Fact {
                    assumption: false,
                    ..fact
                });
            }
            apply_named(name, &[], env, ctx, goal).unwrap_or_else(|| {
                Err(format!(
                    "`{}` is not in scope and is not an axiom, a rule or a theorem",
                    name
                ))
            })
        }
        Justification::Rule(name, args) => apply_named(name, args, env, ctx, goal)
            .unwrap_or_else(|| Err(format!("`{}` is not a rule or a theorem", name))),
    }
}

fn apply_named(
    name: &str,
    args: &[Term],
    env: &Env,
    ctx: &Context,
    goal: &Formula,
) -> Option<Result<Fact, String>> {
    if let Some(rule) = Rule::from_name(name) {
        return Some(apply_rule(rule, args, env, ctx, goal).map_err(|e| e.to_string()));
    }
    ctx.theorems
        .get(name)
        .map(|sequent| apply_theorem(name, sequent, args, env, ctx, goal))
}

fn apply_theorem(
    name: &str,
    sequent: &Sequent,
    args: &[Term],
    env: &Env,
    ctx: &Context,
    goal: &Formula,
) -> Result<Fact, String> {
    if args.len() != sequent.hypotheses.len() {
        return Err(format!(
            "theorem `{}` expects {} argument{}, got {}",
            name,
            sequent.hypotheses.len(),
            if sequent.hypotheses.len() == 1 { "" } else { "s" },
            args.len()
        ));
    }

    let mut facts = Vec::with_capacity(args.len());
    for arg in args {
        let Term::Name(arg_name) = arg else {
            return Err(format!(
                "arguments of theorem `{}` must be names of facts in scope",
                name
            ));
        };
        match lookup_fact(env, ctx, arg_name) {
            Some(fact) => facts.push(fact),
            None => {
                return Err(format!(
                    "`{}` is not in scope and is not an axiom",
                    arg_name
                ))
            }
        }
    }

    let mut pairs: Vec<(&Formula, &Formula)> = sequent
        .hypotheses
        .iter()
        .zip(facts.iter().map(|f| &f.formula))
        .collect();
    pairs.push((&sequent.conclusion, goal));
    if !matches_all(&pairs) {
        return Err(format!(
            "the arguments and the declared formula are not an instance of theorem `{}`: {}",
            name, sequent
        ));
    }

    let deps = facts.iter().flat_map(|f| f.deps.iter().cloned()).collect();
    Ok(Fact {
        formula: goal.clone(),
        deps,
        assumption: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_source;
    use crate::term::Item;

    fn run(source: &str) -> Result<(), CheckError> {
        let mut ctx = Context::default();
        for item in parse_source(source).unwrap() {
            match item {
                Item::Axiom { name, formula, .. } => {
                    ctx.axioms.insert(name, formula);
                }
                Item::Theorem(theorem) => {
                    check(&theorem, &ctx)?;
                    ctx.theorems.insert(
                        theorem.name.clone(),
                        Sequent {
                            hypotheses: theorem.hypotheses.clone(),
                            conclusion: theorem.conclusion.clone(),
                        },
                    );
                }
                Item::Import { .. } => panic!("imports are not handled here"),
            }
        }
        Ok(())
    }

    fn message(source: &str) -> String {
        run(source).unwrap_err().message
    }

    #[test]
    fn assume_then_exact() {
        run("theorem t: A |- A\nproof\n  assume h: A\n  exact h\nqed").unwrap();
    }

    #[test]
    fn assumption_must_be_a_hypothesis() {
        let msg = message("theorem t: A |- B\nproof\n  assume h: B\n  exact h\nqed");
        assert!(msg.contains("neither discharged nor a hypothesis"), "{}", msg);
    }

    #[test]
    fn hypothesis_matches_up_to_renaming() {
        run("theorem t: forall X: P(X) |- forall Y: P(Y)\nproof\n  assume h: forall Y: P(Y)\n  exact h\nqed")
            .unwrap();
    }

    #[test]
    fn wrong_conclusion_is_rejected() {
        let msg = message("theorem t: A |- B\nproof\n  assume h: A\n  exact h\nqed");
        assert!(msg.contains("conclusion"), "{}", msg);
    }

    #[test]
    fn proof_must_end_with_exact() {
        let msg = message("theorem t: A |- A\nproof\n  assume h: A\nqed");
        assert!(msg.contains("must end with `exact`"), "{}", msg);
    }

    #[test]
    fn exact_must_be_last() {
        let msg = message("theorem t: A |- A\nproof\n  assume h: A\n  exact h\n  exact h\nqed");
        assert!(msg.contains("last step"), "{}", msg);
    }

    #[test]
    fn empty_proof_is_rejected() {
        assert!(message("theorem t: A |- A\nproof\nqed").contains("no steps"));
    }

    #[test]
    fn duplicate_names_are_rejected() {
        let msg = message(
            "theorem t: A |- A\nproof\n  assume h: A\n  have h: A := h\n  exact h\nqed",
        );
        assert!(msg.contains("already defined"), "{}", msg);
    }

    #[test]
    fn unknown_reference_is_rejected() {
        let msg = message("theorem t: A |- A\nproof\n  exact nothing\nqed");
        assert!(msg.contains("`nothing` is not in scope"), "{}", msg);
    }

    #[test]
    fn unknown_rule_is_rejected() {
        let msg = message("theorem t: A |- A\nproof\n  assume h: A\n  exact Nope(h)\nqed");
        assert!(msg.contains("not a rule or a theorem"), "{}", msg);
    }

    #[test]
    fn declared_formula_must_match_derivation() {
        let msg = message(
            "theorem t: A, B |- A\nproof\n  assume a: A\n  assume b: B\n  have c: A := AndIntro(a, b)\n  exact c\nqed",
        );
        assert!(msg.contains("does not match declared formula"), "{}", msg);
    }

    #[test]
    fn declared_formula_matches_up_to_renaming() {
        run("theorem t: forall X: P(X) |- forall Y: P(Y)\nproof\n  assume h: forall X: P(X)\n  have g: forall Y: P(Y) := h\n  exact g\nqed")
            .unwrap();
    }

    #[test]
    fn implication_introduction_discharges_assumption() {
        run("theorem t: A |- B => A\nproof\n  assume a: A\n  assume b: B\n  have r: B => A := ImpliesIntro(b, a)\n  exact r\nqed")
            .unwrap();
    }

    #[test]
    fn undischarged_temporary_assumption_blocks_exact() {
        let msg = message(
            "theorem t: A |- B\nproof\n  assume a: A\n  assume b: B\n  exact b\nqed",
        );
        assert!(msg.contains("assumption `b: B`"), "{}", msg);
    }

    #[test]
    fn implies_intro_needs_an_assumption() {
        let msg = message(
            "theorem t: A, B |- B => A\nproof\n  assume a: A\n  assume b: B\n  have x: B := b\n  have r: B => A := ImpliesIntro(x, a)\n  exact r\nqed",
        );
        assert!(msg.contains("not an assumption"), "{}", msg);
    }

    #[test]
    fn axiom_reference() {
        run("axiom ax: P(a)\ntheorem t: |- P(a)\nproof\n  exact ax\nqed").unwrap();
    }

    #[test]
    fn axiom_reference_with_wrong_conclusion() {
        assert!(run("axiom ax: P(a)\ntheorem t: |- P(b)\nproof\n  exact ax\nqed").is_err());
    }

    #[test]
    fn theorem_applied_with_instantiation() {
        let src = "theorem id: |- A => A\nproof\n  assume a: A\n  have r: A => A := ImpliesIntro(a, a)\n  exact r\nqed\n\
                   theorem use_id: |- (B AND C) => (B AND C)\nproof\n  exact id\nqed";
        run(src).unwrap();
    }

    #[test]
    fn theorem_applied_with_hypotheses() {
        let src = "theorem comm: A AND B |- B AND A\nproof\n  assume h: A AND B\n  have a: A := AndElimLeft(h)\n  have b: B := AndElimRight(h)\n  have r: B AND A := AndIntro(b, a)\n  exact r\nqed\n\
                   theorem use_comm: X AND Y |- Y AND X\nproof\n  assume h: X AND Y\n  have r: Y AND X := comm(h)\n  exact r\nqed";
        run(src).unwrap();
    }

    #[test]
    fn theorem_application_rejects_non_instance() {
        let src = "theorem comm: A AND B |- B AND A\nproof\n  assume h: A AND B\n  have a: A := AndElimLeft(h)\n  have b: B := AndElimRight(h)\n  have r: B AND A := AndIntro(b, a)\n  exact r\nqed\n\
                   theorem bad: X AND Y |- X AND Y\nproof\n  assume h: X AND Y\n  have r: X AND Y := comm(h)\n  exact r\nqed";
        let msg = message(src);
        assert!(msg.contains("not an instance"), "{}", msg);
    }

    #[test]
    fn theorem_application_checks_argument_count() {
        let src = "theorem id: |- A => A\nproof\n  assume a: A\n  have r: A => A := ImpliesIntro(a, a)\n  exact r\nqed\n\
                   theorem bad: X |- X => X\nproof\n  assume x: X\n  have r: X => X := id(x)\n  exact r\nqed";
        let msg = message(src);
        assert!(msg.contains("expects 0 arguments"), "{}", msg);
    }

    #[test]
    fn theorem_application_inherits_dependencies() {
        let src = "theorem id: |- A => A\nproof\n  assume a: A\n  have r: A => A := ImpliesIntro(a, a)\n  exact r\nqed\n\
                   theorem snd: X, Y |- Y\nproof\n  assume x: X\n  assume y: Y\n  have r: Y => Y := id\n  have s: Y := ModusPonens(r, y)\n  exact s\nqed";
        run(src).unwrap();
    }

    #[test]
    fn axiom_can_be_a_theorem_argument() {
        let src = "axiom ax: X AND Y\n\
                   theorem t: |- Y AND X\nproof\n  have r: Y AND X := and_comm(ax)\n  exact r\nqed";
        let mut ctx = crate::stdlib::load_stdlib().unwrap();
        for item in parse_source(src).unwrap() {
            match item {
                Item::Axiom { name, formula, .. } => {
                    ctx.axioms.insert(name, formula);
                }
                Item::Theorem(theorem) => check(&theorem, &ctx).unwrap(),
                Item::Import { .. } => unreachable!(),
            }
        }
    }

    #[test]
    fn forall_elim_with_predicate() {
        let src = "theorem t: forall X: human(X), human(socrates) |- human(socrates)\nproof\n  assume all: forall X: human(X)\n  assume h: human(socrates)\n  have s: human(socrates) := ForallElim(all, socrates)\n  exact s\nqed";
        run(src).unwrap();
    }

    #[test]
    fn forall_intro_blocked_by_hypothesis() {
        let src = "theorem t: P(x) |- forall x: P(x)\nproof\n  assume h: P(x)\n  have g: forall x: P(x) := ForallIntro(h, x)\n  exact g\nqed";
        let msg = message(src);
        assert!(msg.contains("occurs free"), "{}", msg);
    }

    #[test]
    fn false_elim_proves_anything() {
        run("theorem t: FALSE |- Z\nproof\n  assume f: FALSE\n  have z: Z := FalseElim(f)\n  exact z\nqed")
            .unwrap();
    }

    #[test]
    fn exact_may_apply_a_rule_directly() {
        run("theorem t: A, B |- A AND B\nproof\n  assume a: A\n  assume b: B\n  exact AndIntro(a, b)\nqed")
            .unwrap();
    }
}
