//! General-purpose numeric integration, ported from the original's
//! `int`/`gauss`/`simpson`/`trapezoid`/`newton`/`boole`/`ordersix`/`weddle`
//! functions (`fSum`/`fTrapezoid`/`fSimpson`/... and `Tegral`/`Gauss` in
//! `MCalc.pas`). Unlike the rest of this module tree, these operate on an
//! arbitrary expression evaluated repeatedly at different points rather
//! than on plain `f64` arguments — see `eval.rs`'s special-cased handling
//! of these function names for how the expression/variable/context wiring
//! works. This module only implements the numeric quadrature itself, via a
//! caller-supplied sampling closure.

use crate::error::{CalcError, CalcResult};

/// The composite Newton-Cotes rule requested by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Trapezoid,
    Simpson,
    Newton,
    Boole,
    OrderSix,
    Weddle,
}

impl Rule {
    /// Maps a lowercased function name to the rule it selects, including
    /// the original's aliases/typos (`trapez`/`trapezoide` for `trapezoid`,
    /// `ordresix` for `ordersix`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "trapezoid" | "trapez" | "trapezoide" => Some(Rule::Trapezoid),
            "simpson" => Some(Rule::Simpson),
            "newton" => Some(Rule::Newton),
            "boole" => Some(Rule::Boole),
            "ordersix" | "ordresix" => Some(Rule::OrderSix),
            "weddle" => Some(Rule::Weddle),
            _ => None,
        }
    }
}

/// Applies the composite `rule` over `[a, b]` (in either order) using `n`
/// equal-width sub-intervals, sampling the integrand via `f`. Ported from
/// `fSum`/`fTrapezoid`/`fSimpson`/`fNewton`/`fBoole`/`fOrderSix`/`fWeddle`.
///
/// Deviates from the original in one respect: the original steps a `while`
/// loop with a floating-point `<=` comparison (`while xj <= b - h`), which
/// can silently run one sub-interval short or long due to floating-point
/// accumulation. This port instead requires `n` to be a positive whole
/// number and loops exactly `n` times, which is both more predictable and
/// mathematically equivalent to what the original intended.
pub fn composite(
    name: &str,
    rule: Rule,
    mut f: impl FnMut(f64) -> CalcResult<f64>,
    a: f64,
    b: f64,
    n: f64,
) -> CalcResult<f64> {
    if a == b {
        return Ok(0.0);
    }
    if !n.is_finite() || n <= 0.0 || n.fract() != 0.0 {
        return Err(CalcError::DomainError(name.to_string()));
    }

    let (lo, hi, sign) = if a > b { (b, a, -1.0) } else { (a, b, 1.0) };
    let steps = n as u64;
    let h = (hi - lo) / n;

    let mut sum = 0.0;
    let mut x = lo;
    for _ in 0..steps {
        sum += match rule {
            Rule::Trapezoid => h / 2.0 * (f(x)? + f(x + h)?),
            Rule::Simpson => h / 6.0 * (f(x)? + 4.0 * f(x + h / 2.0)? + f(x + h)?),
            Rule::Newton => {
                h / 8.0 * (f(x)? + 3.0 * f(x + h / 3.0)? + 3.0 * f(x + 2.0 * h / 3.0)? + f(x + h)?)
            }
            Rule::Boole => {
                h / 90.0
                    * (7.0 * f(x)?
                        + 32.0 * f(x + h / 4.0)?
                        + 12.0 * f(x + h / 2.0)?
                        + 32.0 * f(x + 3.0 * h / 4.0)?
                        + 7.0 * f(x + h)?)
            }
            Rule::OrderSix => {
                h / 288.0
                    * (19.0 * f(x)?
                        + 75.0 * f(x + h / 5.0)?
                        + 50.0 * f(x + 2.0 * h / 5.0)?
                        + 50.0 * f(x + 3.0 * h / 5.0)?
                        + 75.0 * f(x + 4.0 * h / 5.0)?
                        + 19.0 * f(x + h)?)
            }
            Rule::Weddle => {
                h / 840.0
                    * (41.0 * f(x)?
                        + 216.0 * f(x + h / 6.0)?
                        + 27.0 * f(x + h / 3.0)?
                        + 272.0 * f(x + h / 2.0)?
                        + 27.0 * f(x + 2.0 * h / 3.0)?
                        + 216.0 * f(x + 5.0 * h / 6.0)?
                        + 41.0 * f(x + h)?)
            }
        };
        x += h;
    }

    let result = sign * sum;
    if !result.is_finite() {
        return Err(CalcError::Overflow);
    }
    Ok(result)
}

