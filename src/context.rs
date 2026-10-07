// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;
use std::fmt;

use crate::term::Formula;

#[derive(Debug, Clone, PartialEq)]
pub struct Sequent {
    pub hypotheses: Vec<Formula>,
    pub conclusion: Formula,
}

impl fmt::Display for Sequent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, hypothesis) in self.hypotheses.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", hypothesis)?;
        }
        if !self.hypotheses.is_empty() {
            write!(f, " ")?;
        }
        write!(f, "|- {}", self.conclusion)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub axioms: HashMap<String, Formula>,
    pub theorems: HashMap<String, Sequent>,
}

impl Context {
    pub fn contains(&self, name: &str) -> bool {
        self.axioms.contains_key(name) || self.theorems.contains_key(name)
    }
}
