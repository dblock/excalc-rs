//! Recursive-descent parser using conventional calculator precedence
//! (loosest to tightest binding):
//!
//! 1. Comparison:      `=  >  <`
//! 2. Logical or:      `or  nor  xor  xnor`
//! 3. Logical and:     `and  nand  &`
//! 4. Additive:        `+  -`
//! 5. Multiplicative:  `*  /  mod`
//! 6. Unary prefix:    `-x`
//! 7. Power / root:    `^` (right-assoc), `\` (n-th root, left-assoc)
//! 8. Postfix:         `!` (factorial), `%` (percent)
//! 9. Primary:         numbers, variables, `name(args, ...)`, `( expr )`
//!
//! This intentionally differs from the original Pascal engine, which bound
//! `%` tighter than `* /` and `\` looser than `^`, and used `=` to mean
//! variable assignment rather than equality. That was a reasonable design
//! for a button-driven 90s calculator UI, but it surprises anyone typing an
//! expression today, so v1 normalizes to conventional precedence and gives
//! `=` the more intuitive equality meaning.

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::error::{CalcError, CalcResult};
use crate::lexer::{tokenize, Token};

pub fn parse(input: &str) -> CalcResult<Expr> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_comparison()?;
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

    fn parse_comparison(&mut self) -> CalcResult<Expr> {
        let mut lhs = self.parse_logical_or()?;
        loop {
            let op = match self.peek() {
                Token::Eq => BinaryOp::Eq,
                Token::Gt => BinaryOp::Gt,
                Token::Lt => BinaryOp::Lt,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_logical_or()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_logical_or(&mut self) -> CalcResult<Expr> {
        let mut lhs = self.parse_logical_and()?;
        loop {
            let op = match self.peek() {
                Token::Ident(name) if name.eq_ignore_ascii_case("xnor") => BinaryOp::Xnor,
                Token::Ident(name) if name.eq_ignore_ascii_case("xor") => BinaryOp::Xor,
                Token::Ident(name) if name.eq_ignore_ascii_case("nor") => BinaryOp::Nor,
                Token::Ident(name) if name.eq_ignore_ascii_case("or") => BinaryOp::Or,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_logical_and()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_logical_and(&mut self) -> CalcResult<Expr> {
        let mut lhs = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::Amp => BinaryOp::And,
                Token::Ident(name) if name.eq_ignore_ascii_case("nand") => BinaryOp::Nand,
                Token::Ident(name) if name.eq_ignore_ascii_case("and") => BinaryOp::And,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_additive()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
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
                let inner = self.parse_comparison()?;
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
        args.push(self.parse_comparison()?);
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            args.push(self.parse_comparison()?);
        }
        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_closing_paren_errors() {
        assert_eq!(
            parse("(2 + 3"),
            Err(CalcError::ExpectedToken {
                expected: ")".to_string(),
                position: 4,
            })
        );
        assert_eq!(
            parse("sqrt(4"),
            Err(CalcError::ExpectedToken {
                expected: ")".to_string(),
                position: 3,
            })
        );
    }

    #[test]
    fn trailing_tokens_error_at_eof() {
        assert_eq!(
            parse("2 3"),
            Err(CalcError::ExpectedToken {
                expected: "end of expression".to_string(),
                position: 1,
            })
        );
    }

    #[test]
    fn unary_plus_is_a_no_op() {
        assert_eq!(parse("+5").unwrap(), Expr::Number(5.0));
    }

    #[test]
    fn primary_fallthrough_error() {
        assert_eq!(
            parse("*5"),
            Err(CalcError::ExpectedToken {
                expected: "number, variable, or function call".to_string(),
                position: 0,
            })
        );
    }

    #[test]
    fn empty_call_args() {
        assert_eq!(
            parse("sum()").unwrap(),
            Expr::Call("sum".to_string(), vec![])
        );
    }

    #[test]
    fn comparison_operators_parse() {
        assert_eq!(
            parse("3 > 2").unwrap(),
            Expr::Binary(
                BinaryOp::Gt,
                Box::new(Expr::Number(3.0)),
                Box::new(Expr::Number(2.0))
            )
        );
        assert_eq!(
            parse("3 < 2").unwrap(),
            Expr::Binary(
                BinaryOp::Lt,
                Box::new(Expr::Number(3.0)),
                Box::new(Expr::Number(2.0))
            )
        );
        assert_eq!(
            parse("3 = 3").unwrap(),
            Expr::Binary(
                BinaryOp::Eq,
                Box::new(Expr::Number(3.0)),
                Box::new(Expr::Number(3.0))
            )
        );
    }

    #[test]
    fn logical_operators_parse_with_keywords_and_ampersand() {
        assert_eq!(
            parse("2 xor 4").unwrap(),
            Expr::Binary(
                BinaryOp::Xor,
                Box::new(Expr::Number(2.0)),
                Box::new(Expr::Number(4.0))
            )
        );
        assert_eq!(
            parse("2 xnor 4").unwrap(),
            Expr::Binary(
                BinaryOp::Xnor,
                Box::new(Expr::Number(2.0)),
                Box::new(Expr::Number(4.0))
            )
        );
        assert_eq!(
            parse("2 nor 4").unwrap(),
            Expr::Binary(
                BinaryOp::Nor,
                Box::new(Expr::Number(2.0)),
                Box::new(Expr::Number(4.0))
            )
        );
        assert_eq!(
            parse("3 or 9").unwrap(),
            Expr::Binary(
                BinaryOp::Or,
                Box::new(Expr::Number(3.0)),
                Box::new(Expr::Number(9.0))
            )
        );
        assert_eq!(
            parse("3 nand 9").unwrap(),
            Expr::Binary(
                BinaryOp::Nand,
                Box::new(Expr::Number(3.0)),
                Box::new(Expr::Number(9.0))
            )
        );
        assert_eq!(parse("3 and 9").unwrap(), parse("3 & 9").unwrap(),);
        assert_eq!(
            parse("3 & 9").unwrap(),
            Expr::Binary(
                BinaryOp::And,
                Box::new(Expr::Number(3.0)),
                Box::new(Expr::Number(9.0))
            )
        );
    }

    #[test]
    fn logical_and_binds_tighter_than_or_and_comparison() {
        // `1 or 2 and 4` should parse as `1 or (2 and 4)`, and the whole
        // thing loosest under a comparison: `1 or 2 and 4 = 0` should parse
        // as `(1 or (2 and 4)) = 0`.
        assert_eq!(
            parse("1 or 2 and 4").unwrap(),
            Expr::Binary(
                BinaryOp::Or,
                Box::new(Expr::Number(1.0)),
                Box::new(Expr::Binary(
                    BinaryOp::And,
                    Box::new(Expr::Number(2.0)),
                    Box::new(Expr::Number(4.0))
                ))
            )
        );
        assert_eq!(
            parse("1 or 2 and 4 = 0").unwrap(),
            Expr::Binary(
                BinaryOp::Eq,
                Box::new(Expr::Binary(
                    BinaryOp::Or,
                    Box::new(Expr::Number(1.0)),
                    Box::new(Expr::Binary(
                        BinaryOp::And,
                        Box::new(Expr::Number(2.0)),
                        Box::new(Expr::Number(4.0))
                    ))
                )),
                Box::new(Expr::Number(0.0))
            )
        );
    }

    #[test]
    fn logical_and_binds_looser_than_arithmetic() {
        // `1 and 2 + 3` should parse as `1 and (2 + 3)`.
        assert_eq!(
            parse("1 and 2 + 3").unwrap(),
            Expr::Binary(
                BinaryOp::And,
                Box::new(Expr::Number(1.0)),
                Box::new(Expr::Binary(
                    BinaryOp::Add,
                    Box::new(Expr::Number(2.0)),
                    Box::new(Expr::Number(3.0))
                ))
            )
        );
    }

    #[test]
    fn call_args_accept_comparison_and_logical_expressions() {
        assert_eq!(
            parse("sum(3 > 2, 1 and 1)").unwrap(),
            Expr::Call(
                "sum".to_string(),
                vec![
                    Expr::Binary(
                        BinaryOp::Gt,
                        Box::new(Expr::Number(3.0)),
                        Box::new(Expr::Number(2.0))
                    ),
                    Expr::Binary(
                        BinaryOp::And,
                        Box::new(Expr::Number(1.0)),
                        Box::new(Expr::Number(1.0))
                    ),
                ]
            )
        );
    }
}
