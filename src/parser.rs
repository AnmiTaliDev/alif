// SPDX-License-Identifier: GPL-3.0-only

use crate::error::ParseError;
use crate::lexer::{lex, Lexeme, Token};
use crate::term::{Formula, Item, Justification, ProofStep, Term, Theorem};

struct Parser {
    lexemes: Vec<Lexeme>,
    pos: usize,
    end: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.lexemes.get(self.pos).map(|l| &l.token)
    }

    fn offset(&self) -> usize {
        self.lexemes.get(self.pos).map_or(self.end, |l| l.offset)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn error(&self, message: String) -> ParseError {
        ParseError {
            message,
            offset: Some(self.offset()),
            location: None,
        }
    }

    fn found(&self) -> String {
        match self.lexemes.get(self.pos) {
            Some(l) => format!("`{}`", l.text),
            None => "end of input".to_string(),
        }
    }

    fn expect(&mut self, expected: &Token, description: &str) -> Result<(), ParseError> {
        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("expected {}, found {}", description, self.found())))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.lexemes.get(self.pos) {
            Some(l) if l.token == Token::Ident => {
                let text = l.text.clone();
                self.advance();
                Ok(text)
            }
            _ => Err(self.error(format!("expected identifier, found {}", self.found()))),
        }
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.peek() {
            Some(Token::Axiom) => self.parse_axiom(),
            Some(Token::Theorem) => self.parse_theorem(),
            Some(Token::Import) => self.parse_import(),
            _ => Err(self.error(format!(
                "expected `axiom`, `theorem` or `import`, found {}",
                self.found()
            ))),
        }
    }

    fn parse_axiom(&mut self) -> Result<Item, ParseError> {
        let offset = self.offset();
        self.advance();
        let name = self.expect_ident()?;
        self.expect(&Token::Colon, "`:`")?;
        let formula = self.parse_formula()?;
        Ok(Item::Axiom {
            name,
            formula,
            offset,
        })
    }

    fn parse_import(&mut self) -> Result<Item, ParseError> {
        let offset = self.offset();
        self.advance();
        match self.lexemes.get(self.pos) {
            Some(l) if l.token == Token::Str => {
                let path = l.text[1..l.text.len() - 1].to_string();
                self.advance();
                Ok(Item::Import { path, offset })
            }
            _ => Err(self.error(format!(
                "expected a quoted path, found {}",
                self.found()
            ))),
        }
    }

    fn parse_theorem(&mut self) -> Result<Item, ParseError> {
        let offset = self.offset();
        self.advance();
        let name = self.expect_ident()?;
        self.expect(&Token::Colon, "`:`")?;
        let (hypotheses, conclusion) = self.parse_sequent()?;
        self.expect(&Token::Proof, "`proof`")?;
        let steps = self.parse_steps()?;
        self.expect(&Token::Qed, "`qed`")?;
        Ok(Item::Theorem(Theorem {
            name,
            hypotheses,
            conclusion,
            steps,
            offset,
        }))
    }

    fn parse_sequent(&mut self) -> Result<(Vec<Formula>, Formula), ParseError> {
        let mut formulas = Vec::new();
        if self.peek() != Some(&Token::Turnstile) {
            formulas.push(self.parse_formula()?);
            while self.peek() == Some(&Token::Comma) {
                self.advance();
                formulas.push(self.parse_formula()?);
            }
        }
        if self.peek() == Some(&Token::Turnstile) {
            self.advance();
            let conclusion = self.parse_formula()?;
            Ok((formulas, conclusion))
        } else if formulas.len() == 1 {
            Ok((Vec::new(), formulas.remove(0)))
        } else {
            Err(self.error(format!("expected `|-`, found {}", self.found())))
        }
    }

    fn parse_steps(&mut self) -> Result<Vec<ProofStep>, ParseError> {
        let mut steps = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Assume) => steps.push(self.parse_assume()?),
                Some(Token::Have) => steps.push(self.parse_have()?),
                Some(Token::Exact) => steps.push(self.parse_exact()?),
                _ => return Ok(steps),
            }
        }
    }

    fn parse_assume(&mut self) -> Result<ProofStep, ParseError> {
        let offset = self.offset();
        self.advance();
        let name = self.expect_ident()?;
        self.expect(&Token::Colon, "`:`")?;
        let formula = self.parse_formula()?;
        Ok(ProofStep::Assume {
            name,
            formula,
            offset,
        })
    }

    fn parse_have(&mut self) -> Result<ProofStep, ParseError> {
        let offset = self.offset();
        self.advance();
        let name = self.expect_ident()?;
        self.expect(&Token::Colon, "`:`")?;
        let formula = self.parse_formula()?;
        self.expect(&Token::ColonEq, "`:=`")?;
        let justification = self.parse_justification()?;
        Ok(ProofStep::Have {
            name,
            formula,
            justification,
            offset,
        })
    }

    fn parse_exact(&mut self) -> Result<ProofStep, ParseError> {
        let offset = self.offset();
        self.advance();
        let justification = self.parse_justification()?;
        Ok(ProofStep::Exact {
            justification,
            offset,
        })
    }

    fn parse_justification(&mut self) -> Result<Justification, ParseError> {
        let name = self.expect_ident()?;
        if self.peek() == Some(&Token::LParen) {
            self.advance();
            let args = self.parse_term_list(true)?;
            Ok(Justification::Rule(name, args))
        } else {
            Ok(Justification::Ref(name))
        }
    }

    fn parse_term_list(&mut self, allow_empty: bool) -> Result<Vec<Term>, ParseError> {
        let mut terms = Vec::new();
        if allow_empty && self.peek() == Some(&Token::RParen) {
            self.advance();
            return Ok(terms);
        }
        loop {
            terms.push(self.parse_term()?);
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                self.expect(&Token::RParen, "`,` or `)`")?;
                return Ok(terms);
            }
        }
    }

    fn parse_term(&mut self) -> Result<Term, ParseError> {
        let name = self.expect_ident()?;
        if self.peek() == Some(&Token::LParen) {
            self.advance();
            let args = self.parse_term_list(false)?;
            Ok(Term::App(name, args))
        } else {
            Ok(Term::Name(name))
        }
    }

    fn parse_formula(&mut self) -> Result<Formula, ParseError> {
        self.parse_iff()
    }

    fn parse_iff(&mut self) -> Result<Formula, ParseError> {
        let mut left = self.parse_implies()?;
        while self.peek() == Some(&Token::Iff) {
            self.advance();
            let right = self.parse_implies()?;
            left = Formula::Iff(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_implies(&mut self) -> Result<Formula, ParseError> {
        let left = self.parse_or()?;
        if self.peek() == Some(&Token::Implies) {
            self.advance();
            let right = self.parse_implies()?;
            Ok(Formula::Implies(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_or(&mut self) -> Result<Formula, ParseError> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Formula::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Formula, ParseError> {
        let mut left = self.parse_not()?;
        while self.peek() == Some(&Token::And) {
            self.advance();
            let right = self.parse_not()?;
            left = Formula::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Formula, ParseError> {
        if self.peek() == Some(&Token::Not) {
            self.advance();
            let inner = self.parse_not()?;
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
                let formula = self.parse_formula()?;
                self.expect(&Token::RParen, "`)`")?;
                Ok(formula)
            }
            Some(Token::False) => {
                self.advance();
                Ok(Formula::Bottom)
            }
            Some(Token::Ident) => {
                let term = self.parse_term()?;
                if self.peek() == Some(&Token::Equals) {
                    self.advance();
                    let right = self.parse_term()?;
                    return Ok(Formula::Eq(term, right));
                }
                match term {
                    Term::Name(name) => Ok(Formula::Atom(name, Vec::new())),
                    Term::App(name, args) => Ok(Formula::Atom(name, args)),
                }
            }
            _ => Err(self.error(format!("expected a formula, found {}", self.found()))),
        }
    }
}

pub fn parse_source(source: &str) -> Result<Vec<Item>, ParseError> {
    let lexemes = lex(source)?;
    let mut parser = Parser {
        lexemes,
        pos: 0,
        end: source.len(),
    };
    let mut items = Vec::new();
    while parser.peek().is_some() {
        items.push(parser.parse_item()?);
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(name: &str) -> Formula {
        Formula::Atom(name.to_string(), vec![])
    }

    fn parse_formula_str(text: &str) -> Formula {
        let items = parse_source(&format!("axiom f: {}", text)).unwrap();
        match items.into_iter().next().unwrap() {
            Item::Axiom { formula, .. } => formula,
            _ => panic!("expected axiom"),
        }
    }

    #[test]
    fn axiom_simple() {
        let items = parse_source("axiom id: A").unwrap();
        assert!(matches!(&items[0], Item::Axiom { name, .. } if name == "id"));
    }

    #[test]
    fn axiom_records_offset() {
        let items = parse_source("\n\naxiom id: A").unwrap();
        assert!(matches!(&items[0], Item::Axiom { offset: 2, .. }));
    }

    #[test]
    fn theorem_basic() {
        let src = "theorem id_thm: A |- A\nproof\n  assume h: A\n  exact h\nqed";
        let items = parse_source(src).unwrap();
        match &items[0] {
            Item::Theorem(thm) => {
                assert_eq!(thm.name, "id_thm");
                assert_eq!(thm.hypotheses, vec![atom("A")]);
                assert_eq!(thm.conclusion, atom("A"));
                assert_eq!(thm.steps.len(), 2);
            }
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn theorem_without_hypotheses() {
        let items = parse_source("theorem t: |- A\nproof\n  exact x\nqed").unwrap();
        match &items[0] {
            Item::Theorem(thm) => assert!(thm.hypotheses.is_empty()),
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn theorem_without_turnstile() {
        let items = parse_source("theorem t: A\nproof\n  exact x\nqed").unwrap();
        match &items[0] {
            Item::Theorem(thm) => {
                assert!(thm.hypotheses.is_empty());
                assert_eq!(thm.conclusion, atom("A"));
            }
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn theorem_multiple_hypotheses() {
        let src = "theorem t: A, B |- A AND B\nproof\n  exact x\nqed";
        match &parse_source(src).unwrap()[0] {
            Item::Theorem(thm) => assert_eq!(thm.hypotheses.len(), 2),
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn multiple_formulas_without_turnstile_is_error() {
        assert!(parse_source("theorem t: A, B\nproof\n  exact x\nqed").is_err());
    }

    #[test]
    fn have_with_rule() {
        let src = "theorem t: A AND B |- A\nproof\n  assume h: A AND B\n  have a: A := AndElimLeft(h)\n  exact a\nqed";
        match &parse_source(src).unwrap()[0] {
            Item::Theorem(thm) => assert!(matches!(
                &thm.steps[1],
                ProofStep::Have {
                    justification: Justification::Rule(name, args),
                    ..
                } if name == "AndElimLeft" && args == &vec![Term::Name("h".to_string())]
            )),
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn rule_with_term_arguments() {
        let src = "theorem t: A |- A\nproof\n  have x: A := ForallElim(h, f(a, b))\n  exact x\nqed";
        match &parse_source(src).unwrap()[0] {
            Item::Theorem(thm) => match &thm.steps[0] {
                ProofStep::Have {
                    justification: Justification::Rule(_, args),
                    ..
                } => assert_eq!(
                    args[1],
                    Term::App(
                        "f".to_string(),
                        vec![Term::Name("a".to_string()), Term::Name("b".to_string())]
                    )
                ),
                _ => panic!("expected have"),
            },
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn rule_with_empty_arguments() {
        let src = "theorem t: a = a\nproof\n  exact EqRefl()\nqed";
        match &parse_source(src).unwrap()[0] {
            Item::Theorem(thm) => assert!(matches!(
                &thm.steps[0],
                ProofStep::Exact {
                    justification: Justification::Rule(_, args),
                    ..
                } if args.is_empty()
            )),
            _ => panic!("expected theorem"),
        }
    }

    #[test]
    fn import_item() {
        let items = parse_source("import \"lib/a.alif\"").unwrap();
        assert!(matches!(&items[0], Item::Import { path, .. } if path == "lib/a.alif"));
    }

    #[test]
    fn import_requires_string() {
        assert!(parse_source("import foo").is_err());
    }

    #[test]
    fn precedence_and_binds_tighter_than_or() {
        assert_eq!(
            parse_formula_str("A OR B AND C"),
            Formula::Or(
                Box::new(atom("A")),
                Box::new(Formula::And(Box::new(atom("B")), Box::new(atom("C"))))
            )
        );
    }

    #[test]
    fn precedence_not_binds_tighter_than_and() {
        assert_eq!(
            parse_formula_str("NOT A AND B"),
            Formula::And(
                Box::new(Formula::Not(Box::new(atom("A")))),
                Box::new(atom("B"))
            )
        );
    }

    #[test]
    fn implication_is_right_associative() {
        assert_eq!(
            parse_formula_str("A => B => C"),
            Formula::Implies(
                Box::new(atom("A")),
                Box::new(Formula::Implies(Box::new(atom("B")), Box::new(atom("C"))))
            )
        );
    }

    #[test]
    fn iff_binds_looser_than_implies() {
        assert_eq!(
            parse_formula_str("A => B <=> C"),
            Formula::Iff(
                Box::new(Formula::Implies(Box::new(atom("A")), Box::new(atom("B")))),
                Box::new(atom("C"))
            )
        );
    }

    #[test]
    fn parentheses_override_precedence() {
        assert_eq!(
            parse_formula_str("(A OR B) AND C"),
            Formula::And(
                Box::new(Formula::Or(Box::new(atom("A")), Box::new(atom("B")))),
                Box::new(atom("C"))
            )
        );
    }

    #[test]
    fn predicate_arguments_are_terms() {
        assert_eq!(
            parse_formula_str("human(socrates)"),
            Formula::Atom("human".to_string(), vec![Term::Name("socrates".to_string())])
        );
    }

    #[test]
    fn nested_function_terms() {
        assert_eq!(
            parse_formula_str("P(f(a))"),
            Formula::Atom(
                "P".to_string(),
                vec![Term::App(
                    "f".to_string(),
                    vec![Term::Name("a".to_string())]
                )]
            )
        );
    }

    #[test]
    fn equality_formula() {
        assert_eq!(
            parse_formula_str("f(a) = b"),
            Formula::Eq(
                Term::App("f".to_string(), vec![Term::Name("a".to_string())]),
                Term::Name("b".to_string())
            )
        );
    }

    #[test]
    fn bottom_formula() {
        assert_eq!(parse_formula_str("FALSE"), Formula::Bottom);
    }

    #[test]
    fn quantifier_body_extends_right() {
        assert_eq!(
            parse_formula_str("forall X: P(X) AND Q(X)"),
            Formula::Forall(
                "X".to_string(),
                Box::new(Formula::And(
                    Box::new(Formula::Atom(
                        "P".to_string(),
                        vec![Term::Name("X".to_string())]
                    )),
                    Box::new(Formula::Atom(
                        "Q".to_string(),
                        vec![Term::Name("X".to_string())]
                    ))
                ))
            )
        );
    }

    #[test]
    fn unclosed_argument_list_is_error() {
        assert!(parse_source("axiom f: P(a").is_err());
    }

    #[test]
    fn empty_predicate_arguments_are_error() {
        assert!(parse_source("axiom f: P()").is_err());
    }

    #[test]
    fn missing_qed_is_error() {
        let err = parse_source("theorem t: A |- A\nproof\n  assume h: A\n  exact h").unwrap_err();
        assert!(err.message.contains("`qed`"));
    }

    #[test]
    fn garbage_is_error() {
        assert!(parse_source("garbage @@@").is_err());
    }

    #[test]
    fn error_offset_points_at_offending_token() {
        let err = parse_source("axiom a: A\naxiom : B").unwrap_err();
        assert_eq!(err.offset, Some("axiom a: A\naxiom ".len()));
    }

    #[test]
    fn error_at_end_of_input_points_at_end() {
        let src = "axiom a:";
        let err = parse_source(src).unwrap_err();
        assert_eq!(err.offset, Some(src.len()));
    }
}
