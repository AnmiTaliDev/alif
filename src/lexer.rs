// SPDX-License-Identifier: GPL-3.0-only

//! Tokenizer for the Alif proof language, built with the `logos` crate.
//!
//! Call [`lex`] to convert a source string into a flat token stream. The
//! stream is consumed by [`crate::parser`].

use logos::Logos;

/// All tokens recognised by the Alif lexer.
///
/// Whitespace and line comments (`-- …`) are skipped automatically.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"--[^\n]*")]
pub enum Token {
    // Keywords — matched before `Ident` because logos tries longer / literal
    // patterns before regex patterns of equal priority.
    /// `axiom`
    #[token("axiom")]
    Axiom,
    /// `theorem`
    #[token("theorem")]
    Theorem,
    /// `proof`
    #[token("proof")]
    Proof,
    /// `assume`
    #[token("assume")]
    Assume,
    /// `have`
    #[token("have")]
    Have,
    /// `exact`
    #[token("exact")]
    Exact,
    /// `qed`
    #[token("qed")]
    Qed,
    /// `forall`
    #[token("forall")]
    Forall,
    /// `exists`
    #[token("exists")]
    Exists,

    // Logical connective keywords — uppercase, matched before `Ident`.
    /// `AND`
    #[token("AND")]
    And,
    /// `OR`
    #[token("OR")]
    Or,
    /// `NOT`
    #[token("NOT")]
    Not,

    // Multi-character punctuation — must appear before single-char variants.
    /// `|-` (turnstile / sequent separator)
    #[token("|-")]
    Turnstile,
    /// `=>` (implication arrow)
    #[token("=>")]
    Implies,
    /// `:=` (justification assignment)
    #[token(":=")]
    ColonEq,

    // Single-character punctuation.
    /// `:`
    #[token(":")]
    Colon,
    /// `,`
    #[token(",")]
    Comma,
    /// `(`
    #[token("(")]
    LParen,
    /// `)`
    #[token(")")]
    RParen,

    /// An identifier: a letter or underscore followed by zero or more
    /// alphanumerics or underscores. Matched last so keywords take priority.
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,
}

/// Lex `source` and return an owned token stream as `Vec<(Token, String)>`.
///
/// Each entry pairs a [`Token`] variant with the source slice that produced it,
/// allowing the parser to recover identifier text.
///
/// # Errors
///
/// Returns an error string containing the byte offset when an unrecognised
/// character is encountered.
pub fn lex(source: &str) -> Result<Vec<(Token, String)>, String> {
    let mut tokens = Vec::new();
    let lexer = Token::lexer(source);
    for (result, span) in lexer.spanned() {
        match result {
            Ok(tok) => tokens.push((tok, source[span].to_owned())),
            Err(()) => {
                let start = span.start;
                return Err(format!(
                    "unrecognised token at byte offset {}: {:?}",
                    start,
                    &source[span],
                ))
            }
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_kinds(src: &str) -> Vec<Token> {
        lex(src).unwrap().into_iter().map(|(t, _)| t).collect()
    }

    #[test]
    fn lex_keywords() {
        let kinds =
            token_kinds("axiom theorem proof assume have exact qed forall exists");
        assert_eq!(
            kinds,
            vec![
                Token::Axiom,
                Token::Theorem,
                Token::Proof,
                Token::Assume,
                Token::Have,
                Token::Exact,
                Token::Qed,
                Token::Forall,
                Token::Exists,
            ]
        );
    }

    #[test]
    fn lex_operators() {
        let kinds = token_kinds("|- => := : , ( ) AND OR NOT");
        assert_eq!(
            kinds,
            vec![
                Token::Turnstile,
                Token::Implies,
                Token::ColonEq,
                Token::Colon,
                Token::Comma,
                Token::LParen,
                Token::RParen,
                Token::And,
                Token::Or,
                Token::Not,
            ]
        );
    }

    #[test]
    fn lex_ident_slices() {
        let toks = lex("foo Bar _baz").unwrap();
        let slices: Vec<&str> = toks.iter().map(|(_, s)| s.as_str()).collect();
        assert_eq!(slices, vec!["foo", "Bar", "_baz"]);
    }

    #[test]
    fn lex_skips_comments() {
        let kinds = token_kinds("axiom -- this is a comment\ntheorem");
        assert_eq!(kinds, vec![Token::Axiom, Token::Theorem]);
    }

    #[test]
    fn lex_unrecognised_returns_error() {
        assert!(lex("axiom @bad").is_err());
    }

    #[test]
    fn lex_and_or_not_before_ident() {
        let kinds = token_kinds("AND OR NOT");
        assert_eq!(kinds, vec![Token::And, Token::Or, Token::Not]);
    }
}
