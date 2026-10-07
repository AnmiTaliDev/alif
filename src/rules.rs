// SPDX-License-Identifier: GPL-3.0-only

use std::collections::{BTreeSet, HashMap};

use crate::context::Context;
use crate::error::RuleError;
use crate::formula_ops::{alpha_eq, free_vars, replaces, substitute};
use crate::term::{Formula, Term};

#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub formula: Formula,
    pub deps: BTreeSet<String>,
    pub assumption: bool,
}

pub type Env = HashMap<String, Fact>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    AndIntro,
    AndElimLeft,
    AndElimRight,
    OrIntroLeft,
    OrIntroRight,
    OrElim,
    ModusPonens,
    ImpliesIntro,
    NotIntro,
    NotElim,
    FalseElim,
    IffIntro,
    IffElim,
    ForallIntro,
    ForallElim,
    ExistsIntro,
    ExistsElim,
    EqRefl,
    EqSym,
    EqTrans,
    EqSubst,
}

const RULES: [(&str, Rule); 21] = [
    ("AndIntro", Rule::AndIntro),
    ("AndElimLeft", Rule::AndElimLeft),
    ("AndElimRight", Rule::AndElimRight),
    ("OrIntroLeft", Rule::OrIntroLeft),
    ("OrIntroRight", Rule::OrIntroRight),
    ("OrElim", Rule::OrElim),
    ("ModusPonens", Rule::ModusPonens),
    ("ImpliesIntro", Rule::ImpliesIntro),
    ("NotIntro", Rule::NotIntro),
    ("NotElim", Rule::NotElim),
    ("FalseElim", Rule::FalseElim),
    ("IffIntro", Rule::IffIntro),
    ("IffElim", Rule::IffElim),
    ("ForallIntro", Rule::ForallIntro),
    ("ForallElim", Rule::ForallElim),
    ("ExistsIntro", Rule::ExistsIntro),
    ("ExistsElim", Rule::ExistsElim),
    ("EqRefl", Rule::EqRefl),
    ("EqSym", Rule::EqSym),
    ("EqTrans", Rule::EqTrans),
    ("EqSubst", Rule::EqSubst),
];

impl Rule {
    pub fn from_name(name: &str) -> Option<Rule> {
        RULES.iter().find(|(n, _)| *n == name).map(|(_, r)| *r)
    }

    pub fn name(&self) -> &'static str {
        RULES
            .iter()
            .find(|(_, r)| r == self)
            .map(|(n, _)| *n)
            .expect("every rule is listed in RULES")
    }

    pub fn all() -> impl Iterator<Item = Rule> {
        RULES.iter().map(|(_, r)| *r)
    }
}

pub fn lookup_fact(env: &Env, ctx: &Context, name: &str) -> Option<Fact> {
    if let Some(fact) = env.get(name) {
        return Some(fact.clone());
    }
    ctx.axioms
        .get(name)
        .map(|formula| derived(formula.clone(), BTreeSet::new()))
}

fn derived(formula: Formula, deps: BTreeSet<String>) -> Fact {
    Fact {
        formula,
        deps,
        assumption: false,
    }
}

fn union(a: &BTreeSet<String>, b: &BTreeSet<String>) -> BTreeSet<String> {
    a.union(b).cloned().collect()
}

fn without(deps: &BTreeSet<String>, name: &str) -> BTreeSet<String> {
    let mut result = deps.clone();
    result.remove(name);
    result
}

struct Call<'a> {
    rule: Rule,
    args: &'a [Term],
    env: &'a Env,
    ctx: &'a Context,
    goal: &'a Formula,
}

impl<'a> Call<'a> {
    fn fail<T>(&self, reason: impl Into<String>) -> Result<T, RuleError> {
        Err(RuleError {
            rule: self.rule.name().to_string(),
            reason: reason.into(),
        })
    }

    fn arity(&self, n: usize) -> Result<(), RuleError> {
        if self.args.len() == n {
            Ok(())
        } else {
            self.fail(format!(
                "expects {} argument{}, got {}",
                n,
                if n == 1 { "" } else { "s" },
                self.args.len()
            ))
        }
    }

    fn name(&self, i: usize) -> Result<&'a str, RuleError> {
        match &self.args[i] {
            Term::Name(name) => Ok(name),
            Term::App(..) => self.fail(format!("argument {} must be a plain name", i + 1)),
        }
    }

    fn fact(&self, i: usize) -> Result<Fact, RuleError> {
        let name = self.name(i)?;
        match lookup_fact(self.env, self.ctx, name) {
            Some(fact) => Ok(fact),
            None => self.fail(format!("`{}` is not in scope and is not an axiom", name)),
        }
    }

    fn assumption(&self, i: usize) -> Result<(&'a str, Fact), RuleError> {
        let name = self.name(i)?;
        let fact = self.fact(i)?;
        if fact.assumption {
            Ok((name, fact))
        } else {
            self.fail(format!("`{}` is not an assumption", name))
        }
    }

    fn term(&self, i: usize) -> &'a Term {
        &self.args[i]
    }

    fn is_fixed(&self, var: &str, deps: &BTreeSet<String>) -> bool {
        deps.iter()
            .filter_map(|d| self.env.get(d))
            .any(|f| free_vars(&f.formula).contains(var))
            || self.ctx.axioms.values().any(|f| free_vars(f).contains(var))
    }
}

