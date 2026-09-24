//! excalc: a portable expression calculator, usable as a library, CLI, or
//! (eventually) an MCP tool for AI coding agents.
//!
//! ```
//! use excalc::evaluate;
//! assert_eq!(evaluate("2 + 2 * 3").unwrap(), 8.0);
//! ```

pub mod ast;
pub mod error;
pub mod eval;
pub mod functions;
pub mod lexer;
pub mod parser;

pub use error::{CalcError, CalcResult};

/// Parses and evaluates an expression with no variable bindings.
pub fn evaluate(input: &str) -> CalcResult<f64> {
    let expr = parser::parse(input)?;
    let ctx = eval::Context::new();
    eval::eval(&expr, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arithmetic() {
        assert_eq!(evaluate("2 + 2").unwrap(), 4.0);
        assert_eq!(evaluate("2 + 2 * 3").unwrap(), 8.0);
        assert_eq!(evaluate("(2 + 2) * 3").unwrap(), 12.0);
        assert_eq!(evaluate("10 / 4").unwrap(), 2.5);
        assert_eq!(evaluate("2 - 3 - 4").unwrap(), -5.0);
    }

    #[test]
    fn unary_minus_and_power_precedence() {
        // Conventional precedence: unary minus binds looser than power.
        assert_eq!(evaluate("-2^2").unwrap(), -4.0);
        assert_eq!(evaluate("(-2)^2").unwrap(), 4.0);
        assert_eq!(evaluate("2^3^2").unwrap(), 512.0); // right-assoc: 2^(3^2)
    }

    #[test]
    fn root_operator() {
        // n \ x = n-th root of x
        assert!((evaluate("2 \\ 9").unwrap() - 3.0).abs() < 1e-9);
        assert!((evaluate("3 \\ 27").unwrap() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn postfix_factorial_and_percent() {
        assert_eq!(evaluate("5!").unwrap(), 120.0);
        assert_eq!(evaluate("0!").unwrap(), 1.0);
        assert_eq!(evaluate("50%").unwrap(), 0.5);
        assert_eq!(evaluate("100 * 50%").unwrap(), 50.0);
    }

    #[test]
    fn mod_operator() {
        assert_eq!(evaluate("10 mod 3").unwrap(), 1.0);
    }

    #[test]
    fn constants() {
        assert!((evaluate("pi").unwrap() - std::f64::consts::PI).abs() < 1e-12);
        assert!((evaluate("e").unwrap() - std::f64::consts::E).abs() < 1e-12);
    }

    #[test]
    fn trig_functions() {
        assert!((evaluate("sin(0)").unwrap()).abs() < 1e-12);
        assert!((evaluate("cos(0)").unwrap() - 1.0).abs() < 1e-12);
        assert!((evaluate("sqrt(16)").unwrap() - 4.0).abs() < 1e-12);
        assert!((evaluate("ln(e)").unwrap() - 1.0).abs() < 1e-9);
        assert!((evaluate("log(100)").unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn statistics_functions() {
        assert_eq!(evaluate("sum(1, 2, 3, 4)").unwrap(), 10.0);
        assert_eq!(evaluate("average(2, 4, 6)").unwrap(), 4.0);
        assert_eq!(evaluate("max(3, 7, 2)").unwrap(), 7.0);
        assert_eq!(evaluate("min(3, 7, 2)").unwrap(), 2.0);
        assert_eq!(evaluate("product(2, 3, 4)").unwrap(), 24.0);
        assert_eq!(evaluate("binom(5, 2)").unwrap(), 10.0);
    }

    #[test]
    fn errors() {
        assert_eq!(evaluate("1 / 0"), Err(CalcError::DivisionByZero));
        assert!(matches!(
            evaluate("sqrt(-1)"),
            Err(CalcError::DomainError(_))
        ));
        assert!(matches!(
            evaluate("nonsense(1)"),
            Err(CalcError::UnknownFunction(_))
        ));
        assert!(matches!(
            evaluate("2 +"),
            Err(CalcError::ExpectedToken { .. })
        ));
    }
}
