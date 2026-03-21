// SPDX-License-Identifier: GPL-3.0-only

//! Core AST data types for the Alif proof verifier.

/// A logical formula in propositional or first-order logic.
#[derive(Debug, Clone, PartialEq)]
pub enum Formula {
    /// A propositional variable or atom, e.g. `A`, `human(X)`.
    Var(String),
    /// Conjunction: `A AND B`.
    And(Box<Formula>, Box<Formula>),
    /// Disjunction: `A OR B`.
    Or(Box<Formula>, Box<Formula>),
    /// Negation: `NOT A`.
    Not(Box<Formula>),
    /// Implication: `A => B`.
    Implies(Box<Formula>, Box<Formula>),
    /// Universal quantification: `forall X: F`.
    Forall(String, Box<Formula>),
    /// Existential quantification: `exists X: F`.
    Exists(String, Box<Formula>),
}

impl std::fmt::Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formula::Var(name) => write!(f, "{}", name),
            Formula::And(a, b) => write!(f, "({} AND {})", a, b),
            Formula::Or(a, b) => write!(f, "({} OR {})", a, b),
            Formula::Not(a) => write!(f, "NOT {}", a),
            Formula::Implies(a, b) => write!(f, "({} => {})", a, b),
            Formula::Forall(x, body) => write!(f, "forall {}: {}", x, body),
            Formula::Exists(x, body) => write!(f, "exists {}: {}", x, body),
        }
    }
}

/// A justification for a proof step — either an axiom reference or a rule application.
#[derive(Debug, Clone, PartialEq)]
pub enum Justification {
    /// Reference to a named axiom, e.g. `identity`.
    Axiom(String),
    /// Application of a named rule to a list of hypothesis names, e.g. `AndIntro(h1, h2)`.
    Rule(String, Vec<String>),
}

/// A single step in a proof.
#[derive(Debug, Clone, PartialEq)]
pub enum ProofStep {
    /// Introduce a hypothesis into scope.
    Assume { name: String, formula: Formula },
    /// Derive a new formula from existing hypotheses.
    Have {
        name: String,
        formula: Formula,
        justification: Justification,
    },
    /// Conclude the proof by citing the final result.
    Exact { justification: Justification },
}

/// A theorem statement together with its proof steps.
#[derive(Debug, Clone, PartialEq)]
pub struct Theorem {
    /// The theorem's name.
    pub name: String,
    /// The hypotheses (antecedents) of the theorem.
    pub hypotheses: Vec<Formula>,
    /// The formula to be proved.
    pub conclusion: Formula,
    /// The ordered list of proof steps.
    pub steps: Vec<ProofStep>,
}

/// A top-level item in an Alif source file.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// An axiom declaration.
    Axiom { name: String, formula: Formula },
    /// A theorem with proof.
    Theorem(Theorem),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_display_var() {
        let f = Formula::Var("A".to_string());
        assert_eq!(f.to_string(), "A");
    }

    #[test]
    fn formula_display_and() {
        let f = Formula::And(
            Box::new(Formula::Var("A".to_string())),
            Box::new(Formula::Var("B".to_string())),
        );
        assert_eq!(f.to_string(), "(A AND B)");
    }

    #[test]
    fn formula_display_not() {
        let f = Formula::Not(Box::new(Formula::Var("A".to_string())));
        assert_eq!(f.to_string(), "NOT A");
    }

    #[test]
    fn formula_display_forall() {
        let f = Formula::Forall("X".to_string(), Box::new(Formula::Var("P".to_string())));
        assert_eq!(f.to_string(), "forall X: P");
    }
}