pub fn apply_rule(
    rule: Rule,
    args: &[Term],
    env: &Env,
    ctx: &Context,
    goal: &Formula,
) -> Result<Fact, RuleError> {
    let call = Call {
        rule,
        args,
        env,
        ctx,
        goal,
    };
    match rule {
        Rule::AndIntro
        | Rule::AndElimLeft
        | Rule::AndElimRight
        | Rule::OrIntroLeft
        | Rule::OrIntroRight
        | Rule::OrElim
        | Rule::ModusPonens
        | Rule::ImpliesIntro
        | Rule::NotIntro
        | Rule::NotElim
        | Rule::FalseElim
        | Rule::IffIntro
        | Rule::IffElim => propositional(&call),
        Rule::ForallIntro | Rule::ForallElim | Rule::ExistsIntro | Rule::ExistsElim => {
            quantifier(&call)
        }
        Rule::EqRefl | Rule::EqSym | Rule::EqTrans | Rule::EqSubst => equality(&call),
    }
}

fn propositional(call: &Call) -> Result<Fact, RuleError> {
    match call.rule {
        Rule::AndIntro => {
            call.arity(2)?;
            let a = call.fact(0)?;
            let b = call.fact(1)?;
            Ok(derived(
                Formula::And(Box::new(a.formula.clone()), Box::new(b.formula.clone())),
                union(&a.deps, &b.deps),
            ))
        }
        Rule::AndElimLeft => {
            call.arity(1)?;
            let a = call.fact(0)?;
            match &a.formula {
                Formula::And(left, _) => Ok(derived(left.as_ref().clone(), a.deps.clone())),
                _ => call.fail("argument must be a conjunction (A AND B)"),
            }
        }
        Rule::AndElimRight => {
            call.arity(1)?;
            let a = call.fact(0)?;
            match &a.formula {
                Formula::And(_, right) => Ok(derived(right.as_ref().clone(), a.deps.clone())),
                _ => call.fail("argument must be a conjunction (A AND B)"),
            }
        }
        Rule::OrIntroLeft => {
            call.arity(1)?;
            let a = call.fact(0)?;
            match call.goal {
                Formula::Or(left, _) if alpha_eq(left, &a.formula) => {
                    Ok(derived(call.goal.clone(), a.deps.clone()))
                }
                _ => call.fail(format!(
                    "the declared formula must be a disjunction whose left side is `{}`",
                    a.formula
                )),
            }
        }
        Rule::OrIntroRight => {
            call.arity(1)?;
            let a = call.fact(0)?;
            match call.goal {
                Formula::Or(_, right) if alpha_eq(right, &a.formula) => {
                    Ok(derived(call.goal.clone(), a.deps.clone()))
                }
                _ => call.fail(format!(
                    "the declared formula must be a disjunction whose right side is `{}`",
                    a.formula
                )),
            }
        }
        Rule::OrElim => {
            call.arity(5)?;
            let disjunction = call.fact(0)?;
            let (left_name, left) = call.assumption(1)?;
            let from_left = call.fact(2)?;
            let (right_name, right) = call.assumption(3)?;
            let from_right = call.fact(4)?;
            let Formula::Or(p, q) = &disjunction.formula else {
                return call.fail("first argument must be a disjunction (A OR B)");
            };
            if !alpha_eq(p, &left.formula) {
                return call.fail(format!(
                    "assumption `{}` must be `{}`",
                    left_name, p
                ));
            }
            if !alpha_eq(q, &right.formula) {
                return call.fail(format!(
                    "assumption `{}` must be `{}`",
                    right_name, q
                ));
            }
            if !alpha_eq(&from_left.formula, &from_right.formula) {
                return call.fail(format!(
                    "both branches must derive the same formula, got `{}` and `{}`",
                    from_left.formula, from_right.formula
                ));
            }
            let deps = union(
                &disjunction.deps,
                &union(
                    &without(&from_left.deps, left_name),
                    &without(&from_right.deps, right_name),
                ),
            );
            Ok(derived(from_left.formula.clone(), deps))
        }
        Rule::ModusPonens => {
            call.arity(2)?;
            let x = call.fact(0)?;
            let y = call.fact(1)?;
            let consequent = match (&x.formula, &y.formula) {
                (Formula::Implies(p, q), _) if alpha_eq(p, &y.formula) => q,
                (_, Formula::Implies(p, q)) if alpha_eq(p, &x.formula) => q,
                _ => {
                    return call.fail(
                        "arguments must be an implication (A => B) and a formula equal to A",
                    )
                }
            };
            Ok(derived(
                consequent.as_ref().clone(),
                union(&x.deps, &y.deps),
            ))
        }
        Rule::ImpliesIntro => {
            call.arity(2)?;
            let (name, assumed) = call.assumption(0)?;
            let conclusion = call.fact(1)?;
            Ok(derived(
                Formula::Implies(
                    Box::new(assumed.formula.clone()),
                    Box::new(conclusion.formula.clone()),
                ),
                without(&conclusion.deps, name),
            ))
        }
        Rule::NotIntro => {
            call.arity(2)?;
            let (name, assumed) = call.assumption(0)?;
            let contradiction = call.fact(1)?;
            if contradiction.formula != Formula::Bottom {
                return call.fail(format!(
                    "second argument must be FALSE, got `{}`",
                    contradiction.formula
                ));
            }
            Ok(derived(
                Formula::Not(Box::new(assumed.formula.clone())),
                without(&contradiction.deps, name),
            ))
        }
        Rule::NotElim => {
            call.arity(2)?;
            let x = call.fact(0)?;
            let y = call.fact(1)?;
            let contradictory = match (&x.formula, &y.formula) {
                (f, Formula::Not(g)) | (Formula::Not(g), f) => alpha_eq(f, g),
                _ => false,
            };
            if contradictory {
                Ok(derived(Formula::Bottom, union(&x.deps, &y.deps)))
            } else {
                call.fail("arguments must be a formula and its negation")
            }
        }
        Rule::FalseElim => {
            call.arity(1)?;
            let a = call.fact(0)?;
            if a.formula == Formula::Bottom {
                Ok(derived(call.goal.clone(), a.deps.clone()))
            } else {
                call.fail(format!("argument must be FALSE, got `{}`", a.formula))
            }
        }
        Rule::IffIntro => {
            call.arity(2)?;
            let forward = call.fact(0)?;
            let backward = call.fact(1)?;
            match (&forward.formula, &backward.formula) {
                (Formula::Implies(p, q), Formula::Implies(q2, p2))
                    if alpha_eq(p, p2) && alpha_eq(q, q2) =>
                {
                    Ok(derived(
                        Formula::Iff(p.clone(), q.clone()),
                        union(&forward.deps, &backward.deps),
                    ))
                }
                _ => call.fail("arguments must be A => B and B => A"),
            }
        }
        Rule::IffElim => {
            call.arity(2)?;
            let equivalence = call.fact(0)?;
            let side = call.fact(1)?;
            let Formula::Iff(p, q) = &equivalence.formula else {
                return call.fail("first argument must be an equivalence (A <=> B)");
            };
            let other = if alpha_eq(p, &side.formula) {
                q
            } else if alpha_eq(q, &side.formula) {
                p
            } else {
                return call.fail("second argument must equal one side of the equivalence");
            };
            Ok(derived(
                other.as_ref().clone(),
                union(&equivalence.deps, &side.deps),
            ))
        }
        _ => unreachable!("not a propositional rule"),
    }
}

