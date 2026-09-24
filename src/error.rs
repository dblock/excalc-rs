use thiserror::Error;

/// All errors that can occur while lexing, parsing, or evaluating an expression.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum CalcError {
    #[error("invalid character '{0}' at position {1}")]
    InvalidCharacter(char, usize),

    #[error("unexpected end of input")]
    UnexpectedEof,

    #[error("expected '{expected}' at position {position}")]
    ExpectedToken { expected: String, position: usize },

    #[error("unknown function: {0}")]
    UnknownFunction(String),

    #[error("unknown variable: {0}")]
    UnknownVariable(String),

    #[error("wrong number of arguments for {name}: expected {expected}, got {got}")]
    WrongArgCount {
        name: String,
        expected: String,
        got: usize,
    },

    #[error("division by zero")]
    DivisionByZero,

    #[error("domain error in {0}")]
    DomainError(String),

    #[error("numeric overflow")]
    Overflow,
}

pub type CalcResult<T> = Result<T, CalcError>;
