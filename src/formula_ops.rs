// SPDX-License-Identifier: GPL-3.0-only

use std::collections::{BTreeSet, HashMap};

use crate::term::{Formula, Term};

pub fn term_free_vars(term: &Term) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    collect_term_free(term, &[], &mut out);
    out
}

pub fn free_vars(formula: &Formula) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    collect_free(formula, &mut Vec::new(), &mut out);
    out
}

fn collect_term_free(term: &Term, bound: &[String], out: &mut BTreeSet<String>) {
    match term {
        Term::Name(name) => {
            if !bound.contains(name) {
                out.insert(name.clone());
            }
        }
        Term::App(_, args) => {
            for arg in args {
                collect_term_free(arg, bound, out);
            }
        }
    }
}

fn collect_free(formula: &Formula, bound: &mut Vec<String>, out: &mut BTreeSet<String>) {
    match formula {
        Formula::Atom(_, args) => {
            for arg in args {
                collect_term_free(arg, bound, out);
            }
        }
        Formula::Eq(a, b) => {
            collect_term_free(a, bound, out);
            collect_term_free(b, bound, out);
        }
        Formula::Bottom => {}
        Formula::And(a, b) | Formula::Or(a, b) | Formula::Implies(a, b) | Formula::Iff(a, b) => {
            collect_free(a, bound, out);
            collect_free(b, bound, out);
        }
        Formula::Not(a) => collect_free(a, bound, out),
        Formula::Forall(x, body) | Formula::Exists(x, body) => {
            bound.push(x.clone());
            collect_free(body, bound, out);
            bound.pop();
        }
    }
}

fn collect_term_names(term: &Term, out: &mut BTreeSet<String>) {
    match term {
        Term::Name(name) => {
            out.insert(name.clone());
        }
        Term::App(_, args) => {
            for arg in args {
                collect_term_names(arg, out);
            }
        }
    }
}

fn collect_names(formula: &Formula, out: &mut BTreeSet<String>) {
    match formula {
        Formula::Atom(_, args) => {
            for arg in args {
                collect_term_names(arg, out);
            }
        }
        Formula::Eq(a, b) => {
            collect_term_names(a, out);
            collect_term_names(b, out);
        }
        Formula::Bottom => {}
        Formula::And(a, b) | Formula::Or(a, b) | Formula::Implies(a, b) | Formula::Iff(a, b) => {
            collect_names(a, out);
            collect_names(b, out);
        }
        Formula::Not(a) => collect_names(a, out),
        Formula::Forall(x, body) | Formula::Exists(x, body) => {
            out.insert(x.clone());
            collect_names(body, out);
        }
    }
}

