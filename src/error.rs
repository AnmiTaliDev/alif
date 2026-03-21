// SPDX-License-Identifier: GPL-3.0-only

//! Unified error types for the Alif proof verifier.

use std::fmt;
use crate::term::ProofStep;

/// An error produced during proof checking.
#[derive(Debug)]
pub struct CheckError {
    /// Zero-based index of the failing step.
    pub step_index: usize,
    /// A copy of the failing step. Boxed to keep the error type small.
    pub step: Box<ProofStep>,
    /// Human-readable explanation of the failure.
    pub message: String,
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "proof error at step {}: {}",
            self.step_index, self.message
        )
    }
}

impl std::error::Error for CheckError {}

/// An error produced during parsing.
#[derive(Debug)]
pub struct ParseError {
    /// Human-readable description of the parse failure.
    pub message: String,
    /// The byte offset in the source where parsing failed, if known.
    pub offset: Option<usize>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(off) => write!(f, "parse error at offset {}: {}", off, self.message),
            None => write!(f, "parse error: {}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}

/// An error produced when applying an inference rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleError {
    /// The rule that was invoked.
    pub rule: String,
    /// Why the rule could not be applied.
    pub reason: String,
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rule `{}` failed: {}", self.rule, self.reason)
    }
}

impl std::error::Error for RuleError {}

/// Top-level error type returned by the public `verify` API.
#[derive(Debug)]
pub enum AlifError {
    /// Source could not be parsed.
    Parse(ParseError),
    /// A theorem's proof is invalid.
    Check(CheckError),
}

impl fmt::Display for AlifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlifError::Parse(e) => write!(f, "{}", e),
            AlifError::Check(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AlifError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AlifError::Parse(e) => Some(e),
            AlifError::Check(e) => Some(e),
        }
    }
}

impl From<ParseError> for AlifError {
    fn from(e: ParseError) -> Self {
        AlifError::Parse(e)
    }
}

impl From<CheckError> for AlifError {
    fn from(e: CheckError) -> Self {
        AlifError::Check(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::Justification;

    fn dummy_step() -> ProofStep {
        ProofStep::Exact {
            justification: Justification::Axiom("test".to_string()),
        }
    }

    #[test]
    fn check_error_display() {
        let e = CheckError {
            step_index: 2,
            step: Box::new(dummy_step()),
            message: "hypothesis not found".to_string(),
        };
        assert!(e.to_string().contains("step 2"));
        assert!(e.to_string().contains("hypothesis not found"));
    }

    #[test]
    fn parse_error_display_with_offset() {
        let e = ParseError { message: "unexpected token".to_string(), offset: Some(10) };
        assert!(e.to_string().contains("offset 10"));
    }

    #[test]
    fn rule_error_display() {
        let e = RuleError { rule: "AndIntro".to_string(), reason: "wrong arity".to_string() };
        assert!(e.to_string().contains("AndIntro"));
    }
}
