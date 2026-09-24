//! Lexer: turns an input string into a stream of tokens.

use crate::error::{CalcError, CalcResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,     // ^  power
    Bang,      // !  factorial (postfix)
    Percent,   // %  percent (postfix)
    Backslash, // \  nth root (binary): `n \ x` means the n-th root of x
    LParen,
    RParen,
    Comma,
    Eof,
}

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.char_indices().peekable(),
            input,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, c)| c)
    }

    pub fn tokenize(mut self) -> CalcResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            let (pos, c) = match self.chars.next() {
                Some(pair) => pair,
                None => {
                    tokens.push(Token::Eof);
                    break;
                }
            };
            let token = match c {
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '^' => Token::Caret,
                '!' => Token::Bang,
                '%' => Token::Percent,
                '\\' => Token::Backslash,
                '(' => Token::LParen,
                ')' => Token::RParen,
                ',' => Token::Comma,
                '0'..='9' | '.' => self.read_number(pos, c)?,
                c if c.is_ascii_alphabetic() || c == '_' => self.read_ident(c),
                other => return Err(CalcError::InvalidCharacter(other, pos)),
            };
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self, start: usize, first: char) -> CalcResult<Token> {
        let mut end = start + first.len_utf8();
        let mut seen_dot = first == '.';
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                end += c.len_utf8();
                self.chars.next();
            } else if c == '.' && !seen_dot {
                seen_dot = true;
                end += c.len_utf8();
                self.chars.next();
            } else if (c == 'e' || c == 'E') && self.is_exponent_start() {
                // Scientific notation, e.g. 1e10 or 1e-10.
                end += c.len_utf8();
                self.chars.next();
                if let Some(sign @ ('+' | '-')) = self.peek_char() {
                    end += sign.len_utf8();
                    self.chars.next();
                }
                while let Some(d) = self.peek_char() {
                    if d.is_ascii_digit() {
                        end += d.len_utf8();
                        self.chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        let text = &self.input[start..end];
        text.parse::<f64>()
            .map(Token::Number)
            .map_err(|_| CalcError::InvalidCharacter(first, start))
    }

    /// Only treat 'e'/'E' as the start of an exponent if followed by a digit
    /// or a sign-then-digit, so `e` (Euler's number) and `exp(x)` still lex
    /// as identifiers when they don't immediately follow digits meant as a
    /// mantissa (the caller only calls this while already reading a number).
    fn is_exponent_start(&mut self) -> bool {
        let mut lookahead = self.chars.clone();
        lookahead.next(); // consume 'e'/'E' hypothetically
        match lookahead.peek() {
            Some(&(_, d)) if d.is_ascii_digit() => true,
            Some(&(_, '+')) | Some(&(_, '-')) => {
                lookahead.next();
                matches!(lookahead.peek(), Some(&(_, d)) if d.is_ascii_digit())
            }
            _ => false,
        }
    }

    fn read_ident(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                s.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        Token::Ident(s)
    }
}

pub fn tokenize(input: &str) -> CalcResult<Vec<Token>> {
    Lexer::new(input).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_character_errors() {
        assert_eq!(tokenize("@"), Err(CalcError::InvalidCharacter('@', 0)));
    }

    #[test]
    fn decimal_numbers() {
        assert_eq!(tokenize("2.75"), Ok(vec![Token::Number(2.75), Token::Eof]));
    }

    #[test]
    fn scientific_notation() {
        assert_eq!(tokenize("1e10"), Ok(vec![Token::Number(1e10), Token::Eof]));
        assert_eq!(tokenize("1E10"), Ok(vec![Token::Number(1e10), Token::Eof]));
        assert_eq!(
            tokenize("1.5e-3"),
            Ok(vec![Token::Number(1.5e-3), Token::Eof])
        );
        assert_eq!(tokenize("2e+3"), Ok(vec![Token::Number(2e3), Token::Eof]));
    }

    #[test]
    fn e_not_followed_by_digit_is_not_an_exponent() {
        // "1ex" -> Number(1) then an identifier "ex", not scientific notation.
        assert_eq!(
            tokenize("1ex"),
            Ok(vec![
                Token::Number(1.0),
                Token::Ident("ex".to_string()),
                Token::Eof
            ])
        );
        // "1e+x" -> the sign lookahead fails since 'x' isn't a digit either.
        assert_eq!(
            tokenize("1e+x"),
            Ok(vec![
                Token::Number(1.0),
                Token::Ident("e".to_string()),
                Token::Plus,
                Token::Ident("x".to_string()),
                Token::Eof
            ])
        );
    }

    #[test]
    fn scientific_notation_followed_by_more_tokens() {
        // Exercises the exponent-digit loop terminating on a non-digit
        // (rather than end-of-input), e.g. "1e10+5".
        assert_eq!(
            tokenize("1e10+5"),
            Ok(vec![
                Token::Number(1e10),
                Token::Plus,
                Token::Number(5.0),
                Token::Eof
            ])
        );
    }

    #[test]
    fn euler_constant_still_lexes_as_identifier() {
        assert_eq!(
            tokenize("e"),
            Ok(vec![Token::Ident("e".to_string()), Token::Eof])
        );
    }
}
