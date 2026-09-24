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
//! `=` the more intuitive equality meaning; assignment instead uses `:=`
//! (see [`Stmt::Assign`]) at the statement level, outside expression
//! grammar entirely, so it's never ambiguous with equality.
//!
//! Above expressions sits one more layer: a full input is a `;`- or
//! newline-separated sequence of statements (`parse_program`), each either
//! `name := expr` or a plain expression; the value of the last statement is
//! the program's result.

use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::error::{CalcError, CalcResult};
use crate::lexer::{tokenize, Token};

pub fn parse(input: &str) -> CalcResult<Expr> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_comparison()?;
    parser.expect_eof()?;
    Ok(expr)
}

/// Parses a full program: a `;`/newline-separated sequence of statements.
pub fn parse_program(input: &str) -> CalcResult<Program> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, pos: 0 };
    let stmts = parser.parse_statements()?;
    parser.expect_eof()?;
    Ok(stmts)
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

    fn skip_semis(&mut self) {
        while matches!(self.peek(), Token::Semi) {
            self.advance();
        }
    }

    /// Parses a `;`/newline-separated sequence of statements, tolerating
    /// (and skipping) any number of leading, trailing, or blank/empty
    /// separators between statements.
    fn parse_statements(&mut self) -> CalcResult<Program> {
        let mut stmts = Vec::new();
        self.skip_semis();
        while !matches!(self.peek(), Token::Eof) {
            stmts.push(self.parse_statement()?);
            self.skip_semis();
        }
        if stmts.is_empty() {
            return Err(CalcError::UnexpectedEof);
        }
        Ok(stmts)
    }

    /// `name := expr` (assignment) if the next two tokens are an identifier
    /// followed by `:=`, otherwise a plain expression.
    fn parse_statement(&mut self) -> CalcResult<Stmt> {
        if let Token::Ident(name) = self.peek().clone() {
            if matches!(self.tokens.get(self.pos + 1), Some(Token::Assign)) {
                self.advance(); // identifier
                self.advance(); // :=
                let expr = self.parse_comparison()?;
                return Ok(Stmt::Assign(name, expr));
            }
        }
        Ok(Stmt::Expr(self.parse_comparison()?))
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

    #[test]
    fn parse_program_single_expression_is_one_statement() {
        assert_eq!(
            parse_program("2 + 2").unwrap(),
            vec![Stmt::Expr(Expr::Binary(
                BinaryOp::Add,
                Box::new(Expr::Number(2.0)),
                Box::new(Expr::Number(2.0))
            ))]
        );
    }

    #[test]
    fn parse_program_assignment_statement() {
        assert_eq!(
            parse_program("x := 5").unwrap(),
            vec![Stmt::Assign("x".to_string(), Expr::Number(5.0))]
        );
    }

    #[test]
    fn parse_program_semicolon_and_newline_separated_statements() {
        let expected = vec![
            Stmt::Assign("x".to_string(), Expr::Number(5.0)),
            Stmt::Assign(
                "y".to_string(),
                Expr::Binary(
                    BinaryOp::Pow,
                    Box::new(Expr::Variable("x".to_string())),
                    Box::new(Expr::Number(2.0)),
                ),
            ),
            Stmt::Expr(Expr::Variable("y".to_string())),
        ];
        assert_eq!(parse_program("x := 5; y := x^2; y").unwrap(), expected);
        assert_eq!(parse_program("x := 5\ny := x^2\ny").unwrap(), expected);
    }

    #[test]
    fn parse_program_tolerates_blank_statements() {
        assert_eq!(
            parse_program(";;\n\nx := 5;;\n").unwrap(),
            vec![Stmt::Assign("x".to_string(), Expr::Number(5.0))]
        );
    }

    #[test]
    fn parse_program_empty_input_is_an_error() {
        assert_eq!(parse_program("  ;\n; "), Err(CalcError::UnexpectedEof));
    }

    #[test]
    fn identifier_followed_by_assign_is_not_confused_with_comparison() {
        // `x := y = 5` should be an assignment of `y = 5` (equality) to `x`.
        assert_eq!(
            parse_program("x := y = 5").unwrap(),
            vec![Stmt::Assign(
                "x".to_string(),
                Expr::Binary(
                    BinaryOp::Eq,
                    Box::new(Expr::Variable("y".to_string())),
                    Box::new(Expr::Number(5.0))
                )
            )]
        );
    }
}
