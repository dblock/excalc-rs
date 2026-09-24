//! General-purpose rounding/truncation functions, ported from chapter IV
//! ("General Functions") of the original manual.

use crate::error::{CalcError, CalcResult};
use rand::RngExt;

/// Absolute value.
pub fn abs(x: f64) -> CalcResult<f64> {
    Ok(x.abs())
}

/// Truncates towards zero, e.g. `trunc(2.9) == 2.0`, `trunc(-2.9) == -2.0`.
///
/// The original manual documents `Intg` and `Trunc` identically ("returns
/// the integer portion of a value"); this port treats `intg` as an alias of
/// `trunc` rather than inventing a second, subtly different behavior.
pub fn trunc(x: f64) -> CalcResult<f64> {
    Ok(x.trunc())
}

/// Fractional part: `frac(x) = x - trunc(x)`.
pub fn frac(x: f64) -> CalcResult<f64> {
    Ok(x - x.trunc())
}

/// Rounds to the closest integer, ties rounding towards positive infinity
/// (`round(-2.5) == -2.0`, `round(-2.6) == -3.0`), matching the three worked
/// examples in the original manual. This is "round half up", not Rust's
/// `f64::round` (which rounds ties away from zero).
pub fn round(x: f64) -> CalcResult<f64> {
    Ok((x + 0.5).floor())
}

/// Ceiling: the closest integer `>= x`.
pub fn ceil(x: f64) -> CalcResult<f64> {
    Ok(x.ceil())
}

/// Floor: the closest integer `<= x`.
pub fn floor(x: f64) -> CalcResult<f64> {
    Ok(x.floor())
}

/// A uniformly distributed random number in `[0, x]`.
pub fn random(x: f64) -> CalcResult<f64> {
    if x < 0.0 {
        return Err(CalcError::DomainError("random".to_string()));
    }
    if x == 0.0 {
        return Ok(0.0);
    }
    let mut rng = rand::rng();
    Ok(rng.random_range(0.0..=x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abs_negative_and_positive() {
        assert_eq!(abs(-2.0).unwrap(), 2.0);
        assert_eq!(abs(2.0).unwrap(), 2.0);
        assert_eq!(abs(0.0).unwrap(), 0.0);
    }

    #[test]
    fn trunc_towards_zero() {
        assert_eq!(trunc(2.1).unwrap(), 2.0);
        assert_eq!(trunc(-2.1).unwrap(), -2.0);
    }

    #[test]
    fn frac_matches_manual_example() {
        assert!((frac(1.345).unwrap() - 0.345).abs() < 1e-9);
        assert!((frac(-1.345).unwrap() - -0.345).abs() < 1e-9);
    }

    #[test]
    fn round_half_up_matches_manual_examples() {
        assert_eq!(round(-2.1).unwrap(), -2.0);
        assert_eq!(round(-2.5).unwrap(), -2.0);
        assert_eq!(round(-2.6).unwrap(), -3.0);
        assert_eq!(round(2.5).unwrap(), 3.0);
    }

    #[test]
    fn ceil_and_floor_match_manual_examples() {
        assert_eq!(ceil(2.1).unwrap(), 3.0);
        assert_eq!(ceil(-2.1).unwrap(), -2.0);
        assert_eq!(floor(2.1).unwrap(), 2.0);
        assert_eq!(floor(-2.1).unwrap(), -3.0);
    }

    #[test]
    fn random_domain_and_range() {
        assert_eq!(
            random(-1.0),
            Err(CalcError::DomainError("random".to_string()))
        );
        assert_eq!(random(0.0).unwrap(), 0.0);
        for _ in 0..100 {
            let r = random(8.0).unwrap();
            assert!((0.0..=8.0).contains(&r));
        }
    }
}
