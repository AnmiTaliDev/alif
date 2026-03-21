// SPDX-License-Identifier: GPL-3.0-only

//! Inference rules for the Alif proof system.
//!
//! Each rule is a pure function that takes a list of premise [`Formula`]s and
//! returns the derived [`Formula`] or a [`RuleError`] explaining why the rule
//! could not be applied.

use crate::error::RuleError;
use crate::term::Formula;

/// All inference rules supported by the Alif proof system.
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    /// `A`, `B` ⊢ `A AND B`
    AndIntro,
    /// `A AND B` ⊢ `A`
    AndElimLeft,
    /// `A AND B` ⊢ `B`
    AndElimRight,
    /// `A` ⊢ `A OR B` (left injection)
    OrIntroLeft,
    /// `B` ⊢ `A OR B` (right injection)
    OrIntroRight,
    /// `A => B`, `A` ⊢ `B` (modus ponens)
    ModusPonens,
    /// Derive `A => B` from a sub-proof that derives `B` assuming `A`.
    ImpliesIntro,
    /// `A`, `NOT A` ⊢ `B` (ex falso / explosion)
    NotElim,
    /// From a proof that `A` leads to contradiction, derive `NOT A`.
    NotIntro,
    /// `forall X: F`, `t` ⊢ `F[X := t]`
    ForallElim,
    /// Derive `forall X: F` when `F` holds for an arbitrary variable.
    ForallIntro,
    /// `F[X := t]` ⊢ `exists X: F`
    ExistsIntro,
}

impl Rule {
    /// Parse a rule name string into a [`Rule`] variant.
    ///
    /// Returns `None` if the name does not correspond to any known rule.
    pub fn from_name(name: &str) -> Option<Rule> {
        match name {
            "AndIntro" => Some(Rule::AndIntro),
            "AndElimLeft" => Some(Rule::AndElimLeft),
            "AndElimRight" => Some(Rule::AndElimRight),
            "OrIntroLeft" => Some(Rule::OrIntroLeft),
            "OrIntroRight" => Some(Rule::OrIntroRight),
            "ModusPonens" => Some(Rule::ModusPonens),
            "ImpliesIntro" => Some(Rule::ImpliesIntro),
            "NotElim" => Some(Rule::NotElim),
            "NotIntro" => Some(Rule::NotIntro),
            "ForallElim" => Some(Rule::ForallElim),
            "ForallIntro" => Some(Rule::ForallIntro),
            "ExistsIntro" => Some(Rule::ExistsIntro),
            _ => None,
        }
    }

    /// Return the canonical name of this rule.
    pub fn name(&self) -> &'static str {
        match self {
            Rule::AndIntro => "AndIntro",
            Rule::AndElimLeft => "AndElimLeft",
            Rule::AndElimRight => "AndElimRight",
            Rule::OrIntroLeft => "OrIntroLeft",
            Rule::OrIntroRight => "OrIntroRight",
            Rule::ModusPonens => "ModusPonens",
            Rule::ImpliesIntro => "ImpliesIntro",
            Rule::NotElim => "NotElim",
            Rule::NotIntro => "NotIntro",
            Rule::ForallElim => "ForallElim",
            Rule::ForallIntro => "ForallIntro",
            Rule::ExistsIntro => "ExistsIntro",
        }
    }
}