fn quantifier(call: &Call) -> Result<Fact, RuleError> {
    match call.rule {
        Rule::ForallIntro => {
            call.arity(2)?;
            let body = call.fact(0)?;
            let var = call.name(1)?;
            if call.is_fixed(var, &body.deps) {
                return call.fail(format!(
                    "variable `{}` occurs free in an undischarged assumption or an axiom",
                    var
                ));
            }
            Ok(derived(
                Formula::Forall(var.to_string(), Box::new(body.formula.clone())),
                body.deps.clone(),
            ))
        }
        Rule::ForallElim => {
            call.arity(2)?;
            let universal = call.fact(0)?;
            let Formula::Forall(var, body) = &universal.formula else {
                return call.fail("first argument must be a universal formula (forall X: F)");
            };
            Ok(derived(
                substitute(body, var, call.term(1)),
                universal.deps.clone(),
            ))
        }
        Rule::ExistsIntro => {
            call.arity(2)?;
            let instance = call.fact(0)?;
            let witness = call.term(1);
            let Formula::Exists(var, body) = call.goal else {
                return call.fail("the declared formula must be existential (exists X: F)");
            };
            let expected = substitute(body, var, witness);
            if alpha_eq(&expected, &instance.formula) {
                Ok(derived(call.goal.clone(), instance.deps.clone()))
            } else {
                call.fail(format!(
                    "expected the first argument to be `{}`, got `{}`",
                    expected, instance.formula
                ))
            }
        }
        Rule::ExistsElim => {
            call.arity(4)?;
            let existential = call.fact(0)?;
            let (name, assumed) = call.assumption(1)?;
            let conclusion = call.fact(2)?;
            let witness = call.name(3)?;
            let Formula::Exists(var, body) = &existential.formula else {
                return call.fail("first argument must be an existential formula (exists X: F)");
            };
            let expected = substitute(body, var, &Term::Name(witness.to_string()));
            if !alpha_eq(&expected, &assumed.formula) {
                return call.fail(format!(
                    "assumption `{}` must be `{}`",
                    name, expected
                ));
            }
            let deps = union(&existential.deps, &without(&conclusion.deps, name));
            let escapes = free_vars(&existential.formula).contains(witness)
                || free_vars(&conclusion.formula).contains(witness)
                || call.is_fixed(witness, &deps);
            if escapes {
                return call.fail(format!(
                    "witness `{}` is not fresh: it occurs in the existential formula, the conclusion, an undischarged assumption or an axiom",
                    witness
                ));
            }
            Ok(derived(conclusion.formula.clone(), deps))
        }
        _ => unreachable!("not a quantifier rule"),
    }
}

