// SPDX-License-Identifier: GPL-3.0-only

//! Parser for the Alif proof language.
//!
//! Converts a flat token stream produced by [`crate::lexer`] into a structured
//! [`Vec<Item>`] AST. Uses hand-rolled recursive descent so there are no
//! external parser-combinator dependencies.

use crate::error::ParseError;
use crate::lexer::{lex, Token};
use crate::term::{Formula, Item, Justification, ProofStep, Theorem};

/// Internal parser state: a cursor over an owned token stream.
struct Parser {
    tokens: Vec<(Token, String)>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<(Token, String)>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn peek_slice(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|(_, s)| s.as_str())
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Consume the current token if it matches `expected`, returning its text.
    fn expect(&mut self, expected: &Token, desc: &str) -> Result<(), ParseError> {
        match self.tokens.get(self.pos) {
            Some((tok, _)) if tok == expected => {
                self.pos += 1;
                Ok(())
            }
            Some((tok, slice)) => Err(ParseError {
                message: format!("expected {}, got {:?} ({:?})", desc, tok, slice),
                offset: None,
            }),
            None => Err(ParseError {
                message: format!("expected {}, got end of input", desc),
                offset: None,
            }),
        }
    }

    /// Consume an `Ident` token and return its text.
    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.tokens.get(self.pos) {
            Some((Token::Ident, slice)) => {
                let s = slice.clone();
                self.pos += 1;
                Ok(s)
            }
            Some((tok, slice)) => Err(ParseError {
                message: format!("expected identifier, got {:?} ({:?})", tok, slice),
                offset: None,
            }),
            None => Err(ParseError {
                message: "expected identifier, got end of input".to_string(),
                offset: None,
            }),
        }
    }

    /// Parse a single top-level item (axiom or theorem).
    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.peek() {
            Some(Token::Axiom) => self.parse_axiom(),
            Some(Token::Theorem) => self.parse_theorem(),
            Some(tok) => Err(ParseError {
                message: format!("expected `axiom` or `theorem`, got {:?}", tok),
                offset: None,
            }),
            None => Err(ParseError {
                message: "unexpected end of input".to_string(),
                offset: None,
            }),
        }
    }

    /// Parse `axiom <name>: <formula>`.
    fn parse_axiom(&mut self) -> Result<Item, ParseError> {
        self.advance(); // consume `axiom`
        let name = self.expect_ident()?;
        self.expect(&Token::Colon, "`:`")?;
        let formula = self.parse_formula()?;
        Ok(Item::Axiom { name, formula })
    }

    /// Parse a theorem block including its proof steps.
    fn parse_theorem(&mut self) -> Result<Item, ParseError> {
        self.advance(); // consume `theorem`
        let name = self.expect_ident()?;
        self.expect(&Token::Colon, "`:`")?;

        let (hypotheses, conclusion) = self.parse_sequent()?;

        self.expect(&Token::Proof, "`proof`")?;
        let steps = self.parse_proof_steps()?;
        self.expect(&Token::Qed, "`qed`")?;

        Ok(Item::Theorem(Theorem { name, hypotheses, conclusion, steps }))
    }

    /// Parse `[H1, H2, ...] |- C` or just `C` when there are no hypotheses.
    fn parse_sequent(&mut self) -> Result<(Vec<Formula>, Formula), ParseError> {
        let mut formulas = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Turnstile) | Some(Token::Proof) => break,
                None => {
                    return Err(ParseError {
                        message: "unexpected end of input while parsing sequent".to_string(),
                        offset: None,
                    })
                }
                _ => {
                    formulas.push(self.parse_formula()?);
                    if matches!(self.peek(), Some(Token::Comma)) {
                        self.advance();
                    }
                }
            }
        }

        if matches!(self.peek(), Some(Token::Turnstile)) {
            self.advance(); // consume `|-`
            let conclusion = self.parse_formula()?;
            Ok((formulas, conclusion))
        } else if formulas.len() == 1 {
            Ok((vec![], formulas.remove(0)))
        } else if formulas.is_empty() {
            Err(ParseError {
                message: "empty sequent: expected at least a conclusion formula".to_string(),
                offset: None,
            })
        } else {
            Err(ParseError {
                message: "multiple formulas before `proof` but no `|-` turnstile".to_string(),
                offset: None,
            })
        }
    }

    /// Parse all proof steps until `qed`.
    fn parse_proof_steps(&mut self) -> Result<Vec<ProofStep>, ParseError> {
        let mut steps = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Qed) | None => break,
                Some(Token::Assume) => steps.push(self.parse_assume()?),
                Some(Token::Have) => steps.push(self.parse_have()?),
                Some(Token::Exact) => steps.push(self.parse_exact()?),
                Some(tok) => {
                    return Err(ParseError {
                        message: format!(
                            "expected `assume`, `have`, or `exact`, got {:?}",
                            tok
                        ),
                        offset: None,
                    })
                }
            }
        }
        Ok(steps)
    }

    /// Parse `assume <name>: <formula>`.
    fn parse_assume(&mut self) -> Result<ProofStep, ParseError> {
        self.advance(); // consume `assume`
        let name = self.expect_ident()?;
        self.expect(&Token::Colon, "`:`")?;
        let formula = self.parse_formula()?;
        Ok(ProofStep::Assume { name, formula })
    }

    /// Parse `have <name>: <formula> := <justification>`.
    fn parse_have(&mut self) -> Result<ProofStep, ParseError> {
        self.advance(); // consume `have`
        let name = self.expect_ident()?;
        self.expect(&Token::Colon, "`:`")?;
        let formula = self.parse_formula()?;
        self.expect(&Token::ColonEq, "`:=`")?;
        let justification = self.parse_justification()?;
        Ok(ProofStep::Have { name, formula, justification })
    }

    /// Parse `exact <justification>`.
    fn parse_exact(&mut self) -> Result<ProofStep, ParseError> {
        self.advance(); // consume `exact`
        let justification = self.parse_justification()?;
        Ok(ProofStep::Exact { justification })
    }

    /// Parse a justification: `<name>` (axiom/env ref) or `<Name>(<h1>, <h2>, …)` (rule).
    fn parse_justification(&mut self) -> Result<Justification, ParseError> {
        let name = self.expect_ident()?;
        if matches!(self.peek(), Some(Token::LParen)) {
            self.advance(); // consume `(`
            let mut args = Vec::new();
            loop {
                match self.peek() {
                    Some(Token::RParen) => {
                        self.advance();
                        break;
                    }
                    Some(Token::Ident) => {
                        args.push(self.expect_ident()?);
                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.advance();
                        }
                    }
                    Some(tok) => {
                        return Err(ParseError {
                            message: format!(
                                "expected argument name or `)`, got {:?}",
                                tok
                            ),
                            offset: None,
                        })
                    }
                    None => {
                        return Err(ParseError {
                            message: "unexpected end of input in justification argument list"
                                .to_string(),
                            offset: None,
                        })
                    }
                }
            }
            Ok(Justification::Rule(name, args))
        } else {
            Ok(Justification::Axiom(name))
        }
    }

    /// Parse a formula with standard operator precedence:
    ///
    /// ```text
    /// formula      = implies_expr
    /// implies_expr = or_expr  ( `=>` or_expr  )*
    /// or_expr      = and_expr ( `OR` and_expr )*
    /// and_expr     = not_expr ( `AND` not_expr )*
    /// not_expr     = `NOT` not_expr | atom
    /// atom         = `forall` ident `:` formula
    ///              | `exists` ident `:` formula
    ///              | `(` formula `)`
    ///              | ident [ `(` ident { `,` ident } `)` ]
    /// ```
    fn parse_formula(&mut self) -> Result<Formula, ParseError> {
        self.parse_implies_expr()
    }

    fn parse_implies_expr(&mut self) -> Result<Formula, ParseError> {
        let mut left = self.parse_or_expr()?;
        while matches!(self.peek(), Some(Token::Implies)) {
            self.advance();
            let right = self.parse_or_expr()?;
            left = Formula::Implies(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Formula, ParseError> {
        let mut left = self.parse_and_expr()?;
        while matches!(self.peek(), Some(Token::Or)) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Formula::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Formula, ParseError> {
        let mut left = self.parse_not_expr()?;
        while matches!(self.peek(), Some(Token::And)) {
            self.advance();
            let right = self.parse_not_expr()?;
            left = Formula::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not_expr(&mut self) -> Result<Formula, ParseError> {
        if matches!(self.peek(), Some(Token::Not)) {
            self.advance();
            let inner = self.parse_not_expr()?;
            Ok(Formula::Not(Box::new(inner)))
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Formula, ParseError> {
        match self.peek() {
            Some(Token::Forall) => {
                self.advance();
                let var = self.expect_ident()?;
                self.expect(&Token::Colon, "`:`")?;
                let body = self.parse_formula()?;
                Ok(Formula::Forall(var, Box::new(body)))
            }
            Some(Token::Exists) => {
                self.advance();
                let var = self.expect_ident()?;
                self.expect(&Token::Colon, "`:`")?;
                let body = self.parse_formula()?;
                Ok(Formula::Exists(var, Box::new(body)))
            }
            Some(Token::LParen) => {
                self.advance();
                let f = self.parse_formula()?;
                self.expect(&Token::RParen, "`)`")?;
                Ok(f)
            }
            Some(Token::Ident) => {
                let name = self.peek_slice().unwrap().to_string();
                self.advance();
                // Optional function-application syntax: `ident(arg, arg, …)`
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.advance();
                    let mut args = Vec::new();
                    loop {
                        match self.peek() {
                            Some(Token::RParen) => {
                                self.advance();
                                break;
                            }
                            Some(Token::Ident) => {
                                args.push(self.expect_ident()?);
                                if matches!(self.peek(), Some(Token::Comma)) {
                                    self.advance();
                                }
                            }
                            _ => break,
                        }
                    }
                    if args.is_empty() {
                        Ok(Formula::Var(name))
                    } else {
                        Ok(Formula::Var(format!("{}({})", name, args.join(","))))
                    }
                } else {
                    Ok(Formula::Var(name))
                }
            }
            Some(tok) => Err(ParseError {
                message: format!("expected a formula atom, got {:?}", tok),
                offset: None,
            }),
            None => Err(ParseError {
                message: "unexpected end of input while parsing formula".to_string(),
                offset: None,
            }),
        }
    }
}

/// Parse an Alif source string into a list of top-level [`Item`]s.
///
/// # Errors
///
/// Returns a [`ParseError`] on any lexer or grammar error.
pub fn parse_source(source: &str) -> Result<Vec<Item>, ParseError> {
    let tokens = lex(source).map_err(|msg| ParseError { message: msg, offset: None })?;
    let mut parser = Parser::new(tokens);
    let mut items = Vec::new();
    while parser.peek().is_some() {
        items.push(parser.parse_item()?);
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::{Formula, Item, Justification, ProofStep};

    #[test]
    fn parse_axiom_simple() {
        let items = parse_source("axiom id: A").unwrap();
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0], Item::Axiom { name, .. } if name == "id"));
    }

    #[test]
    fn parse_axiom_conjunction() {
        let items = parse_source("axiom comm: A AND B").unwrap();
        match &items[0] {
            Item::Axiom { formula, .. } => assert!(matches!(formula, Formula::And(_, _))),
            _ => panic!("expected axiom"),
        }
    }

    #[test]
    fn parse_theorem_basic() {
        let src = "theorem id_thm: A |- A\nproof\n  assume h: A\n  exact h\nqed";
        let items = parse_source(src).unwrap();
        assert_eq!(items.len(), 1);
        match &items[0] {
            Item::Theorem(thm) => {
                assert_eq!(thm.name, "id_thm");
                assert_eq!(thm.hypotheses.len(), 1);
                assert_eq!(thm.steps.len(), 2);
            }
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn parse_have_with_rule() {
        let src = "theorem t: A AND B |- A\nproof\n  assume h: A AND B\n  have a: A := AndElimLeft(h)\n  exact a\nqed";
        let items = parse_source(src).unwrap();
        match &items[0] {
            Item::Theorem(thm) => {
                assert_eq!(thm.steps.len(), 3);
                assert!(matches!(
                    &thm.steps[1],
                    ProofStep::Have {
                        justification: Justification::Rule(name, _),
                        ..
                    } if name == "AndElimLeft"
                ));
            }
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn parse_implies_formula() {
        let items = parse_source("axiom impl: A => B").unwrap();
        match &items[0] {
            Item::Axiom { formula, .. } => {
                assert!(matches!(formula, Formula::Implies(_, _)))
            }
            _ => panic!("expected axiom"),
        }
    }

    #[test]
    fn parse_forall_formula() {
        let items = parse_source("axiom fa: forall X: X").unwrap();
        match &items[0] {
            Item::Axiom { formula, .. } => {
                assert!(matches!(formula, Formula::Forall(_, _)))
            }
            _ => panic!("expected axiom"),
        }
    }

    #[test]
    fn parse_error_on_garbage() {
        assert!(parse_source("garbage @@@").is_err());
    }

    #[test]
    fn parse_multiple_hypotheses() {
        let src = "theorem t: A, B |- A AND B\nproof\n  assume ha: A\n  assume hb: B\n  have c: A AND B := AndIntro(ha, hb)\n  exact c\nqed";
        let items = parse_source(src).unwrap();
        match &items[0] {
            Item::Theorem(thm) => assert_eq!(thm.hypotheses.len(), 2),
            _ => panic!("expected theorem"),
        }
    }
}