/// Number of composite-Simpson sub-intervals to start the adaptive search
/// from, and how many times to double it before giving up. Mirrors the
/// same constants used by `functions::advanced`'s private integrator.
const ADAPTIVE_START_N: u64 = 64;
const ADAPTIVE_MAX_DOUBLINGS: u32 = 14;

/// Adaptively integrates `f` over `[a, b]` (in either order), doubling the
/// composite-Simpson sub-interval count until two successive estimates
/// agree within `tolerance`, or `ADAPTIVE_MAX_DOUBLINGS` is reached.
///
/// Stands in for the original's `int`/`gauss` (`Tegral`/`Gauss`: Hairer's
/// 30-point Gauss-Kronrod quadrature with Aitken extrapolation) — ported
/// "in spirit" per this project's established convention for
/// integration-based functions (see `docs/functions/advanced.md`) rather
/// than reproducing that specific algorithm's hardcoded coefficient
/// tables.
pub fn adaptive(
    name: &str,
    mut f: impl FnMut(f64) -> CalcResult<f64>,
    a: f64,
    b: f64,
    tolerance: f64,
) -> CalcResult<f64> {
    if a == b {
        return Ok(0.0);
    }
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(CalcError::DomainError(name.to_string()));
    }

    let (lo, hi, sign) = if a > b { (b, a, -1.0) } else { (a, b, 1.0) };

    let mut n = ADAPTIVE_START_N;
    let mut prev = simpson_pass(&mut f, lo, hi, n)?;
    for _ in 0..ADAPTIVE_MAX_DOUBLINGS {
        n *= 2;
        let cur = simpson_pass(&mut f, lo, hi, n)?;
        if (cur - prev).abs() < tolerance {
            let result = sign * cur;
            // Defensive only: `tolerance` is already checked finite above,
            // so `(cur - prev).abs() < tolerance` passing guarantees `cur`
            // (and thus `sign * cur`, a plain negation) is finite too - if
            // either were infinite or NaN, the comparison above would be
            // false and we'd never reach this branch. Kept as a guard
            // rather than an `unwrap`/assert, matching this project's
            // convention of erring on the side of a clean `Overflow` error
            // over a panic for defensive checks believed unreachable.
            return if result.is_finite() {
                Ok(result)
            } else {
                Err(CalcError::Overflow)
            };
        }
        prev = cur;
    }
    // Didn't converge to the requested tolerance within the refinement
    // budget above (e.g. the caller asked for an unreasonably tight
    // tolerance, or the integrand is discontinuous/highly oscillatory).
    Err(CalcError::Overflow)
}

