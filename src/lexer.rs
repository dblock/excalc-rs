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
    Eq,        // =  equality
    Gt,        // >  greater than
    Lt,        // <  less than
    Amp,       // &  bitwise and (synonym for the `and` keyword)
    Assign,    // := variable assignment
    Question,  // ?  ternary conditional: cond ? then : else
    Colon,     // :  ternary conditional's separator (not part of `:=`)
    Semi,      // ;  or a newline: statement separator
    LParen,
    RParen,
    Comma,
    Eof,
}

/// The number-literal formatting hints detected while lexing, so the
/// evaluator can echo them back in the printed result: the first `,`/`_`
/// thousands-grouping separator, and the first `$`/`£`/`€`/`¥` currency
/// symbol, encountered anywhere in the input (each independently, since a
/// literal can use both, e.g. `$1,234`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NumberFormat {
    pub separator: Option<char>,
    pub currency: Option<char>,
}

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    input: &'a str,
    /// The first `,`/`_` thousands-grouping separator encountered while
    /// reading a number literal, if any (see `scan_grouped_integer`).
    /// The first currency symbol (`$`/`£`/`€`/`¥`) seen prefixing a number
    /// literal, if any.
    /// Surfaced by `tokenize_with_number_format` so the evaluator can echo
    /// the same separator/currency back in its printed result.
    group_separator: Option<char>,
    currency: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.char_indices().peekable(),
            input,
            group_separator: None,
            currency: None,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, c)| c)
    }

    pub fn tokenize(mut self) -> CalcResult<(Vec<Token>, NumberFormat)> {
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
                '=' => Token::Eq,
                '>' => Token::Gt,
                '<' => Token::Lt,
                '&' => Token::Amp,
                '?' => Token::Question,
                ':' => {
                    if self.peek_char() == Some('=') {
                        self.chars.next();
                        Token::Assign
                    } else {
                        Token::Colon
                    }
                }
                ';' | '\n' => Token::Semi,
                '(' => Token::LParen,
                ')' => Token::RParen,
                ',' => Token::Comma,
                '0'..='9' | '.' => self.read_number(pos, c)?,
                '$' | '£' | '€' | '¥' => self.read_currency_number(pos, c)?,
                c if c.is_ascii_alphabetic() || c == '_' => self.read_ident(c),
                other => return Err(CalcError::InvalidCharacter(other, pos)),
            };
            tokens.push(token);
        }
        Ok((
            tokens,
            NumberFormat {
                separator: self.group_separator,
                currency: self.currency,
            },
        ))
    }

    /// Skips ordinary whitespace, but stops at (doesn't consume) a newline —
    /// callers treat `\n` as a statement separator token, not whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c == '\n' {
                break;
            } else if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self, start: usize, first: char) -> CalcResult<Token> {
        if first == '0' {
            if let Some(radix_char @ ('x' | 'X' | 'o' | 'O' | 'b' | 'B')) = self.peek_char() {
                return self.read_radix_number(start, radix_char);
            }
        }
        let mut end = start + first.len_utf8();
        let mut seen_dot = first == '.';

        // `1,234` / `1_234_567`-style thousands grouping: only tried when
        // the literal starts with a digit (not `.`), and only commits if
        // `scan_grouped_integer` finds a fully well-formed grouping (see
        // its doc comment) — otherwise this falls through to the ordinary
        // digit loop below unchanged, so e.g. `sum(1,2)` still lexes as two
        // arguments exactly as before.
        let mut number_separator = None;
        if first.is_ascii_digit() {
            if let Some((grouped_end, sep)) = scan_grouped_integer(self.input, start) {
                for _ in 0..(grouped_end - end) {
                    self.chars.next();
                }
                end = grouped_end;
                number_separator = Some(sep);
                self.group_separator.get_or_insert(sep);
            }
        }

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
        let raw_text = &self.input[start..end];
        let owned_text;
        let text: &str = if let Some(sep) = number_separator {
            owned_text = raw_text.replace(sep, "");
            &owned_text
        } else {
            raw_text
        };
        text.parse::<f64>()
            .map(Token::Number)
            .map_err(|_| CalcError::InvalidCharacter(first, start))
    }

    /// Reads a `$`/`£`/`€`/`¥`-prefixed number literal (e.g. `$100`,
    /// `£1,234.56`), called once the main loop has consumed the currency
    /// symbol. The symbol itself carries no arithmetic meaning — the
    /// number behind it is read exactly like any other numeric literal
    /// (including `,`/`_` grouping) — but the *first* currency symbol seen
    /// anywhere in the input is recorded so the evaluator can prefix the
    /// printed result with it (see `NumberFormat`).
    fn read_currency_number(&mut self, start: usize, symbol: char) -> CalcResult<Token> {
        let Some((npos, nc)) = self.chars.next() else {
            return Err(CalcError::InvalidCharacter(symbol, start));
        };
        if !(nc.is_ascii_digit() || nc == '.') {
            return Err(CalcError::InvalidCharacter(symbol, start));
        }
        let token = self.read_number(npos, nc)?;
        self.currency.get_or_insert(symbol);
        Ok(token)
    }

    /// Reads a `0x`/`0o`/`0b`-prefixed radix literal (e.g. `0xff`, `0o17`,
    /// `0b1010`), called once `read_number` has seen the leading `0` and
    /// peeked the radix marker. Errors if no valid digits for that radix
    /// follow the marker.
    fn read_radix_number(&mut self, start: usize, radix_char: char) -> CalcResult<Token> {
        self.chars.next(); // consume the radix marker (x/X/o/O/b/B)
        let radix: u32 = match radix_char {
            'x' | 'X' => 16,
            'o' | 'O' => 8,
            'b' | 'B' => 2,
            _ => unreachable!("read_radix_number only called for x/X/o/O/b/B"),
        };
        let digits_start = start + 2; // past "0x"/"0o"/"0b"
        let mut digits_end = digits_start;
        while let Some(c) = self.peek_char() {
            if c.is_digit(radix) {
                digits_end += c.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }
        if digits_end == digits_start {
            return Err(CalcError::InvalidCharacter(radix_char, start));
        }
        let digits = &self.input[digits_start..digits_end];
        u64::from_str_radix(digits, radix)
            .map(|n| Token::Number(n as f64))
            .map_err(|_| CalcError::InvalidCharacter(radix_char, start))
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

    /// Identifiers may end with a single trailing `?` (Scheme-style predicate
    /// naming convention, e.g. `prime?`), matched greedily like any other
    /// identifier character but not expected to appear more than once.
    fn read_ident(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '?' {
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
    Lexer::new(input).tokenize().map(|(tokens, _)| tokens)
}

/// Like [`tokenize`], but also returns the [`NumberFormat`] hints (the
/// first `,`/`_` thousands-grouping separator and the first `$`/`£`/`€`/`¥`
/// currency symbol) used by any number literal in `input`. Used by the
/// top-level evaluation entry points so grouped/currency input like
/// `$1,000 + 1` can echo the same formatting back in its printed result
/// (`$1,001`).
pub fn tokenize_with_number_format(input: &str) -> CalcResult<(Vec<Token>, NumberFormat)> {
    Lexer::new(input).tokenize()
}

/// Attempts to match a `,`/`_`-grouped integer literal (e.g. `1,234` or
/// `1_234_567`) starting at byte offset `start` in `input`, which must be
/// the position of a leading ASCII digit. A valid grouping is: 1-3 digits,
/// then one or more repetitions of (separator, exactly 3 digits), using
/// the *same* separator character throughout — the conventional
/// thousands-grouping shape. Returns the exclusive end byte offset and the
/// separator used if at least one such group was found; otherwise `None`,
/// in which case the caller falls back to reading a plain, separator-free
/// integer (so e.g. `sum(1,2)` still lexes as two comma-separated
/// arguments, since `2` isn't a valid 3-digit trailing group).
fn scan_grouped_integer(input: &str, start: usize) -> Option<(usize, char)> {
    let bytes = input.as_bytes();
    let mut i = start;
    let mut digit_run = 0u8;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        digit_run += 1;
    }
    if digit_run == 0 || digit_run > 3 {
        return None;
    }
    let mut separator = None;
    let mut end = i;
    while let Some(&c) = bytes.get(i) {
        if c != b',' && c != b'_' {
            break;
        }
        if let Some(sep) = separator {
            if sep != c {
                break; // mixed separators within one literal: stop here
            }
        }
        let group_start = i + 1;
        let mut group_end = group_start;
        while group_end < bytes.len() && bytes[group_end].is_ascii_digit() {
            group_end += 1;
        }
        if group_end - group_start != 3 {
            break; // a group after a separator must be exactly 3 digits
        }
        separator = Some(c); // only commit once this group has validated
        i = group_end;
        end = group_end;
    }
    separator.map(|sep| (end, sep as char))
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
    fn comma_grouped_number() {
        assert_eq!(
            tokenize_with_number_format("1,234"),
            Ok((
                vec![Token::Number(1234.0), Token::Eof],
                NumberFormat {
                    separator: Some(','),
                    currency: None
                }
            ))
        );
    }

    #[test]
    fn underscore_grouped_number() {
        assert_eq!(
            tokenize_with_number_format("1_234_567"),
            Ok((
                vec![Token::Number(1_234_567.0), Token::Eof],
                NumberFormat {
                    separator: Some('_'),
                    currency: None
                }
            ))
        );
    }

    #[test]
    fn grouped_number_with_fraction() {
        assert_eq!(
            tokenize_with_number_format("1,234.56"),
            Ok((
                vec![Token::Number(1234.56), Token::Eof],
                NumberFormat {
                    separator: Some(','),
                    currency: None
                }
            ))
        );
    }

    #[test]
    fn short_trailing_group_falls_back_to_call_args() {
        // "1,2" isn't a valid grouped number (the group after `,` must be
        // exactly 3 digits), so it must still tokenize as two separate
        // numbers, preserving `sum(1,2)`-style 2-argument calls.
        assert_eq!(
            tokenize_with_number_format("1,2"),
            Ok((
                vec![
                    Token::Number(1.0),
                    Token::Comma,
                    Token::Number(2.0),
                    Token::Eof
                ],
                NumberFormat::default()
            ))
        );
        assert_eq!(
            tokenize_with_number_format("sum(1,2)"),
            Ok((
                vec![
                    Token::Ident("sum".to_string()),
                    Token::LParen,
                    Token::Number(1.0),
                    Token::Comma,
                    Token::Number(2.0),
                    Token::RParen,
                    Token::Eof
                ],
                NumberFormat::default()
            ))
        );
    }

    #[test]
    fn incomplete_trailing_group_keeps_valid_prefix() {
        // The first two groups ("1,234") are valid and grouped; the
        // trailing ",5" is not a full 3-digit group, so it's left for
        // ordinary tokenization (a real 2-argument call).
        assert_eq!(
            tokenize_with_number_format("1,234,5"),
            Ok((
                vec![
                    Token::Number(1234.0),
                    Token::Comma,
                    Token::Number(5.0),
                    Token::Eof
                ],
                NumberFormat {
                    separator: Some(','),
                    currency: None
                }
            ))
        );
    }

    #[test]
    fn first_separator_wins() {
        let (_, format) = tokenize_with_number_format("1,234 + 1_000").unwrap();
        assert_eq!(format.separator, Some(','));
    }

    #[test]
    fn ungrouped_numbers_have_no_separator() {
        let (tokens, format) = tokenize_with_number_format("123 + 4.5").unwrap();
        assert_eq!(format, NumberFormat::default());
        assert_eq!(
            tokens,
            vec![
                Token::Number(123.0),
                Token::Plus,
                Token::Number(4.5),
                Token::Eof
            ]
        );
    }

    #[test]
    fn dollar_prefixed_number() {
        assert_eq!(
            tokenize_with_number_format("$100"),
            Ok((
                vec![Token::Number(100.0), Token::Eof],
                NumberFormat {
                    separator: None,
                    currency: Some('$')
                }
            ))
        );
    }

    #[test]
    fn pound_prefixed_number_with_grouping() {
        assert_eq!(
            tokenize_with_number_format("£1,234.56"),
            Ok((
                vec![Token::Number(1234.56), Token::Eof],
                NumberFormat {
                    separator: Some(','),
                    currency: Some('£')
                }
            ))
        );
    }

    #[test]
    fn euro_and_yen_prefixed_numbers() {
        let (tokens, format) = tokenize_with_number_format("€50").unwrap();
        assert_eq!(tokens, vec![Token::Number(50.0), Token::Eof]);
        assert_eq!(format.currency, Some('€'));

        let (tokens, format) = tokenize_with_number_format("¥1000").unwrap();
        assert_eq!(tokens, vec![Token::Number(1000.0), Token::Eof]);
        assert_eq!(format.currency, Some('¥'));
    }

    #[test]
    fn first_currency_wins() {
        let (_, format) = tokenize_with_number_format("$1 + £2").unwrap();
        assert_eq!(format.currency, Some('$'));
    }

    #[test]
    fn currency_symbol_without_digit_errors() {
        assert_eq!(
            tokenize_with_number_format("$"),
            Err(CalcError::InvalidCharacter('$', 0))
        );
        assert_eq!(
            tokenize_with_number_format("$a"),
            Err(CalcError::InvalidCharacter('$', 0))
        );
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
    fn radix_literals() {
        assert_eq!(tokenize("0xff"), Ok(vec![Token::Number(255.0), Token::Eof]));
        assert_eq!(tokenize("0XFF"), Ok(vec![Token::Number(255.0), Token::Eof]));
        assert_eq!(tokenize("0o17"), Ok(vec![Token::Number(15.0), Token::Eof]));
        assert_eq!(
            tokenize("0b1010"),
            Ok(vec![Token::Number(10.0), Token::Eof])
        );
        // A bare "0" (or "0" followed by ordinary digits) is still a
        // regular decimal number.
        assert_eq!(tokenize("0"), Ok(vec![Token::Number(0.0), Token::Eof]));
        assert_eq!(tokenize("0.5"), Ok(vec![Token::Number(0.5), Token::Eof]));
        assert_eq!(tokenize("012"), Ok(vec![Token::Number(12.0), Token::Eof]));
    }

    #[test]
    fn radix_literal_requires_digits() {
        assert_eq!(tokenize("0x"), Err(CalcError::InvalidCharacter('x', 0)));
        assert_eq!(tokenize("0xzz"), Err(CalcError::InvalidCharacter('x', 0)));
        assert_eq!(tokenize("0b2"), Err(CalcError::InvalidCharacter('b', 0)));
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

    #[test]
    fn identifier_with_trailing_question_mark() {
        assert_eq!(
            tokenize("prime?(86)"),
            Ok(vec![
                Token::Ident("prime?".to_string()),
                Token::LParen,
                Token::Number(86.0),
                Token::RParen,
                Token::Eof
            ])
        );
    }

    #[test]
    fn comparison_and_ampersand_tokens() {
        assert_eq!(
            tokenize("= > < &"),
            Ok(vec![
                Token::Eq,
                Token::Gt,
                Token::Lt,
                Token::Amp,
                Token::Eof
            ])
        );
    }

    #[test]
    fn assign_and_statement_separator_tokens() {
        assert_eq!(
            tokenize("x := 5; y"),
            Ok(vec![
                Token::Ident("x".to_string()),
                Token::Assign,
                Token::Number(5.0),
                Token::Semi,
                Token::Ident("y".to_string()),
                Token::Eof
            ])
        );
        assert_eq!(
            tokenize("x := 5\ny"),
            Ok(vec![
                Token::Ident("x".to_string()),
                Token::Assign,
                Token::Number(5.0),
                Token::Semi,
                Token::Ident("y".to_string()),
                Token::Eof
            ])
        );
    }

    #[test]
    fn lone_colon_tokenizes_as_colon() {
        assert_eq!(tokenize(":"), Ok(vec![Token::Colon, Token::Eof]));
        assert_eq!(
            tokenize("x : 5"),
            Ok(vec![
                Token::Ident("x".to_string()),
                Token::Colon,
                Token::Number(5.0),
                Token::Eof
            ])
        );
    }

    #[test]
    fn question_mark_tokenizes_as_question_when_not_in_ident() {
        assert_eq!(
            tokenize("1 ? 2 : 3"),
            Ok(vec![
                Token::Number(1.0),
                Token::Question,
                Token::Number(2.0),
                Token::Colon,
                Token::Number(3.0),
                Token::Eof
            ])
        );
    }
}
