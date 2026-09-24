//! Root-finding: `bisect`/`secant` locate a zero of an expression treated
//! as a function of a single variable, mirroring `functions::integration`'s
//! pattern of a caller-supplied sampling closure — see `eval.rs`'s
//! special-cased handling of these function names for how the
//! expression/variable/context wiring works.

use crate::error::{CalcError, CalcResult};

/// Maximum number of iterations before giving up (matches
/// `integration::ADAPTIVE_MAX_DOUBLINGS`'s role of an escape hatch for
/// inputs that never converge to the requested tolerance).
const MAX_ITERATIONS: u32 = 200;

/// Finds a root of `f` in `[a, b]` via the bisection method: repeatedly
/// halves the bracket, keeping whichever half still contains a sign
/// change, until the bracket width is within `tolerance`. Requires `f(a)`
/// and `f(b)` to have opposite signs (a valid bracket); `a`/`b` order
/// doesn't matter.
pub fn bisect(
    name: &str,
    mut f: impl FnMut(f64) -> CalcResult<f64>,
    a: f64,
    b: f64,
    tolerance: f64,
) -> CalcResult<f64> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    let (mut lo, mut hi) = (a, b);
    let mut f_lo = f(lo)?;
    let f_hi = f(hi)?;
    if f_lo == 0.0 {
        return Ok(lo);
    }
    if f_hi == 0.0 {
        return Ok(hi);
    }
    if f_lo.signum() == f_hi.signum() {
        return Err(CalcError::DomainError(name.to_string()));
    }
    for _ in 0..MAX_ITERATIONS {
        let mid = (lo + hi) / 2.0;
        let f_mid = f(mid)?;
        if f_mid == 0.0 || (hi - lo).abs() / 2.0 < tolerance {
            return Ok(mid);
        }
        if f_mid.signum() == f_lo.signum() {
            lo = mid;
            f_lo = f_mid;
        } else {
            hi = mid;
        }
    }
    Err(CalcError::Overflow)
}

/// Finds a root of `f` near `x0`/`x1` via the secant method: extrapolates a
/// line through the last two samples to estimate the next guess, without
/// requiring a bracketing interval (unlike [`bisect`]) or a derivative
/// (unlike Newton-Raphson).
pub fn secant(
    name: &str,
    mut f: impl FnMut(f64) -> CalcResult<f64>,
    x0: f64,
    x1: f64,
    tolerance: f64,
) -> CalcResult<f64> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    let (mut x_prev, mut x_curr) = (x0, x1);
    let mut f_prev = f(x_prev)?;
    for _ in 0..MAX_ITERATIONS {
        let f_curr = f(x_curr)?;
        if f_curr == 0.0 || (x_curr - x_prev).abs() < tolerance {
            return Ok(x_curr);
        }
        let denom = f_curr - f_prev;
        if denom == 0.0 {
            return Err(CalcError::DomainError(name.to_string()));
        }
        let x_next = x_curr - f_curr * (x_curr - x_prev) / denom;
        if !x_next.is_finite() {
            return Err(CalcError::Overflow);
        }
        x_prev = x_curr;
        f_prev = f_curr;
        x_curr = x_next;
    }
    Err(CalcError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    // f(x) = x^2 - 2, root at sqrt(2).
    fn sqrt2_minus(x: f64) -> CalcResult<f64> {
        Ok(x * x - 2.0)
    }

    #[test]
    fn bisect_finds_sqrt_2() {
        let result = bisect("bisect", sqrt2_minus, 0.0, 2.0, 1e-9).unwrap();
        assert!((result - std::f64::consts::SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn bisect_bounds_order_does_not_matter() {
        let result = bisect("bisect", sqrt2_minus, 2.0, 0.0, 1e-9).unwrap();
        assert!((result - std::f64::consts::SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn bisect_returns_endpoint_that_is_exactly_zero() {
        // f(sqrt(2)) == 0 exactly isn't representable, so use a linear
        // function with an exact integer root instead.
        let f = |x: f64| -> CalcResult<f64> { Ok(x - 2.0) };
        assert_eq!(bisect("bisect", f, 2.0, 5.0, 1e-9), Ok(2.0));
        assert_eq!(bisect("bisect", f, -1.0, 2.0, 1e-9), Ok(2.0));
    }

    #[test]
    fn bisect_requires_a_sign_change() {
        assert_eq!(
            bisect("bisect", sqrt2_minus, 3.0, 4.0, 1e-9),
            Err(CalcError::DomainError("bisect".to_string()))
        );
    }

    #[test]
    fn bisect_rejects_non_positive_tolerance() {
        assert_eq!(
            bisect("bisect", sqrt2_minus, 0.0, 2.0, 0.0),
            Err(CalcError::DomainError("bisect".to_string()))
        );
    }

    #[test]
    fn bisect_propagates_sample_errors() {
        let f = |x: f64| -> CalcResult<f64> {
            if x > 1.9 {
                Err(CalcError::DomainError("test".to_string()))
            } else {
                Ok(x - 2.0)
            }
        };
        assert_eq!(
            bisect("bisect", f, 0.0, 2.0, 1e-9),
            Err(CalcError::DomainError("test".to_string()))
        );
    }

    #[test]
    fn secant_finds_sqrt_2() {
        let result = secant("secant", sqrt2_minus, 0.0, 2.0, 1e-9).unwrap();
        assert!((result - std::f64::consts::SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn secant_rejects_non_positive_tolerance() {
        assert_eq!(
            secant("secant", sqrt2_minus, 0.0, 2.0, 0.0),
            Err(CalcError::DomainError("secant".to_string()))
        );
    }

    #[test]
    fn secant_errors_when_samples_agree() {
        // A constant function never produces a sign change / progress.
        let f = |_: f64| -> CalcResult<f64> { Ok(5.0) };
        assert_eq!(
            secant("secant", f, 0.0, 1.0, 1e-9),
            Err(CalcError::DomainError("secant".to_string()))
        );
    }

    #[test]
    fn secant_propagates_sample_errors() {
        // Errors on the very first sample of x1, so it propagates
        // regardless of how quickly the algorithm would otherwise converge.
        let f = |x: f64| -> CalcResult<f64> {
            if x > 0.5 {
                Err(CalcError::DomainError("test".to_string()))
            } else {
                Ok(x - 2.0)
            }
        };
        assert_eq!(
            secant("secant", f, 0.0, 1.0, 1e-9),
            Err(CalcError::DomainError("test".to_string()))
        );
    }
}