/// Apply `rule` to `premises` and return the derived formula.
///
/// # Errors
///
/// Returns a [`RuleError`] if the premises do not match the rule's requirements.
pub fn apply_rule(rule: &Rule, premises: &[Formula]) -> Result<Formula, RuleError> {
    let rule_name = rule.name().to_string();
    let err = |reason: &str| -> RuleError {
        RuleError { rule: rule_name.clone(), reason: reason.to_string() }
    };

    match rule {
        Rule::AndIntro => {
            if premises.len() != 2 {
                return Err(err("AndIntro requires exactly 2 premises"));
            }
            Ok(Formula::And(
                Box::new(premises[0].clone()),
                Box::new(premises[1].clone()),
            ))
        }

        Rule::AndElimLeft => {
            if premises.len() != 1 {
                return Err(err("AndElimLeft requires exactly 1 premise"));
            }
            match &premises[0] {
                Formula::And(a, _) => Ok(*a.clone()),
                _ => Err(err("premise must be a conjunction (A AND B)")),
            }
        }

        Rule::AndElimRight => {
            if premises.len() != 1 {
                return Err(err("AndElimRight requires exactly 1 premise"));
            }
            match &premises[0] {
                Formula::And(_, b) => Ok(*b.clone()),
                _ => Err(err("premise must be a conjunction (A AND B)")),
            }
        }

        Rule::OrIntroLeft => {
            if premises.len() != 2 {
                return Err(err("OrIntroLeft requires 2 arguments: the left formula and the right formula"));
            }
            Ok(Formula::Or(
                Box::new(premises[0].clone()),
                Box::new(premises[1].clone()),
            ))
        }

        Rule::OrIntroRight => {
            if premises.len() != 2 {
                return Err(err("OrIntroRight requires 2 arguments: the left formula and the right formula"));
            }
            Ok(Formula::Or(
                Box::new(premises[1].clone()),
                Box::new(premises[0].clone()),
            ))
        }

        Rule::ModusPonens => {
            if premises.len() != 2 {
                return Err(err("ModusPonens requires exactly 2 premises: (A => B) and A"));
            }
            match &premises[0] {
                Formula::Implies(ant, cons) => {
                    if **ant == premises[1] {
                        Ok(*cons.clone())
                    } else {
                        Err(err("second premise does not match the antecedent of the implication"))
                    }
                }
                _ => match &premises[1] {
                    Formula::Implies(ant, cons) => {
                        if **ant == premises[0] {
                            Ok(*cons.clone())
                        } else {
                            Err(err("first premise does not match the antecedent of the implication"))
                        }
                    }
                    _ => Err(err("one premise must be an implication (A => B)")),
                },
            }
        }

        Rule::ImpliesIntro => {
            if premises.len() != 2 {
                return Err(err("ImpliesIntro requires 2 premises: the hypothesis A and the conclusion B"));
            }
            Ok(Formula::Implies(
                Box::new(premises[0].clone()),
                Box::new(premises[1].clone()),
            ))
        }

        Rule::NotElim => {
            if premises.len() != 2 {
                return Err(err("NotElim requires exactly 2 premises: A and NOT A (or vice versa)"));
            }
            let contradiction = match (&premises[0], &premises[1]) {
                (f, Formula::Not(g)) if f == g.as_ref() => true,
                (Formula::Not(f), g) if f.as_ref() == g => true,
                _ => false,
            };
            if contradiction {
                Ok(Formula::Var("\u{22a5}".to_string()))
            } else {
                Err(err("premises must be a formula and its negation"))
            }
        }

        Rule::NotIntro => {
            if premises.len() != 1 {
                return Err(err("NotIntro requires 1 premise: the formula A to be negated"));
            }
            Ok(Formula::Not(Box::new(premises[0].clone())))
        }

        Rule::ForallElim => {
            if premises.len() != 2 {
                return Err(err("ForallElim requires 2 premises: (forall X: F) and the term to substitute"));
            }
            match &premises[0] {
                Formula::Forall(var, body) => {
                    let term = &premises[1];
                    Ok(substitute(body, var, term))
                }
                _ => Err(err("first premise must be a universal quantification (forall X: F)")),
            }
        }

        Rule::ForallIntro => {
            if premises.len() != 2 {
                return Err(err("ForallIntro requires 2 premises: the variable name (as Var) and the formula"));
            }
            match &premises[0] {
                Formula::Var(var) => Ok(Formula::Forall(var.clone(), Box::new(premises[1].clone()))),
                _ => Err(err("first premise must be a variable name")),
            }
        }

        Rule::ExistsIntro => {
            if premises.len() != 3 {
                return Err(err("ExistsIntro requires 3 premises: the variable, the witness term, and the formula F[X:=t]"));
            }
            match &premises[0] {
                Formula::Var(var) => Ok(Formula::Exists(var.clone(), Box::new(premises[2].clone()))),
                _ => Err(err("first premise must be a variable name")),
            }
        }
    }
}