fn equality(call: &Call) -> Result<Fact, RuleError> {
    match call.rule {
        Rule::EqRefl => {
            call.arity(0)?;
            match call.goal {
                Formula::Eq(a, b) if a == b => Ok(derived(call.goal.clone(), BTreeSet::new())),
                _ => call.fail("the declared formula must have the form t = t"),
            }
        }
        Rule::EqSym => {
            call.arity(1)?;
            let eq = call.fact(0)?;
            match &eq.formula {
                Formula::Eq(a, b) => Ok(derived(
                    Formula::Eq(b.clone(), a.clone()),
                    eq.deps.clone(),
                )),
                _ => call.fail("argument must be an equality (s = t)"),
            }
        }
        Rule::EqTrans => {
            call.arity(2)?;
            let first = call.fact(0)?;
            let second = call.fact(1)?;
            match (&first.formula, &second.formula) {
                (Formula::Eq(a, b), Formula::Eq(b2, c)) if b == b2 => Ok(derived(
                    Formula::Eq(a.clone(), c.clone()),
                    union(&first.deps, &second.deps),
                )),
                _ => call.fail("arguments must be s = t and t = u"),
            }
        }
        Rule::EqSubst => {
            call.arity(2)?;
            let eq = call.fact(0)?;
            let source = call.fact(1)?;
            let Formula::Eq(from, to) = &eq.formula else {
                return call.fail("first argument must be an equality (s = t)");
            };
            if replaces(&source.formula, call.goal, from, to) {
                Ok(derived(call.goal.clone(), union(&eq.deps, &source.deps)))
            } else {
                call.fail(format!(
                    "the declared formula must be `{}` with some occurrences of `{}` replaced by `{}`",
                    source.formula, from, to
                ))
            }
        }
        _ => unreachable!("not an equality rule"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(name: &str) -> Formula {
        Formula::Atom(name.to_string(), vec![])
    }

    fn name(n: &str) -> Term {
        Term::Name(n.to_string())
    }

    fn pred(p: &str, args: Vec<Term>) -> Formula {
        Formula::Atom(p.to_string(), args)
    }

    fn and(a: Formula, b: Formula) -> Formula {
        Formula::And(Box::new(a), Box::new(b))
    }

    fn or(a: Formula, b: Formula) -> Formula {
        Formula::Or(Box::new(a), Box::new(b))
    }

    fn not(a: Formula) -> Formula {
        Formula::Not(Box::new(a))
    }

    fn implies(a: Formula, b: Formula) -> Formula {
        Formula::Implies(Box::new(a), Box::new(b))
    }

    fn forall(x: &str, body: Formula) -> Formula {
        Formula::Forall(x.to_string(), Box::new(body))
    }

    fn exists(x: &str, body: Formula) -> Formula {
        Formula::Exists(x.to_string(), Box::new(body))
    }

    fn deps(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    fn proven(formula: Formula, on: &[&str]) -> Fact {
        Fact {
            formula,
            deps: deps(on),
            assumption: false,
        }
    }

    fn assumed(name: &str, formula: Formula) -> (String, Fact) {
        (
            name.to_string(),
            Fact {
                formula,
                deps: deps(&[name]),
                assumption: true,
            },
        )
    }

    fn env(entries: Vec<(String, Fact)>) -> Env {
        entries.into_iter().collect()
    }

    fn run(rule: Rule, args: &[&str], env: &Env, goal: &Formula) -> Result<Fact, RuleError> {
        let terms: Vec<Term> = args.iter().map(|a| name(a)).collect();
        apply_rule(rule, &terms, env, &Context::default(), goal)
    }

    fn run_terms(
        rule: Rule,
        args: &[Term],
        env: &Env,
        ctx: &Context,
        goal: &Formula,
    ) -> Result<Fact, RuleError> {
        apply_rule(rule, args, env, ctx, goal)
    }

    fn any_goal() -> Formula {
        atom("_goal")
    }

    #[test]
    fn rule_names_round_trip() {
        for rule in Rule::all() {
            assert_eq!(Rule::from_name(rule.name()), Some(rule));
        }
        assert_eq!(Rule::all().count(), 21);
    }

    #[test]
    fn unknown_rule_name() {
        assert_eq!(Rule::from_name("Nope"), None);
    }

    #[test]
    fn wrong_arity_is_error() {
        let e = env(vec![assumed("a", atom("A"))]);
        let err = run(Rule::AndIntro, &["a"], &e, &any_goal()).unwrap_err();
        assert!(err.reason.contains("expects 2 arguments"));
    }

    #[test]
    fn unknown_argument_is_error() {
        let e = env(vec![]);
        let err = run(Rule::AndElimLeft, &["x"], &e, &any_goal()).unwrap_err();
        assert!(err.reason.contains("not in scope"));
    }

    #[test]
    fn and_intro_unions_dependencies() {
        let e = env(vec![assumed("a", atom("A")), assumed("b", atom("B"))]);
        let result = run(Rule::AndIntro, &["a", "b"], &e, &any_goal()).unwrap();
        assert_eq!(result.formula, and(atom("A"), atom("B")));
        assert_eq!(result.deps, deps(&["a", "b"]));
        assert!(!result.assumption);
    }

    #[test]
    fn and_elim_left_and_right() {
        let e = env(vec![assumed("h", and(atom("A"), atom("B")))]);
        let left = run(Rule::AndElimLeft, &["h"], &e, &any_goal()).unwrap();
        let right = run(Rule::AndElimRight, &["h"], &e, &any_goal()).unwrap();
        assert_eq!(left.formula, atom("A"));
        assert_eq!(right.formula, atom("B"));
        assert_eq!(left.deps, deps(&["h"]));
    }

    #[test]
    fn and_elim_requires_conjunction() {
        let e = env(vec![assumed("h", atom("A"))]);
        assert!(run(Rule::AndElimLeft, &["h"], &e, &any_goal()).is_err());
        assert!(run(Rule::AndElimRight, &["h"], &e, &any_goal()).is_err());
    }

    #[test]
    fn or_intro_left_uses_goal() {
        let e = env(vec![assumed("a", atom("A"))]);
        let goal = or(atom("A"), atom("B"));
        let result = run(Rule::OrIntroLeft, &["a"], &e, &goal).unwrap();
        assert_eq!(result.formula, goal);
        assert_eq!(result.deps, deps(&["a"]));
    }

    #[test]
    fn or_intro_left_rejects_wrong_side() {
        let e = env(vec![assumed("a", atom("A"))]);
        let goal = or(atom("B"), atom("A"));
        assert!(run(Rule::OrIntroLeft, &["a"], &e, &goal).is_err());
        assert!(run(Rule::OrIntroLeft, &["a"], &e, &atom("A")).is_err());
    }

    #[test]
    fn or_intro_right_uses_goal() {
        let e = env(vec![assumed("b", atom("B"))]);
        let goal = or(atom("A"), atom("B"));
        let result = run(Rule::OrIntroRight, &["b"], &e, &goal).unwrap();
        assert_eq!(result.formula, goal);
        assert!(run(Rule::OrIntroRight, &["b"], &e, &or(atom("B"), atom("A"))).is_err());
    }

    #[test]
    fn or_elim_discharges_both_branches() {
        let e = env(vec![
            assumed("o", or(atom("A"), atom("B"))),
            assumed("x", atom("A")),
            assumed("y", atom("B")),
            ("cx".to_string(), proven(atom("C"), &["x", "k"])),
            ("cy".to_string(), proven(atom("C"), &["y"])),
        ]);
        let result = run(Rule::OrElim, &["o", "x", "cx", "y", "cy"], &e, &any_goal()).unwrap();
        assert_eq!(result.formula, atom("C"));
        assert_eq!(result.deps, deps(&["o", "k"]));
    }

    #[test]
    fn or_elim_requires_matching_assumptions() {
        let e = env(vec![
            assumed("o", or(atom("A"), atom("B"))),
            assumed("x", atom("B")),
            assumed("y", atom("B")),
            ("cx".to_string(), proven(atom("C"), &["x"])),
            ("cy".to_string(), proven(atom("C"), &["y"])),
        ]);
        assert!(run(Rule::OrElim, &["o", "x", "cx", "y", "cy"], &e, &any_goal()).is_err());
    }

    #[test]
    fn or_elim_requires_equal_branches() {
        let e = env(vec![
            assumed("o", or(atom("A"), atom("B"))),
            assumed("x", atom("A")),
            assumed("y", atom("B")),
            ("cx".to_string(), proven(atom("C"), &["x"])),
            ("cy".to_string(), proven(atom("D"), &["y"])),
        ]);
        assert!(run(Rule::OrElim, &["o", "x", "cx", "y", "cy"], &e, &any_goal()).is_err());
    }

    #[test]
    fn or_elim_requires_assumptions() {
        let e = env(vec![
            assumed("o", or(atom("A"), atom("B"))),
            ("x".to_string(), proven(atom("A"), &[])),
            assumed("y", atom("B")),
            ("cx".to_string(), proven(atom("C"), &[])),
            ("cy".to_string(), proven(atom("C"), &["y"])),
        ]);
        let err = run(Rule::OrElim, &["o", "x", "cx", "y", "cy"], &e, &any_goal()).unwrap_err();
        assert!(err.reason.contains("not an assumption"));
    }

    #[test]
    fn modus_ponens_either_order() {
        let e = env(vec![
            assumed("i", implies(atom("A"), atom("B"))),
            assumed("a", atom("A")),
        ]);
        let forward = run(Rule::ModusPonens, &["i", "a"], &e, &any_goal()).unwrap();
        let backward = run(Rule::ModusPonens, &["a", "i"], &e, &any_goal()).unwrap();
        assert_eq!(forward.formula, atom("B"));
        assert_eq!(backward.formula, atom("B"));
        assert_eq!(forward.deps, deps(&["i", "a"]));
    }

    #[test]
    fn modus_ponens_wrong_antecedent() {
        let e = env(vec![
            assumed("i", implies(atom("A"), atom("B"))),
            assumed("c", atom("C")),
        ]);
        assert!(run(Rule::ModusPonens, &["i", "c"], &e, &any_goal()).is_err());
    }

    #[test]
    fn implies_intro_discharges_assumption() {
        let e = env(vec![
            assumed("x", atom("A")),
            ("y".to_string(), proven(atom("B"), &["x", "h"])),
        ]);
        let result = run(Rule::ImpliesIntro, &["x", "y"], &e, &any_goal()).unwrap();
        assert_eq!(result.formula, implies(atom("A"), atom("B")));
        assert_eq!(result.deps, deps(&["h"]));
    }

    #[test]
    fn implies_intro_requires_assumption() {
        let e = env(vec![
            ("x".to_string(), proven(atom("A"), &[])),
            assumed("y", atom("B")),
        ]);
        assert!(run(Rule::ImpliesIntro, &["x", "y"], &e, &any_goal()).is_err());
    }

    #[test]
    fn not_intro_requires_false() {
        let e = env(vec![assumed("x", atom("A")), assumed("y", atom("B"))]);
        assert!(run(Rule::NotIntro, &["x", "y"], &e, &any_goal()).is_err());
    }

    #[test]
    fn not_intro_discharges_assumption() {
        let e = env(vec![
            assumed("x", atom("A")),
            ("f".to_string(), proven(Formula::Bottom, &["x", "h"])),
        ]);
        let result = run(Rule::NotIntro, &["x", "f"], &e, &any_goal()).unwrap();
        assert_eq!(result.formula, not(atom("A")));
        assert_eq!(result.deps, deps(&["h"]));
    }

    #[test]
    fn not_elim_either_order() {
        let e = env(vec![assumed("a", atom("A")), assumed("n", not(atom("A")))]);
        let one = run(Rule::NotElim, &["a", "n"], &e, &any_goal()).unwrap();
        let two = run(Rule::NotElim, &["n", "a"], &e, &any_goal()).unwrap();
        assert_eq!(one.formula, Formula::Bottom);
        assert_eq!(two.formula, Formula::Bottom);
        assert_eq!(one.deps, deps(&["a", "n"]));
    }

    #[test]
    fn not_elim_rejects_unrelated() {
        let e = env(vec![assumed("b", atom("B")), assumed("n", not(atom("A")))]);
        assert!(run(Rule::NotElim, &["b", "n"], &e, &any_goal()).is_err());
    }

    #[test]
    fn false_elim_derives_goal() {
        let e = env(vec![assumed("f", Formula::Bottom)]);
        let result = run(Rule::FalseElim, &["f"], &e, &atom("Z")).unwrap();
        assert_eq!(result.formula, atom("Z"));
        assert_eq!(result.deps, deps(&["f"]));
    }

    #[test]
    fn false_elim_requires_false() {
        let e = env(vec![assumed("a", atom("A"))]);
        assert!(run(Rule::FalseElim, &["a"], &e, &atom("Z")).is_err());
    }

    #[test]
    fn iff_intro_and_elim() {
        let e = env(vec![
            assumed("f", implies(atom("A"), atom("B"))),
            assumed("b", implies(atom("B"), atom("A"))),
        ]);
        let iff = run(Rule::IffIntro, &["f", "b"], &e, &any_goal()).unwrap();
        assert_eq!(
            iff.formula,
            Formula::Iff(Box::new(atom("A")), Box::new(atom("B")))
        );
        let e = env(vec![
            ("i".to_string(), iff),
            assumed("a", atom("A")),
            assumed("bb", atom("B")),
        ]);
        let to_b = run(Rule::IffElim, &["i", "a"], &e, &any_goal()).unwrap();
        let to_a = run(Rule::IffElim, &["i", "bb"], &e, &any_goal()).unwrap();
        assert_eq!(to_b.formula, atom("B"));
        assert_eq!(to_a.formula, atom("A"));
    }

    #[test]
    fn iff_intro_rejects_mismatched_directions() {
        let e = env(vec![
            assumed("f", implies(atom("A"), atom("B"))),
            assumed("b", implies(atom("C"), atom("A"))),
        ]);
        assert!(run(Rule::IffIntro, &["f", "b"], &e, &any_goal()).is_err());
    }

    #[test]
    fn forall_intro_generalises_free_variable() {
        let e = env(vec![(
            "p".to_string(),
            proven(pred("P", vec![name("X")]), &[]),
        )]);
        let result = run(Rule::ForallIntro, &["p", "X"], &e, &any_goal()).unwrap();
        assert_eq!(result.formula, forall("X", pred("P", vec![name("X")])));
    }

    #[test]
    fn forall_intro_blocked_by_assumption() {
        let e = env(vec![
            assumed("h", pred("P", vec![name("X")])),
            ("p".to_string(), proven(pred("P", vec![name("X")]), &["h"])),
        ]);
        let err = run(Rule::ForallIntro, &["p", "X"], &e, &any_goal()).unwrap_err();
        assert!(err.reason.contains("occurs free"));
    }

    #[test]
    fn forall_intro_allows_variable_absent_from_assumptions() {
        let e = env(vec![
            assumed("h", pred("Q", vec![name("Y")])),
            ("p".to_string(), proven(pred("P", vec![name("X")]), &["h"])),
        ]);
        assert!(run(Rule::ForallIntro, &["p", "X"], &e, &any_goal()).is_ok());
    }

    #[test]
    fn forall_intro_blocked_by_axiom() {
        let e = env(vec![(
            "p".to_string(),
            proven(pred("P", vec![name("c")]), &[]),
        )]);
        let mut ctx = Context::default();
        ctx.axioms
            .insert("ax".to_string(), pred("P", vec![name("c")]));
        let result = run_terms(
            Rule::ForallIntro,
            &[name("p"), name("c")],
            &e,
            &ctx,
            &any_goal(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn axioms_can_be_passed_to_rules() {
        let mut ctx = Context::default();
        ctx.axioms.insert(
            "all".to_string(),
            forall("X", pred("P", vec![name("X")])),
        );
        let result = run_terms(
            Rule::ForallElim,
            &[name("all"), name("a")],
            &env(vec![]),
            &ctx,
            &any_goal(),
        )
        .unwrap();
        assert_eq!(result.formula, pred("P", vec![name("a")]));
        assert!(result.deps.is_empty());
    }

    #[test]
    fn local_names_take_precedence_over_axioms() {
        let mut ctx = Context::default();
        ctx.axioms.insert("h".to_string(), atom("FromAxiom"));
        let e = env(vec![assumed("h", and(atom("A"), atom("B")))]);
        let result = run_terms(Rule::AndElimLeft, &[name("h")], &e, &ctx, &any_goal()).unwrap();
        assert_eq!(result.formula, atom("A"));
    }

    #[test]
    fn forall_intro_requires_plain_variable() {
        let e = env(vec![("p".to_string(), proven(atom("P"), &[]))]);
        let args = [
            name("p"),
            Term::App("f".to_string(), vec![name("a")]),
        ];
        assert!(run_terms(Rule::ForallIntro, &args, &e, &Context::default(), &any_goal()).is_err());
    }

    #[test]
    fn forall_elim_substitutes_term() {
        let e = env(vec![assumed("all", forall("X", pred("P", vec![name("X")])))]);
        let result = run_terms(
            Rule::ForallElim,
            &[name("all"), name("socrates")],
            &e,
            &Context::default(),
            &any_goal(),
        )
        .unwrap();
        assert_eq!(result.formula, pred("P", vec![name("socrates")]));
        assert_eq!(result.deps, deps(&["all"]));
    }

    #[test]
    fn forall_elim_accepts_function_term() {
        let e = env(vec![assumed("all", forall("X", pred("P", vec![name("X")])))]);
        let term = Term::App("f".to_string(), vec![name("a")]);
        let result = run_terms(
            Rule::ForallElim,
            &[name("all"), term.clone()],
            &e,
            &Context::default(),
            &any_goal(),
        )
        .unwrap();
        assert_eq!(result.formula, pred("P", vec![term]));
    }

    #[test]
    fn forall_elim_avoids_capture() {
        let body = exists("Y", pred("R", vec![name("X"), name("Y")]));
        let e = env(vec![assumed("all", forall("X", body))]);
        let result = run_terms(
            Rule::ForallElim,
            &[name("all"), name("Y")],
            &e,
            &Context::default(),
            &any_goal(),
        )
        .unwrap();
        let expected = exists("Y_1", pred("R", vec![name("Y"), name("Y_1")]));
        assert_eq!(result.formula, expected);
    }

    #[test]
    fn forall_elim_requires_universal() {
        let e = env(vec![assumed("h", atom("A"))]);
        assert!(run(Rule::ForallElim, &["h", "a"], &e, &any_goal()).is_err());
    }

    #[test]
    fn exists_intro_checks_witness() {
        let e = env(vec![assumed("p", pred("P", vec![name("a")]))]);
        let goal = exists("X", pred("P", vec![name("X")]));
        let result = run(Rule::ExistsIntro, &["p", "a"], &e, &goal).unwrap();
        assert_eq!(result.formula, goal);
        assert!(run(Rule::ExistsIntro, &["p", "b"], &e, &goal).is_err());
    }

    #[test]
    fn exists_intro_requires_existential_goal() {
        let e = env(vec![assumed("p", pred("P", vec![name("a")]))]);
        assert!(run(Rule::ExistsIntro, &["p", "a"], &e, &atom("A")).is_err());
    }

    #[test]
    fn exists_elim_discharges_witness_assumption() {
        let e = env(vec![
            assumed("ex", exists("X", pred("P", vec![name("X")]))),
            assumed("w", pred("P", vec![name("c")])),
            ("r".to_string(), proven(atom("Q"), &["w"])),
        ]);
        let result = run(Rule::ExistsElim, &["ex", "w", "r", "c"], &e, &any_goal()).unwrap();
        assert_eq!(result.formula, atom("Q"));
        assert_eq!(result.deps, deps(&["ex"]));
    }

    #[test]
    fn exists_elim_rejects_witness_in_conclusion() {
        let e = env(vec![
            assumed("ex", exists("X", pred("P", vec![name("X")]))),
            assumed("w", pred("P", vec![name("c")])),
            ("r".to_string(), proven(pred("Q", vec![name("c")]), &["w"])),
        ]);
        let err = run(Rule::ExistsElim, &["ex", "w", "r", "c"], &e, &any_goal()).unwrap_err();
        assert!(err.reason.contains("not fresh"));
    }

    #[test]
    fn exists_elim_rejects_witness_in_other_assumption() {
        let e = env(vec![
            assumed("ex", exists("X", pred("P", vec![name("X")]))),
            assumed("w", pred("P", vec![name("c")])),
            assumed("other", pred("S", vec![name("c")])),
            ("r".to_string(), proven(atom("Q"), &["w", "other"])),
        ]);
        assert!(run(Rule::ExistsElim, &["ex", "w", "r", "c"], &e, &any_goal()).is_err());
    }

    #[test]
    fn exists_elim_rejects_wrong_assumption_shape() {
        let e = env(vec![
            assumed("ex", exists("X", pred("P", vec![name("X")]))),
            assumed("w", pred("Q", vec![name("c")])),
            ("r".to_string(), proven(atom("Q"), &["w"])),
        ]);
        assert!(run(Rule::ExistsElim, &["ex", "w", "r", "c"], &e, &any_goal()).is_err());
    }

    fn eq(a: &str, b: &str) -> Formula {
        Formula::Eq(name(a), name(b))
    }

    #[test]
    fn eq_refl_uses_goal() {
        let e = env(vec![]);
        assert!(run(Rule::EqRefl, &[], &e, &eq("a", "a")).is_ok());
        assert!(run(Rule::EqRefl, &[], &e, &eq("a", "b")).is_err());
        assert!(run(Rule::EqRefl, &[], &e, &atom("A")).is_err());
    }

    #[test]
    fn eq_sym_flips_sides() {
        let e = env(vec![assumed("h", eq("a", "b"))]);
        let result = run(Rule::EqSym, &["h"], &e, &any_goal()).unwrap();
        assert_eq!(result.formula, eq("b", "a"));
    }

    #[test]
    fn eq_trans_chains() {
        let e = env(vec![assumed("p", eq("a", "b")), assumed("q", eq("b", "c"))]);
        let result = run(Rule::EqTrans, &["p", "q"], &e, &any_goal()).unwrap();
        assert_eq!(result.formula, eq("a", "c"));
        assert!(run(Rule::EqTrans, &["q", "p"], &e, &any_goal()).is_err());
    }

    #[test]
    fn eq_subst_replaces_terms() {
        let e = env(vec![
            assumed("e", eq("a", "b")),
            assumed("p", pred("P", vec![name("a")])),
        ]);
        let goal = pred("P", vec![name("b")]);
        let result = run(Rule::EqSubst, &["e", "p"], &e, &goal).unwrap();
        assert_eq!(result.formula, goal);
        assert_eq!(result.deps, deps(&["e", "p"]));
        assert!(run(Rule::EqSubst, &["e", "p"], &e, &pred("P", vec![name("c")])).is_err());
    }
}