fn simpson_pass(
    f: &mut impl FnMut(f64) -> CalcResult<f64>,
    a: f64,
    b: f64,
    n: u64,
) -> CalcResult<f64> {
    let h = (b - a) / n as f64;
    let mut sum = f(a)? + f(b)?;
    for i in 1..n {
        let x = a + i as f64 * h;
        sum += if i % 2 == 0 { 2.0 } else { 4.0 } * f(x)?;
    }
    Ok(sum * h / 3.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_square(x: f64) -> CalcResult<f64> {
        Ok(x * x)
    }

    #[test]
    fn rule_from_name_matches_aliases() {
        assert_eq!(Rule::from_name("trapezoid"), Some(Rule::Trapezoid));
        assert_eq!(Rule::from_name("trapez"), Some(Rule::Trapezoid));
        assert_eq!(Rule::from_name("trapezoide"), Some(Rule::Trapezoid));
        assert_eq!(Rule::from_name("simpson"), Some(Rule::Simpson));
        assert_eq!(Rule::from_name("newton"), Some(Rule::Newton));
        assert_eq!(Rule::from_name("boole"), Some(Rule::Boole));
        assert_eq!(Rule::from_name("ordersix"), Some(Rule::OrderSix));
        assert_eq!(Rule::from_name("ordresix"), Some(Rule::OrderSix));
        assert_eq!(Rule::from_name("weddle"), Some(Rule::Weddle));
        assert_eq!(Rule::from_name("nope"), None);
    }

    #[test]
    fn composite_integrates_x_squared() {
        // integral of x^2 from 0 to 3 = 9. Trapezoid converges much more
        // slowly than the higher-order rules for a fixed n, so it gets a
        // looser tolerance.
        for (rule, tolerance) in [
            (Rule::Trapezoid, 1e-3),
            (Rule::Simpson, 1e-6),
            (Rule::Newton, 1e-6),
            (Rule::Boole, 1e-6),
            (Rule::OrderSix, 1e-6),
            (Rule::Weddle, 1e-6),
        ] {
            let result = composite("test", rule, ok_square, 0.0, 3.0, 100.0).unwrap();
            assert!((result - 9.0).abs() < tolerance, "{rule:?} => {result}");
        }
    }

    #[test]
    fn composite_swaps_reversed_bounds() {
        let result = composite("simpson", Rule::Simpson, ok_square, 3.0, 0.0, 100.0).unwrap();
        assert!((result - -9.0).abs() < 1e-6);
    }

    #[test]
    fn composite_equal_bounds_is_zero() {
        assert_eq!(
            composite("simpson", Rule::Simpson, ok_square, 2.0, 2.0, 10.0),
            Ok(0.0)
        );
    }

    #[test]
    fn composite_rejects_non_whole_n() {
        assert_eq!(
            composite("simpson", Rule::Simpson, ok_square, 0.0, 1.0, 2.5),
            Err(CalcError::DomainError("simpson".to_string()))
        );
    }

    #[test]
    fn composite_rejects_zero_or_negative_n() {
        assert_eq!(
            composite("simpson", Rule::Simpson, ok_square, 0.0, 1.0, 0.0),
            Err(CalcError::DomainError("simpson".to_string()))
        );
        assert_eq!(
            composite("simpson", Rule::Simpson, ok_square, 0.0, 1.0, -4.0),
            Err(CalcError::DomainError("simpson".to_string()))
        );
    }

    #[test]
    fn composite_rejects_non_finite_n() {
        assert_eq!(
            composite("simpson", Rule::Simpson, ok_square, 0.0, 1.0, f64::NAN),
            Err(CalcError::DomainError("simpson".to_string()))
        );
    }

    #[test]
    fn composite_propagates_sample_errors() {
        let f = |x: f64| -> CalcResult<f64> {
            if x > 0.5 {
                Err(CalcError::DomainError("test".to_string()))
            } else {
                Ok(x)
            }
        };
        assert_eq!(
            composite("trapezoid", Rule::Trapezoid, f, 0.0, 1.0, 4.0),
            Err(CalcError::DomainError("test".to_string()))
        );
    }

    #[test]
    fn composite_overflow_on_infinite_result() {
        let f = |_: f64| -> CalcResult<f64> { Ok(f64::MAX) };
        assert_eq!(
            composite("trapezoid", Rule::Trapezoid, f, 0.0, 2.0, 2.0),
            Err(CalcError::Overflow)
        );
    }

    #[test]
    fn adaptive_integrates_x_squared() {
        let result = adaptive("int", ok_square, 0.0, 3.0, 1e-9).unwrap();
        assert!((result - 9.0).abs() < 1e-6);
    }

    #[test]
    fn adaptive_swaps_reversed_bounds() {
        let result = adaptive("gauss", ok_square, 3.0, 0.0, 1e-9).unwrap();
        assert!((result - -9.0).abs() < 1e-6);
    }

    #[test]
    fn adaptive_equal_bounds_is_zero() {
        assert_eq!(adaptive("int", ok_square, 5.0, 5.0, 1e-9), Ok(0.0));
    }

    #[test]
    fn adaptive_rejects_non_positive_tolerance() {
        assert_eq!(
            adaptive("int", ok_square, 0.0, 1.0, 0.0),
            Err(CalcError::DomainError("int".to_string()))
        );
        assert_eq!(
            adaptive("int", ok_square, 0.0, 1.0, -1e-6),
            Err(CalcError::DomainError("int".to_string()))
        );
    }

    #[test]
    fn adaptive_rejects_non_finite_tolerance() {
        assert_eq!(
            adaptive("int", ok_square, 0.0, 1.0, f64::NAN),
            Err(CalcError::DomainError("int".to_string()))
        );
    }

    #[test]
    fn adaptive_propagates_sample_errors() {
        let f = |x: f64| -> CalcResult<f64> {
            if x > 0.9 {
                Err(CalcError::DomainError("test".to_string()))
            } else {
                Ok(x)
            }
        };
        assert_eq!(
            adaptive("int", f, 0.0, 1.0, 1e-9),
            Err(CalcError::DomainError("test".to_string()))
        );
    }

    #[test]
    fn adaptive_overflow_when_it_does_not_converge() {
        // A wildly oscillating, effectively-random integrand never
        // stabilizes within the doubling budget at this tolerance.
        let f = |x: f64| -> CalcResult<f64> { Ok((x * 1e8).sin()) };
        assert_eq!(
            adaptive("int", f, 0.0, 1.0, 1e-15),
            Err(CalcError::Overflow)
        );
    }
}