/// Substitute all free occurrences of `var` with `term` inside `formula`.
pub fn substitute(formula: &Formula, var: &str, term: &Formula) -> Formula {
    match formula {
        Formula::Var(name) => {
            if name == var {
                term.clone()
            } else {
                formula.clone()
            }
        }
        Formula::And(a, b) => Formula::And(
            Box::new(substitute(a, var, term)),
            Box::new(substitute(b, var, term)),
        ),
        Formula::Or(a, b) => Formula::Or(
            Box::new(substitute(a, var, term)),
            Box::new(substitute(b, var, term)),
        ),
        Formula::Not(a) => Formula::Not(Box::new(substitute(a, var, term))),
        Formula::Implies(a, b) => Formula::Implies(
            Box::new(substitute(a, var, term)),
            Box::new(substitute(b, var, term)),
        ),
        Formula::Forall(x, body) => {
            if x == var {
                formula.clone()
            } else {
                Formula::Forall(x.clone(), Box::new(substitute(body, var, term)))
            }
        }
        Formula::Exists(x, body) => {
            if x == var {
                formula.clone()
            } else {
                Formula::Exists(x.clone(), Box::new(substitute(body, var, term)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn var(s: &str) -> Formula { Formula::Var(s.to_string()) }
    fn and(a: Formula, b: Formula) -> Formula { Formula::And(Box::new(a), Box::new(b)) }
    fn not(a: Formula) -> Formula { Formula::Not(Box::new(a)) }
    fn implies(a: Formula, b: Formula) -> Formula { Formula::Implies(Box::new(a), Box::new(b)) }
    fn forall(x: &str, f: Formula) -> Formula { Formula::Forall(x.to_string(), Box::new(f)) }

    #[test]
    fn and_intro() {
        let result = apply_rule(&Rule::AndIntro, &[var("A"), var("B")]).unwrap();
        assert_eq!(result, and(var("A"), var("B")));
    }

    #[test]
    fn and_elim_left() {
        let conj = and(var("A"), var("B"));
        let result = apply_rule(&Rule::AndElimLeft, &[conj]).unwrap();
        assert_eq!(result, var("A"));
    }

    #[test]
    fn and_elim_right() {
        let conj = and(var("A"), var("B"));
        let result = apply_rule(&Rule::AndElimRight, &[conj]).unwrap();
        assert_eq!(result, var("B"));
    }

    #[test]
    fn modus_ponens() {
        let imp = implies(var("A"), var("B"));
        let result = apply_rule(&Rule::ModusPonens, &[imp, var("A")]).unwrap();
        assert_eq!(result, var("B"));
    }

    #[test]
    fn modus_ponens_wrong_antecedent() {
        let imp = implies(var("A"), var("B"));
        assert!(apply_rule(&Rule::ModusPonens, &[imp, var("C")]).is_err());
    }

    #[test]
    fn not_elim_produces_bottom() {
        let result = apply_rule(&Rule::NotElim, &[var("A"), not(var("A"))]).unwrap();
        assert_eq!(result, var("\u{22a5}"));
    }

    #[test]
    fn forall_elim_substitutes() {
        let forall_f = forall("X", var("X"));
        let result = apply_rule(&Rule::ForallElim, &[forall_f, var("socrates")]).unwrap();
        assert_eq!(result, var("socrates"));
    }

    #[test]
    fn substitute_replaces_free_var() {
        let f = Formula::And(Box::new(var("X")), Box::new(var("Y")));
        let result = substitute(&f, "X", &var("a"));
        assert_eq!(result, and(var("a"), var("Y")));
    }

    #[test]
    fn substitute_does_not_replace_bound_var() {
        let f = forall("X", var("X"));
        let result = substitute(&f, "X", &var("a"));
        assert_eq!(result, f);
    }
}
