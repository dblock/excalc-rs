//! Recursive-descent parser using conventional calculator precedence
//! (loosest to tightest binding):
//!
//! 1. Additive:        `+  -`
//! 2. Multiplicative:  `*  /  mod`
//! 3. Unary prefix:    `-x`
//! 4. Power / root:    `^` (right-assoc), `\` (n-th root, left-assoc)
//! 5. Postfix:         `!` (factorial), `%` (percent)
//! 6. Primary:         numbers, variables, `name(args, ...)`, `( expr )`
//!
//! This intentionally differs from the original Pascal engine, which bound
//! `%` tighter than `* /` and `\` looser than `^`. That was a reasonable
//! design for a button-driven 90s calculator UI, but it surprises anyone
//! typing an expression today, so v1 normalizes to conventional precedence.

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::error::{CalcError, CalcResult};
use crate::lexer::{tokenize, Token};

pub fn parse(input: &str) -> CalcResult<Expr> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_additive()?;
    parser.expect_eof()?;
    Ok(expr)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &Token, label: &str) -> CalcResult<()> {
        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(CalcError::ExpectedToken {
                expected: label.to_string(),
                position: self.pos,
            })
        }
    }

    fn expect_eof(&mut self) -> CalcResult<()> {
        if matches!(self.peek(), Token::Eof) {
            Ok(())
        } else {
            Err(CalcError::ExpectedToken {
                expected: "end of expression".to_string(),
                position: self.pos,
            })
        }
    }

    fn parse_additive(&mut self) -> CalcResult<Expr> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_multiplicative()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> CalcResult<Expr> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Ident(name) if name.eq_ignore_ascii_case("mod") => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_unary()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> CalcResult<Expr> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(operand)))
            }
            Token::Plus => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_power(),
        }
    }

    fn parse_power(&mut self) -> CalcResult<Expr> {
        let mut base = self.parse_postfix()?;
        loop {
            match self.peek() {
                Token::Caret => {
                    self.advance();
                    // Right-associative: 2^3^2 == 2^(3^2).
                    let exponent = self.parse_unary()?;
                    base = Expr::Binary(BinaryOp::Pow, Box::new(base), Box::new(exponent));
                }
                Token::Backslash => {
                    self.advance();
                    let radicand = self.parse_unary()?;
                    base = Expr::Binary(BinaryOp::Root, Box::new(base), Box::new(radicand));
                }
                _ => break,
            }
        }
        Ok(base)
    }

    fn parse_postfix(&mut self) -> CalcResult<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::Bang => {
                    self.advance();
                    expr = Expr::Unary(UnaryOp::Factorial, Box::new(expr));
                }
                Token::Percent => {
                    self.advance();
                    expr = Expr::Unary(UnaryOp::Percent, Box::new(expr));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> CalcResult<Expr> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Token::LParen => {
                self.advance();
                let inner = self.parse_additive()?;
                self.expect(&Token::RParen, ")")?;
                Ok(inner)
            }
            Token::Ident(name) => {
                self.advance();
                if matches!(self.peek(), Token::LParen) {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen, ")")?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Token::Minus => {
                // Allow nested unary via primary -> unary re-entry, e.g. (-x)
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(operand)))
            }
            _ => Err(CalcError::ExpectedToken {
                expected: "number, variable, or function call".to_string(),
                position: self.pos,
            }),
        }
    }

    fn parse_args(&mut self) -> CalcResult<Vec<Expr>> {
        let mut args = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(args);
        }
        args.push(self.parse_additive()?);
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            args.push(self.parse_additive()?);
        }
        Ok(args)
    }
}
