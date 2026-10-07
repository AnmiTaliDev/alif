// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Name(String),
    App(String, Vec<Term>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Formula {
    Atom(String, Vec<Term>),
    Eq(Term, Term),
    Bottom,
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Not(Box<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    Iff(Box<Formula>, Box<Formula>),
    Forall(String, Box<Formula>),
    Exists(String, Box<Formula>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Justification {
    Ref(String),
    Rule(String, Vec<Term>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProofStep {
    Assume {
        name: String,
        formula: Formula,
        offset: usize,
    },
    Have {
        name: String,
        formula: Formula,
        justification: Justification,
        offset: usize,
    },
    Exact {
        justification: Justification,
        offset: usize,
    },
}

impl ProofStep {
    pub fn offset(&self) -> usize {
        match self {
            ProofStep::Assume { offset, .. }
            | ProofStep::Have { offset, .. }
            | ProofStep::Exact { offset, .. } => *offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Theorem {
    pub name: String,
    pub hypotheses: Vec<Formula>,
    pub conclusion: Formula,
    pub steps: Vec<ProofStep>,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Axiom {
        name: String,
        formula: Formula,
        offset: usize,
    },
    Theorem(Theorem),
    Import {
        path: String,
        offset: usize,
    },
}

fn write_terms(f: &mut fmt::Formatter<'_>, terms: &[Term]) -> fmt::Result {
    for (i, term) in terms.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}", term)?;
    }
    Ok(())
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Name(name) => write!(f, "{}", name),
            Term::App(name, args) => {
                write!(f, "{}(", name)?;
                write_terms(f, args)?;
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Formula::Atom(name, args) if args.is_empty() => write!(f, "{}", name),
            Formula::Atom(name, args) => {
                write!(f, "{}(", name)?;
                write_terms(f, args)?;
                write!(f, ")")
            }
            Formula::Eq(a, b) => write!(f, "{} = {}", a, b),
            Formula::Bottom => write!(f, "FALSE"),
            Formula::And(a, b) => write!(f, "({} AND {})", a, b),
            Formula::Or(a, b) => write!(f, "({} OR {})", a, b),
            Formula::Not(a) => write!(f, "NOT {}", a),
            Formula::Implies(a, b) => write!(f, "({} => {})", a, b),
            Formula::Iff(a, b) => write!(f, "({} <=> {})", a, b),
            Formula::Forall(x, body) => write!(f, "forall {}: {}", x, body),
            Formula::Exists(x, body) => write!(f, "exists {}: {}", x, body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(name: &str) -> Formula {
        Formula::Atom(name.to_string(), vec![])
    }

    #[test]
    fn display_atom() {
        assert_eq!(atom("A").to_string(), "A");
    }

    #[test]
    fn display_predicate() {
        let f = Formula::Atom(
            "human".to_string(),
            vec![Term::Name("socrates".to_string())],
        );
        assert_eq!(f.to_string(), "human(socrates)");
    }

    #[test]
    fn display_nested_term() {
        let t = Term::App(
            "f".to_string(),
            vec![
                Term::Name("a".to_string()),
                Term::App("g".to_string(), vec![Term::Name("b".to_string())]),
            ],
        );
        assert_eq!(t.to_string(), "f(a, g(b))");
    }

    #[test]
    fn display_and() {
        let f = Formula::And(Box::new(atom("A")), Box::new(atom("B")));
        assert_eq!(f.to_string(), "(A AND B)");
    }

    #[test]
    fn display_not() {
        let f = Formula::Not(Box::new(atom("A")));
        assert_eq!(f.to_string(), "NOT A");
    }

    #[test]
    fn display_forall() {
        let f = Formula::Forall("X".to_string(), Box::new(atom("P")));
        assert_eq!(f.to_string(), "forall X: P");
    }

    #[test]
    fn display_bottom_and_eq() {
        assert_eq!(Formula::Bottom.to_string(), "FALSE");
        let f = Formula::Eq(Term::Name("a".to_string()), Term::Name("b".to_string()));
        assert_eq!(f.to_string(), "a = b");
    }

    #[test]
    fn display_iff() {
        let f = Formula::Iff(Box::new(atom("A")), Box::new(atom("B")));
        assert_eq!(f.to_string(), "(A <=> B)");
    }
}