fn fresh_name(base: &str, avoid: &BTreeSet<String>) -> String {
    let mut n = 1;
    loop {
        let candidate = format!("{}_{}", base, n);
        if !avoid.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

fn substitute_term(term: &Term, var: &str, replacement: &Term) -> Term {
    match term {
        Term::Name(name) if name == var => replacement.clone(),
        Term::Name(_) => term.clone(),
        Term::App(f, args) => Term::App(
            f.clone(),
            args.iter()
                .map(|a| substitute_term(a, var, replacement))
                .collect(),
        ),
    }
}

fn substitute_binder(
    bound: &str,
    body: &Formula,
    var: &str,
    replacement: &Term,
    build: fn(String, Box<Formula>) -> Formula,
) -> Formula {
    if bound == var {
        return build(bound.to_string(), Box::new(body.clone()));
    }
    let captures = term_free_vars(replacement).contains(bound) && free_vars(body).contains(var);
    if !captures {
        return build(
            bound.to_string(),
            Box::new(substitute(body, var, replacement)),
        );
    }
    let mut avoid = BTreeSet::new();
    collect_names(body, &mut avoid);
    collect_term_names(replacement, &mut avoid);
    avoid.insert(var.to_string());
    avoid.insert(bound.to_string());
    let fresh = fresh_name(bound, &avoid);
    let renamed = substitute(body, bound, &Term::Name(fresh.clone()));
    build(fresh, Box::new(substitute(&renamed, var, replacement)))
}

pub fn substitute(formula: &Formula, var: &str, replacement: &Term) -> Formula {
    match formula {
        Formula::Atom(name, args) => Formula::Atom(
            name.clone(),
            args.iter()
                .map(|a| substitute_term(a, var, replacement))
                .collect(),
        ),
        Formula::Eq(a, b) => Formula::Eq(
            substitute_term(a, var, replacement),
            substitute_term(b, var, replacement),
        ),
        Formula::Bottom => Formula::Bottom,
        Formula::And(a, b) => Formula::And(
            Box::new(substitute(a, var, replacement)),
            Box::new(substitute(b, var, replacement)),
        ),
        Formula::Or(a, b) => Formula::Or(
            Box::new(substitute(a, var, replacement)),
            Box::new(substitute(b, var, replacement)),
        ),
        Formula::Implies(a, b) => Formula::Implies(
            Box::new(substitute(a, var, replacement)),
            Box::new(substitute(b, var, replacement)),
        ),
        Formula::Iff(a, b) => Formula::Iff(
            Box::new(substitute(a, var, replacement)),
            Box::new(substitute(b, var, replacement)),
        ),
        Formula::Not(a) => Formula::Not(Box::new(substitute(a, var, replacement))),
        Formula::Forall(x, body) => {
            substitute_binder(x, body, var, replacement, Formula::Forall)
        }
        Formula::Exists(x, body) => {
            substitute_binder(x, body, var, replacement, Formula::Exists)
        }
    }
}

struct Matcher {
    scope: Vec<(String, String)>,
    bindings: Option<HashMap<String, Formula>>,
}

impl Matcher {
    fn term(&self, pattern: &Term, target: &Term) -> bool {
        match (pattern, target) {
            (Term::Name(p), Term::Name(t)) => {
                match self.scope.iter().rev().find(|(x, y)| x == p || y == t) {
                    Some((x, y)) => x == p && y == t,
                    None => p == t,
                }
            }
            (Term::App(f, xs), Term::App(g, ys)) => f == g && self.terms(xs, ys),
            _ => false,
        }
    }

    fn terms(&self, patterns: &[Term], targets: &[Term]) -> bool {
        patterns.len() == targets.len()
            && patterns
                .iter()
                .zip(targets)
                .all(|(p, t)| self.term(p, t))
    }

    fn bind(&mut self, name: &str, target: &Formula) -> bool {
        let captured = free_vars(target)
            .iter()
            .any(|v| self.scope.iter().any(|(_, y)| y == v));
        if captured {
            return false;
        }
        let Some(bindings) = self.bindings.as_mut() else {
            return false;
        };
        match bindings.get(name) {
            Some(previous) => alpha_eq(previous, target),
            None => {
                bindings.insert(name.to_string(), target.clone());
                true
            }
        }
    }

    fn formula(&mut self, pattern: &Formula, target: &Formula) -> bool {
        match (pattern, target) {
            (Formula::Atom(name, args), _) if args.is_empty() && self.bindings.is_some() => {
                self.bind(name, target)
            }
            (Formula::Atom(n, xs), Formula::Atom(m, ys)) => n == m && self.terms(xs, ys),
            (Formula::Eq(a1, a2), Formula::Eq(b1, b2)) => self.term(a1, b1) && self.term(a2, b2),
            (Formula::Bottom, Formula::Bottom) => true,
            (Formula::And(a1, a2), Formula::And(b1, b2))
            | (Formula::Or(a1, a2), Formula::Or(b1, b2))
            | (Formula::Implies(a1, a2), Formula::Implies(b1, b2))
            | (Formula::Iff(a1, a2), Formula::Iff(b1, b2)) => {
                self.formula(a1, b1) && self.formula(a2, b2)
            }
            (Formula::Not(a), Formula::Not(b)) => self.formula(a, b),
            (Formula::Forall(x, p), Formula::Forall(y, q))
            | (Formula::Exists(x, p), Formula::Exists(y, q)) => {
                self.scope.push((x.clone(), y.clone()));
                let result = self.formula(p, q);
                self.scope.pop();
                result
            }
            _ => false,
        }
    }
}

pub fn alpha_eq(a: &Formula, b: &Formula) -> bool {
    Matcher {
        scope: Vec::new(),
        bindings: None,
    }
    .formula(a, b)
}

pub fn matches_all(pairs: &[(&Formula, &Formula)]) -> bool {
    let mut matcher = Matcher {
        scope: Vec::new(),
        bindings: Some(HashMap::new()),
    };
    pairs.iter().all(|(pattern, target)| matcher.formula(pattern, target))
}

fn term_replaces(source: &Term, target: &Term, from: &Term, to: &Term) -> bool {
    if source == target {
        return true;
    }
    if source == from && target == to {
        return true;
    }
    match (source, target) {
        (Term::App(f, xs), Term::App(g, ys)) => {
            f == g
                && xs.len() == ys.len()
                && xs
                    .iter()
                    .zip(ys)
                    .all(|(x, y)| term_replaces(x, y, from, to))
        }
        _ => false,
    }
}

pub fn replaces(source: &Formula, target: &Formula, from: &Term, to: &Term) -> bool {
    if source == target {
        return true;
    }
    match (source, target) {
        (Formula::Atom(n, xs), Formula::Atom(m, ys)) => {
            n == m
                && xs.len() == ys.len()
                && xs
                    .iter()
                    .zip(ys)
                    .all(|(x, y)| term_replaces(x, y, from, to))
        }
        (Formula::Eq(a1, a2), Formula::Eq(b1, b2)) => {
            term_replaces(a1, b1, from, to) && term_replaces(a2, b2, from, to)
        }
        (Formula::And(a1, a2), Formula::And(b1, b2))
        | (Formula::Or(a1, a2), Formula::Or(b1, b2))
        | (Formula::Implies(a1, a2), Formula::Implies(b1, b2))
        | (Formula::Iff(a1, a2), Formula::Iff(b1, b2)) => {
            replaces(a1, b1, from, to) && replaces(a2, b2, from, to)
        }
        (Formula::Not(a), Formula::Not(b)) => replaces(a, b, from, to),
        (Formula::Forall(x, p), Formula::Forall(y, q))
        | (Formula::Exists(x, p), Formula::Exists(y, q)) => {
            x == y
                && !term_free_vars(from).contains(x)
                && !term_free_vars(to).contains(x)
                && replaces(p, q, from, to)
        }
        _ => false,
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

    fn forall(x: &str, body: Formula) -> Formula {
        Formula::Forall(x.to_string(), Box::new(body))
    }

    fn exists(x: &str, body: Formula) -> Formula {
        Formula::Exists(x.to_string(), Box::new(body))
    }

    fn and(a: Formula, b: Formula) -> Formula {
        Formula::And(Box::new(a), Box::new(b))
    }

    fn implies(a: Formula, b: Formula) -> Formula {
        Formula::Implies(Box::new(a), Box::new(b))
    }

    #[test]
    fn free_vars_excludes_bound() {
        let f = forall("X", and(pred("P", vec![name("X")]), pred("Q", vec![name("Y")])));
        let vars: Vec<String> = free_vars(&f).into_iter().collect();
        assert_eq!(vars, vec!["Y".to_string()]);
    }

    #[test]
    fn free_vars_ignores_propositional_atoms() {
        assert!(free_vars(&atom("A")).is_empty());
    }

    #[test]
    fn free_vars_in_equality_and_functions() {
        let f = Formula::Eq(
            Term::App("f".to_string(), vec![name("a")]),
            name("b"),
        );
        let vars: Vec<String> = free_vars(&f).into_iter().collect();
        assert_eq!(vars, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn substitute_replaces_free_occurrence() {
        let f = pred("P", vec![name("X")]);
        assert_eq!(substitute(&f, "X", &name("a")), pred("P", vec![name("a")]));
    }

    #[test]
    fn substitute_replaces_inside_function_term() {
        let f = pred("P", vec![Term::App("f".to_string(), vec![name("X")])]);
        let expected = pred("P", vec![Term::App("f".to_string(), vec![name("a")])]);
        assert_eq!(substitute(&f, "X", &name("a")), expected);
    }

    #[test]
    fn substitute_skips_bound_variable() {
        let f = forall("X", pred("P", vec![name("X")]));
        assert_eq!(substitute(&f, "X", &name("a")), f);
    }

    #[test]
    fn substitute_does_not_touch_propositional_atom_with_same_name() {
        assert_eq!(substitute(&atom("X"), "X", &name("a")), atom("X"));
    }

    #[test]
    fn substitute_avoids_capture() {
        let f = forall("Y", pred("R", vec![name("X"), name("Y")]));
        let result = substitute(&f, "X", &name("Y"));
        let expected = forall("Y_1", pred("R", vec![name("Y"), name("Y_1")]));
        assert_eq!(result, expected);
    }

    #[test]
    fn substitute_renames_only_when_needed() {
        let f = forall("Y", pred("R", vec![name("X"), name("Y")]));
        let result = substitute(&f, "X", &name("a"));
        assert_eq!(result, forall("Y", pred("R", vec![name("a"), name("Y")])));
    }

    #[test]
    fn substitute_fresh_name_avoids_existing_names() {
        let f = forall(
            "Y",
            and(
                pred("R", vec![name("X"), name("Y")]),
                pred("S", vec![name("Y_1")]),
            ),
        );
        let result = substitute(&f, "X", &name("Y"));
        let expected = forall(
            "Y_2",
            and(
                pred("R", vec![name("Y"), name("Y_2")]),
                pred("S", vec![name("Y_1")]),
            ),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn alpha_eq_renamed_binder() {
        let a = forall("X", pred("P", vec![name("X")]));
        let b = forall("Y", pred("P", vec![name("Y")]));
        assert!(alpha_eq(&a, &b));
    }

    #[test]
    fn alpha_eq_distinguishes_free_variables() {
        let a = pred("P", vec![name("X")]);
        let b = pred("P", vec![name("Y")]);
        assert!(!alpha_eq(&a, &b));
    }

    #[test]
    fn alpha_eq_rejects_capture_confusion() {
        let a = forall("X", pred("R", vec![name("X"), name("Y")]));
        let b = forall("Y", pred("R", vec![name("Y"), name("Y")]));
        assert!(!alpha_eq(&a, &b));
    }

    #[test]
    fn alpha_eq_distinguishes_quantifiers() {
        let a = forall("X", pred("P", vec![name("X")]));
        let b = exists("X", pred("P", vec![name("X")]));
        assert!(!alpha_eq(&a, &b));
    }

    #[test]
    fn alpha_eq_distinguishes_connectives() {
        assert!(!alpha_eq(
            &and(atom("A"), atom("B")),
            &Formula::Or(Box::new(atom("A")), Box::new(atom("B")))
        ));
    }

    #[test]
    fn matches_binds_propositional_atoms() {
        let pattern = implies(atom("P"), atom("Q"));
        let target = implies(atom("A"), and(atom("B"), atom("C")));
        assert!(matches_all(&[(&pattern, &target)]));
    }

    #[test]
    fn matches_requires_consistent_bindings() {
        let pattern = and(atom("P"), atom("P"));
        assert!(matches_all(&[(&pattern, &and(atom("A"), atom("A")))]));
        assert!(!matches_all(&[(&pattern, &and(atom("A"), atom("B")))]));
    }

    #[test]
    fn matches_shares_bindings_across_pairs() {
        let p1 = atom("P");
        let p2 = implies(atom("P"), atom("Q"));
        let t1 = atom("A");
        let t2 = implies(atom("A"), atom("B"));
        assert!(matches_all(&[(&p1, &t1), (&p2, &t2)]));
        let t3 = implies(atom("C"), atom("B"));
        assert!(!matches_all(&[(&p1, &t1), (&p2, &t3)]));
    }

    #[test]
    fn matches_does_not_bind_predicates_with_arguments() {
        let pattern = pred("P", vec![name("a")]);
        assert!(matches_all(&[(&pattern, &pred("P", vec![name("a")]))]));
        assert!(!matches_all(&[(&pattern, &pred("Q", vec![name("a")]))]));
    }

    #[test]
    fn matches_rejects_capture_of_free_variable() {
        let pattern = forall("X", atom("P"));
        let target = forall("X", pred("Q", vec![name("X")]));
        assert!(!matches_all(&[(&pattern, &target)]));
    }

    #[test]
    fn matches_allows_binding_without_capture() {
        let pattern = forall("X", atom("P"));
        let target = forall("X", pred("Q", vec![name("Y")]));
        assert!(matches_all(&[(&pattern, &target)]));
    }

    #[test]
    fn matches_accepts_renamed_binder() {
        let pattern = forall("X", pred("P", vec![name("X")]));
        let target = forall("Y", pred("P", vec![name("Y")]));
        assert!(matches_all(&[(&pattern, &target)]));
    }

    #[test]
    fn replaces_identical_formulas() {
        let f = pred("P", vec![name("a")]);
        assert!(replaces(&f, &f, &name("a"), &name("b")));
    }

    #[test]
    fn replaces_single_occurrence() {
        let source = and(pred("P", vec![name("a")]), pred("Q", vec![name("a")]));
        let target = and(pred("P", vec![name("b")]), pred("Q", vec![name("a")]));
        assert!(replaces(&source, &target, &name("a"), &name("b")));
    }

    #[test]
    fn replaces_rejects_other_change() {
        let source = pred("P", vec![name("a")]);
        let target = pred("P", vec![name("c")]);
        assert!(!replaces(&source, &target, &name("a"), &name("b")));
    }

    #[test]
    fn replaces_rejects_reverse_direction() {
        let source = pred("P", vec![name("b")]);
        let target = pred("P", vec![name("a")]);
        assert!(!replaces(&source, &target, &name("a"), &name("b")));
    }

    #[test]
    fn replaces_inside_function_term() {
        let source = pred("P", vec![Term::App("f".to_string(), vec![name("a")])]);
        let target = pred("P", vec![Term::App("f".to_string(), vec![name("b")])]);
        assert!(replaces(&source, &target, &name("a"), &name("b")));
    }

    #[test]
    fn replaces_rejects_capture_under_binder() {
        let source = forall("X", pred("P", vec![name("a"), name("X")]));
        let target = forall("X", pred("P", vec![name("X"), name("X")]));
        assert!(!replaces(&source, &target, &name("a"), &name("X")));
    }

    #[test]
    fn replaces_under_binder_without_capture() {
        let source = forall("X", pred("P", vec![name("a"), name("X")]));
        let target = forall("X", pred("P", vec![name("b"), name("X")]));
        assert!(replaces(&source, &target, &name("a"), &name("b")));
    }
}
