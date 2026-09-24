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

/// Base-2 logarithm.
pub fn log2(x: f64) -> CalcResult<f64> {
    if x <= 0.0 {
        return Err(CalcError::DomainError("log2".to_string()));
    }
    Ok(x.log2())
}

/// Cube root. Unlike `sqrt`, defined for negative `x` too
/// (`cbrt(-8) == -2`).
pub fn cbrt(x: f64) -> CalcResult<f64> {
    Ok(x.cbrt())
}

/// Euclidean distance `sqrt(x^2 + y^2)`, computed via `f64::hypot` to avoid
/// intermediate overflow/underflow for very large or very small inputs.
pub fn hypot(x: f64, y: f64) -> CalcResult<f64> {
    Ok(x.hypot(y))
}

/// Sign of `x`: `-1` if negative, `0` if zero, `1` if positive.
pub fn sign(x: f64) -> CalcResult<f64> {
    Ok(if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    })
}

/// Clamps `x` to the closed interval `[lo, hi]`.
pub fn clamp(x: f64, lo: f64, hi: f64) -> CalcResult<f64> {
    if lo > hi {
        return Err(CalcError::DomainError("clamp".to_string()));
    }
    Ok(x.clamp(lo, hi))
}

/// Linear interpolation between `a` and `b` at parameter `t`
/// (`lerp(a, b, 0) == a`, `lerp(a, b, 1) == b`). `t` isn't required to be
/// in `[0, 1]`; values outside that range extrapolate.
pub fn lerp(a: f64, b: f64, t: f64) -> CalcResult<f64> {
    Ok(a + (b - a) * t)
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

    #[test]
    fn log2_matches_powers_of_two() {
        assert_eq!(log2(8.0).unwrap(), 3.0);
        assert_eq!(log2(1.0).unwrap(), 0.0);
        assert_eq!(log2(0.0), Err(CalcError::DomainError("log2".to_string())));
        assert_eq!(log2(-1.0), Err(CalcError::DomainError("log2".to_string())));
    }

    #[test]
    fn cbrt_handles_negative_input() {
        assert_eq!(cbrt(8.0).unwrap(), 2.0);
        assert_eq!(cbrt(-8.0).unwrap(), -2.0);
        assert_eq!(cbrt(0.0).unwrap(), 0.0);
    }

    #[test]
    fn hypot_matches_pythagorean_triple() {
        assert_eq!(hypot(3.0, 4.0).unwrap(), 5.0);
    }

    #[test]
    fn sign_of_negative_zero_and_positive() {
        assert_eq!(sign(-5.0).unwrap(), -1.0);
        assert_eq!(sign(0.0).unwrap(), 0.0);
        assert_eq!(sign(5.0).unwrap(), 1.0);
    }

    #[test]
    fn clamp_bounds_and_domain_error() {
        assert_eq!(clamp(5.0, 0.0, 10.0).unwrap(), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0).unwrap(), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0).unwrap(), 10.0);
        assert_eq!(
            clamp(5.0, 10.0, 0.0),
            Err(CalcError::DomainError("clamp".to_string()))
        );
    }

    #[test]
    fn lerp_interpolates_and_extrapolates() {
        assert_eq!(lerp(0.0, 10.0, 0.0).unwrap(), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0).unwrap(), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5).unwrap(), 5.0);
        assert_eq!(lerp(0.0, 10.0, 2.0).unwrap(), 20.0);
    }
}
