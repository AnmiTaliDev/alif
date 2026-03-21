// SPDX-License-Identifier: GPL-3.0-only

//! Standard library of logical axioms, embedded from `stdlib/logic.alif`.

use std::collections::HashMap;

use crate::error::ParseError;
use crate::parser::parse_source;
use crate::term::{Formula, Item};

/// The raw source text of the standard axiom set.
const STDLIB_SOURCE: &str = include_str!("../stdlib/logic.alif");

/// Parse the embedded standard library and return a map of axiom names to formulas.
///
/// # Errors
///
/// Returns a [`ParseError`] if the embedded source is malformed — which indicates
/// a build-time error in `stdlib/logic.alif`.
pub fn load_stdlib() -> Result<HashMap<String, Formula>, ParseError> {
    let items = parse_source(STDLIB_SOURCE)?;
    let mut axioms = HashMap::new();
    for item in items {
        if let Item::Axiom { name, formula } = item {
            axioms.insert(name, formula);
        }
    }
    Ok(axioms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdlib_loads_without_error() {
        let axioms = load_stdlib().expect("stdlib should parse cleanly");
        assert!(axioms.contains_key("identity"), "expected `identity` axiom");
        assert!(axioms.contains_key("and_comm"), "expected `and_comm` axiom");
        assert!(axioms.contains_key("or_comm"), "expected `or_comm` axiom");
        assert!(axioms.contains_key("ex_falso"), "expected `ex_falso` axiom");
    }

    #[test]
    fn identity_axiom_is_var() {
        let axioms = load_stdlib().unwrap();
        // identity axiom is defined as `axiom identity: A` — a simple Var
        assert!(matches!(axioms["identity"], crate::term::Formula::Var(_)));
    }
}
