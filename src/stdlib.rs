// SPDX-License-Identifier: GPL-3.0-only

use crate::context::Context;
use crate::error::AlifError;
use crate::verify::Verifier;

const STDLIB_SOURCE: &str = include_str!("../stdlib/logic.alif");

pub fn load_stdlib() -> Result<Context, AlifError> {
    let mut verifier = Verifier::with_context(Context::default());
    verifier.process(STDLIB_SOURCE, Some("<stdlib>"), None)?;
    Ok(verifier.into_context())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdlib_loads_and_verifies() {
        let ctx = load_stdlib().expect("stdlib must verify");
        for name in [
            "identity",
            "and_comm",
            "and_assoc",
            "or_comm",
            "ex_falso",
            "not_not_intro",
            "modus_tollens",
            "contraposition",
            "iff_comm",
        ] {
            assert!(ctx.theorems.contains_key(name), "missing `{}`", name);
        }
    }

    #[test]
    fn stdlib_declares_no_axioms() {
        assert!(load_stdlib().unwrap().axioms.is_empty());
    }
}
