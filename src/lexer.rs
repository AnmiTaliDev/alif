// SPDX-License-Identifier: GPL-3.0-only

use logos::Logos;

use crate::error::ParseError;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"--[^\n]*")]
pub enum Token {
    #[token("axiom")]
    Axiom,
    #[token("theorem")]
    Theorem,
    #[token("import")]
    Import,
    #[token("proof")]
    Proof,
    #[token("assume")]
    Assume,
    #[token("have")]
    Have,
    #[token("exact")]
    Exact,
    #[token("qed")]
    Qed,
    #[token("forall")]
    Forall,
    #[token("exists")]
    Exists,

    #[token("AND")]
    And,
    #[token("OR")]
    Or,
    #[token("NOT")]
    Not,
    #[token("FALSE")]
    False,

    #[token("|-")]
    Turnstile,
    #[token("=>")]
    Implies,
    #[token("<=>")]
    Iff,
    #[token(":=")]
    ColonEq,
    #[token("=")]
    Equals,

    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[regex(r#""[^"\n]*""#)]
    Str,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lexeme {
    pub token: Token,
    pub text: String,
    pub offset: usize,
}

pub fn lex(source: &str) -> Result<Vec<Lexeme>, ParseError> {
    let mut lexemes = Vec::new();
    for (result, span) in Token::lexer(source).spanned() {
        match result {
            Ok(token) => lexemes.push(Lexeme {
                token,
                text: source[span.clone()].to_owned(),
                offset: span.start,
            }),
            Err(()) => {
                return Err(ParseError {
                    message: format!("unrecognised token {:?}", &source[span.clone()]),
                    offset: Some(span.start),
                    location: None,
                })
            }
        }
    }
    Ok(lexemes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<Token> {
        lex(src).unwrap().into_iter().map(|l| l.token).collect()
    }

    #[test]
    fn keywords() {
        assert_eq!(
            kinds("axiom theorem import proof assume have exact qed forall exists"),
            vec![
                Token::Axiom,
                Token::Theorem,
                Token::Import,
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
    fn operators() {
        assert_eq!(
            kinds("|- => <=> := = : , ( ) AND OR NOT FALSE"),
            vec![
                Token::Turnstile,
                Token::Implies,
                Token::Iff,
                Token::ColonEq,
                Token::Equals,
                Token::Colon,
                Token::Comma,
                Token::LParen,
                Token::RParen,
                Token::And,
                Token::Or,
                Token::Not,
                Token::False,
            ]
        );
    }

    #[test]
    fn identifier_slices() {
        let lexemes = lex("foo Bar _baz").unwrap();
        let texts: Vec<&str> = lexemes.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, vec!["foo", "Bar", "_baz"]);
    }

    #[test]
    fn keyword_prefix_is_identifier() {
        assert_eq!(kinds("ANDx FALSEY proofs"), vec![Token::Ident; 3]);
    }

    #[test]
    fn skips_comments() {
        assert_eq!(kinds("axiom -- comment\ntheorem"), vec![Token::Axiom, Token::Theorem]);
    }

    #[test]
    fn string_literal() {
        let lexemes = lex("import \"a/b.alif\"").unwrap();
        assert_eq!(lexemes[1].token, Token::Str);
        assert_eq!(lexemes[1].text, "\"a/b.alif\"");
    }

    #[test]
    fn offsets_are_byte_positions() {
        let lexemes = lex("ab  cd").unwrap();
        assert_eq!(lexemes[1].offset, 4);
    }

    #[test]
    fn unrecognised_character_reports_offset() {
        let err = lex("axiom @bad").unwrap_err();
        assert_eq!(err.offset, Some(6));
    }
}
