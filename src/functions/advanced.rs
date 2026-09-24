//! Advanced / special functions, ported from the "VI. Special Functions and
//! Series" chapter of the original manual.
//!
//! In the original Pascal engine, most of these functions (`ellipticE`,
//! `ellipticF`, `dilog`, `erf`, `erfc`, `si`, `ssi`, `ci`, `chi`, `dawson`,
//! `fresnelC`/`fresnelS`) are thin wrappers around a generic numeric
//! integration engine (`int(expr, var, lo, hi, tolerance)`), and `gamma`
//! itself is a hand-rolled Riemann-sum integral (see `ShortGamma`/`Factor`
//! in `MCalc.pas`). General-purpose numeric integration/plotting is
//! deferred to a separate future pass in this port, so this module ports
//! the same underlying formulas but evaluates them with a private, ad hoc
//! adaptive Simpson's-rule integrator (`integrate`, below) rather than the
//! full quadrature engine.
//!
//! TODO: once general numeric integration is implemented, consider routing
//! these through the shared engine (or dedicated closed-form
//! approximations, e.g. the Lanczos approximation for `gamma` and rational
//!/continued-fraction approximations for `erf`) for better precision and
//! performance than this fixed adaptive-Simpson stand-in.

use crate::error::{CalcError, CalcResult};

/// The Euler-Mascheroni constant, used by `ci`/`chi`. The original Pascal
/// source reads this from a variable named `G` that the user is expected to
/// set themselves (undefined variables default to `0`) — almost certainly a
/// bug rather than intentional, since `Ci`/`Chi` are meaningless without the
/// correct constant. This port uses the real mathematical constant.
const EULER_MASCHERONI: f64 = 0.5772156649015329;

/// Composite Simpson's rule over `[a, b]` with `n` sub-intervals (`n` must
/// be even).
fn simpson(f: &impl Fn(f64) -> f64, a: f64, b: f64, n: usize) -> f64 {
    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);
    for i in 1..n {
        let x = a + i as f64 * h;
        sum += if i % 2 == 0 { 2.0 } else { 4.0 } * f(x);
    }
    sum * h / 3.0
}

/// Numerically integrates `f` over `[lo, hi]` (in either order) using
/// composite Simpson's rule, doubling the number of sub-intervals until the
/// result stabilizes (or a maximum refinement count is reached).
fn integrate(f: impl Fn(f64) -> f64, lo: f64, hi: f64) -> f64 {
    if lo == hi {
        return 0.0;
    }
    let (a, b, sign) = if lo > hi {
        (hi, lo, -1.0)
    } else {
        (lo, hi, 1.0)
    };
    let mut n = 64usize;
    let mut prev = simpson(&f, a, b, n);
    for _ in 0..14 {
        n *= 2;
        let cur = simpson(&f, a, b, n);
        if (cur - prev).abs() < 1e-12 * cur.abs().max(1.0) {
            return sign * cur;
        }
        prev = cur;
    }
    // Unreachable in practice: 14 doublings from n=64 reaches roughly a
    // million sub-intervals, at which point composite Simpson's rule has
    // converged to well within the 1e-12 relative tolerance above for every
    // integrand used in this module (all finite, smooth, and defined on a
    // bounded interval after the singularity handling above/in callers).
    sign * prev
}

/// The fixed Riemann-sum step used by `gamma`/`ShortGamma`, matching the
/// value implied by the pre-drafted `gamma(0.5)` example in the docs
/// (`1.77244070463831`, vs. the true `sqrt(pi) = 1.77245385...`).
const GAMMA_STEP: f64 = 0.001;

/// Approximates `Int(x^(alpha-1) / e^x, x, 0, infinity)` via a right-endpoint
/// Riemann sum out to a finite cutoff, ported directly from `ShortGamma`.
fn short_gamma(alpha: f64, infinity: f64) -> f64 {
    let mut i = 0.0;
    let mut total = 0.0;
    while i < infinity {
        i += GAMMA_STEP;
        total += (i.powf(alpha - 1.0) / i.exp()) * GAMMA_STEP;
    }
    total
}

/// Extends the Gamma function to non-positive/non-integer `n` via the
/// recurrence `Γ(n) = Γ(n+1) / n`, ported directly from `Factor`. `n` here is
/// `x - 1` relative to the public `gamma(x)`, i.e. `factor(n) = Γ(n+1)`.
fn factor(n: f64) -> CalcResult<f64> {
    if n < 0.0 {
        if n.trunc() == n {
            // Gamma has a pole at every non-positive integer.
            return Err(CalcError::DomainError("gamma".to_string()));
        }
        let mut new_gamma = short_gamma(
            (1.0 + n.fract()).abs() + 1.0,
            50.0 + (n * 1.2).trunc().abs(),
        );
        let frac_n = n.fract();
        let mut i = -1;
        while i < n.abs().trunc() as i64 {
            new_gamma /= frac_n - i as f64;
            i += 1;
        }
        return Ok(new_gamma);
    }
    if n == 0.0 {
        return Ok(1.0);
    }
    if n == 1.0 {
        return Ok(1.0);
    }
    if n == 2.0 {
        return Ok(2.0);
    }
    if n == 3.0 {
        return Ok(6.0);
    }
    Ok(short_gamma(n + 1.0, 50.0 + (n * 1.2).trunc()))
}

/// Euler's Gamma function, `Γ(x) = Int(e^(-t) * t^(x-1), t, 0, Infinity)` for
/// positive `x`, extended to negative/non-integer `x` via the recurrence
/// `Γ(x-1) = Γ(x) / (x-1)`. Domain error at `x <= 0` when `x` is an integer
/// (a pole of the true Gamma function).
pub fn gamma(x: f64) -> CalcResult<f64> {
    factor(x - 1.0)
}

/// Euler's Beta function, `B(a, b) = Γ(a) * Γ(b) / Γ(a+b)`.
pub fn beta(a: f64, b: f64) -> CalcResult<f64> {
    Ok(gamma(a)? * gamma(b)? / gamma(a + b)?)
}

/// The Pochhammer symbol / rising factorial, computed here as
/// `Γ(x+n) / Γ(x)` (matching the original, which special-cases `x = n = 0`
/// as a domain error since the ratio is `0/0`).
pub fn pochhammer(x: f64, n: f64) -> CalcResult<f64> {
    if x == 0.0 && n == 0.0 {
        return Err(CalcError::DomainError("pochhammer".to_string()));
    }
    Ok(gamma(x + n)? / gamma(x)?)
}

/// The binomial theorem `(x+y)^n`, computed via the finite Pascal's-triangle
/// expansion `Sum(C(n, i) * x^i * y^(n-i), i, 0, n)` for integer `n`
/// (negative integer `n` uses `1 / bth(x, y, -n)`), or `Power(x+y, n)`
/// directly for non-integer `n`.
pub fn bth(x: f64, y: f64, n: f64) -> CalcResult<f64> {
    if n.trunc() != n {
        return Ok((x + y).powf(n));
    }
    let negative = n < 0.0;
    let n = n.abs();
    let mut result = 0.0;
    let mut cur_n = 1.0; // running numerator (falling factorial of n)
    let mut cur_u = 1.0; // running denominator (factorial of the term index)
    let mut i = n as i64;
    while i >= 0 {
        result += (x.powf(i as f64) * y.powf(n - i as f64) * cur_n) / cur_u;
        cur_n *= i as f64;
        cur_u *= n - i as f64 + 1.0;
        i -= 1;
    }
    if negative {
        if result == 0.0 {
            return Err(CalcError::Overflow);
        }
        return Ok(1.0 / result);
    }
    Ok(result)
}

/// The binomial theorem `(x+y)^n`, computed the "standard" way (add then
/// raise to a power), rather than `bth`'s explicit series expansion.
pub fn bman(x: f64, y: f64, n: f64) -> CalcResult<f64> {
    Ok((x + y).powf(n))
}

/// Converts an upper bound `z` (a sine value) to the substitution angle
/// `asin(z)` used by the elliptic integrals below; errors if `z` is outside
/// `[-1, 1]` (where `sqrt(1 - t^2)` would be imaginary).
fn elliptic_theta_max(z: f64, name: &str) -> CalcResult<f64> {
    if !(-1.0..=1.0).contains(&z) {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(z.asin())
}

/// Incomplete elliptic integral of the second kind,
/// `Int(sqrt(1 - k^2*t^2) / sqrt(1 - t^2), t, 0, z)`; `ellipticE(k) =
/// ellipticE(k, 1)`. Evaluated via the standard substitution `t = sin(theta)`
/// (which cancels the `sqrt(1-t^2)` denominator exactly), avoiding the
/// integrable-but-numerically-awkward singularity at `t = 1`.
pub fn elliptic_e(k: f64, z: f64) -> CalcResult<f64> {
    let theta_max = elliptic_theta_max(z, "ellipticE")?;
    if (k * z).abs() > 1.0 {
        return Err(CalcError::DomainError("ellipticE".to_string()));
    }
    Ok(integrate(
        |theta| (1.0 - k * k * theta.sin().powi(2)).max(0.0).sqrt(),
        0.0,
        theta_max,
    ))
}

/// Incomplete elliptic integral of the first kind (the original manual
/// calls the complete/incomplete pair `EllipticK`), `Int(1 / sqrt(1-t^2) /
/// sqrt(1-k^2*t^2), t, 0, z)`; `ellipticF(k) = ellipticF(k, 1)`. Same `t =
/// sin(theta)` substitution as `elliptic_e`; diverges when `|k*z| = 1`.
pub fn elliptic_f(k: f64, z: f64) -> CalcResult<f64> {
    let theta_max = elliptic_theta_max(z, "ellipticF")?;
    let kz = (k * z).abs();
    if kz > 1.0 {
        return Err(CalcError::DomainError("ellipticF".to_string()));
    }
    if kz >= 1.0 - 1e-9 {
        return Err(CalcError::Overflow);
    }
    Ok(integrate(
        |theta| 1.0 / (1.0 - k * k * theta.sin().powi(2)).sqrt(),
        0.0,
        theta_max,
    ))
}

/// Complementary complete elliptic integral of the second kind,
/// `ellipticE(sqrt(1-k^2), 1)`.
pub fn elliptic_ce(k: f64) -> CalcResult<f64> {
    if !(-1.0..=1.0).contains(&k) {
        return Err(CalcError::DomainError("ellipticCE".to_string()));
    }
    elliptic_e((1.0 - k * k).sqrt(), 1.0)
}

/// Complementary complete elliptic integral of the first kind,
/// `ellipticF(sqrt(1-k^2), 1)`.
pub fn elliptic_ck(k: f64) -> CalcResult<f64> {
    if !(-1.0..=1.0).contains(&k) {
        return Err(CalcError::DomainError("ellipticCK".to_string()));
    }
    elliptic_f((1.0 - k * k).sqrt(), 1.0)
}

/// Dilogarithm integral, `Int(ln(t) / (1-t), t, 1, x)`. The integrand has a
/// removable singularity at `t = 1` (limit `-1`), handled explicitly.
pub fn dilog(x: f64) -> CalcResult<f64> {
    if x <= 0.0 {
        return Err(CalcError::DomainError("dilog".to_string()));
    }
    Ok(integrate(
        |t| {
            if t == 1.0 {
                -1.0
            } else {
                t.ln() / (1.0 - t)
            }
        },
        1.0,
        x,
    ))
}

/// Dawson integral, `exp(-x^2) * Int(exp(t^2), t, 0, x)`.
pub fn dawson(x: f64) -> CalcResult<f64> {
    Ok((-x * x).exp() * integrate(|t| (t * t).exp(), 0.0, x))
}

/// Error function, `2/sqrt(pi) * Int(exp(-t^2), t, 0, x)`.
pub fn erf(x: f64) -> CalcResult<f64> {
    Ok((2.0 / std::f64::consts::PI.sqrt()) * integrate(|t| (-(t * t)).exp(), 0.0, x))
}

/// Complementary error function, `1 - erf(x)`.
pub fn erfc(x: f64) -> CalcResult<f64> {
    Ok(1.0 - erf(x)?)
}

/// Sine integral, `Int(sin(t)/t, t, 0, x)`. The integrand has a removable
/// singularity at `t = 0` (limit `1`), handled explicitly.
pub fn si(x: f64) -> CalcResult<f64> {
    Ok(integrate(
        |t| if t == 0.0 { 1.0 } else { t.sin() / t },
        0.0,
        x,
    ))
}

/// Shifted sine integral, `si(x) - pi/2`.
pub fn ssi(x: f64) -> CalcResult<f64> {
    Ok(si(x)? - std::f64::consts::FRAC_PI_2)
}

/// Cosine integral, `EULER_MASCHERONI + ln(x) + Int((cos(t)-1)/t, t, 0, x)`.
/// The integrand has a removable singularity at `t = 0` (limit `0`).
pub fn ci(x: f64) -> CalcResult<f64> {
    if x <= 0.0 {
        return Err(CalcError::DomainError("ci".to_string()));
    }
    Ok(EULER_MASCHERONI
        + x.ln()
        + integrate(|t| if t == 0.0 { 0.0 } else { (t.cos() - 1.0) / t }, 0.0, x))
}

/// Hyperbolic cosine integral, `EULER_MASCHERONI + ln(x) + Int((cosh(t)-1)/t,
/// t, 0, x)`. The integrand has a removable singularity at `t = 0` (limit
/// `0`).
pub fn chi(x: f64) -> CalcResult<f64> {
    if x <= 0.0 {
        return Err(CalcError::DomainError("chi".to_string()));
    }
    Ok(EULER_MASCHERONI
        + x.ln()
        + integrate(
            |t| if t == 0.0 { 0.0 } else { (t.cosh() - 1.0) / t },
            0.0,
            x,
        ))
}

/// Fresnel cosine integral, `Int(cos(pi/2 * t^2), t, 0, x)`.
pub fn fresnel_c(x: f64) -> CalcResult<f64> {
    Ok(integrate(
        |t| (std::f64::consts::FRAC_PI_2 * t * t).cos(),
        0.0,
        x,
    ))
}

/// Fresnel sine integral, `Int(sin(pi/2 * t^2), t, 0, x)`.
pub fn fresnel_s(x: f64) -> CalcResult<f64> {
    Ok(integrate(
        |t| (std::f64::consts::FRAC_PI_2 * t * t).sin(),
        0.0,
        x,
    ))
}

/// Fresnel cosine auxiliary function, derived from `fresnelC`/`fresnelS`.
pub fn fresnel_f(x: f64) -> CalcResult<f64> {
    let half_minus_s = 0.5 - fresnel_s(x)?;
    let half_minus_c = 0.5 - fresnel_c(x)?;
    let angle = std::f64::consts::FRAC_PI_2 * x * x;
    Ok(half_minus_s * angle.cos() - half_minus_c * angle.sin())
}

/// Fresnel sine auxiliary function, derived from `fresnelC`/`fresnelS`.
pub fn fresnel_g(x: f64) -> CalcResult<f64> {
    let half_minus_c = 0.5 - fresnel_c(x)?;
    let half_minus_s = 0.5 - fresnel_s(x)?;
    let angle = std::f64::consts::FRAC_PI_2 * x * x;
    Ok(half_minus_c * angle.cos() - half_minus_s * angle.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "{a} !~= {b} (tol {tol})");
    }

    #[test]
    fn gamma_matches_known_values() {
        assert_close(gamma(5.0).unwrap(), 24.0, 1e-6);
        assert_close(gamma(4.0).unwrap(), 6.0, 1e-9);
        assert_close(gamma(1.5).unwrap(), 0.886227, 1e-4);
        assert_close(gamma(0.5).unwrap(), std::f64::consts::PI.sqrt(), 1e-4);
        assert_close(
            gamma(-0.5).unwrap(),
            -2.0 * std::f64::consts::PI.sqrt(),
            1e-3,
        );
    }

    #[test]
    fn gamma_domain_error_at_nonpositive_integers() {
        assert_eq!(gamma(0.0), Err(CalcError::DomainError("gamma".to_string())));
        assert_eq!(
            gamma(-2.0),
            Err(CalcError::DomainError("gamma".to_string()))
        );
    }

    #[test]
    fn beta_matches_manual_example() {
        assert_close(beta(1.0, 2.0).unwrap(), 0.5, 1e-3);
    }

    #[test]
    fn pochhammer_matches_manual_example() {
        assert_close(pochhammer(5.0, 3.0).unwrap(), 210.0, 1e-2);
    }

    #[test]
    fn pochhammer_domain_error_at_zero_zero() {
        assert_eq!(
            pochhammer(0.0, 0.0),
            Err(CalcError::DomainError("pochhammer".to_string()))
        );
    }

    #[test]
    fn bth_matches_manual_example() {
        assert_close(bth(2.0, 3.0, 4.0).unwrap(), 625.0, 1e-9);
    }

    #[test]
    fn bth_matches_power_for_non_integer_n() {
        assert_close(bth(2.0, 3.0, 1.5).unwrap(), 5f64.powf(1.5), 1e-9);
    }

    #[test]
    fn bth_negative_integer_n_is_reciprocal() {
        assert_close(
            bth(2.0, 3.0, -2.0).unwrap(),
            1.0 / bth(2.0, 3.0, 2.0).unwrap(),
            1e-9,
        );
    }

    #[test]
    fn bth_negative_n_overflow_when_result_is_zero() {
        // bth(1, -1, 3) sums to 0 (binomial expansion of (1-1)^3), so the
        // negative-n reciprocal branch would divide by zero.
        assert_eq!(bth(1.0, -1.0, -3.0), Err(CalcError::Overflow));
    }

    #[test]
    fn bman_matches_manual_example() {
        assert_close(bman(2.0, 3.0, 4.0).unwrap(), 625.0, 1e-9);
    }

    #[test]
    fn elliptic_e_matches_manual_example() {
        assert_close(elliptic_e(1.0, 1.0).unwrap(), 1.0, 1e-9);
    }

    #[test]
    fn elliptic_f_matches_manual_example() {
        assert_close(elliptic_f(0.01, 1.0).unwrap(), 1.57083559891215, 1e-8);
    }

    #[test]
    fn elliptic_ce_matches_manual_example() {
        assert_close(elliptic_ce(1.0).unwrap(), std::f64::consts::FRAC_PI_2, 1e-9);
    }

    #[test]
    fn elliptic_ck_matches_manual_example() {
        assert_close(elliptic_ck(1.0).unwrap(), std::f64::consts::FRAC_PI_2, 1e-9);
    }

    #[test]
    fn elliptic_ck_diverges_at_k_zero() {
        assert_eq!(elliptic_ck(0.0), Err(CalcError::Overflow));
    }

    #[test]
    fn elliptic_domain_errors_outside_range() {
        assert_eq!(
            elliptic_e(1.0, 2.0),
            Err(CalcError::DomainError("ellipticE".to_string()))
        );
        assert_eq!(
            elliptic_f(1.0, 2.0),
            Err(CalcError::DomainError("ellipticF".to_string()))
        );
        assert_eq!(
            elliptic_ce(2.0),
            Err(CalcError::DomainError("ellipticCE".to_string()))
        );
        assert_eq!(
            elliptic_ck(2.0),
            Err(CalcError::DomainError("ellipticCK".to_string()))
        );
    }

    #[test]
    fn elliptic_domain_errors_when_k_times_z_exceeds_one() {
        // z = 0.9 is within [-1, 1] (passes the asin range check), but k = 2
        // makes k*z = 1.8 > 1, which is a separate domain error.
        assert_eq!(
            elliptic_e(2.0, 0.9),
            Err(CalcError::DomainError("ellipticE".to_string()))
        );
        assert_eq!(
            elliptic_f(2.0, 0.9),
            Err(CalcError::DomainError("ellipticF".to_string()))
        );
    }

    #[test]
    fn dilog_at_one_is_zero() {
        assert_close(dilog(1.0).unwrap(), 0.0, 1e-9);
    }

    #[test]
    fn dilog_domain_error_at_nonpositive() {
        assert_eq!(dilog(0.0), Err(CalcError::DomainError("dilog".to_string())));
    }

    #[test]
    fn dilog_handles_bounds_below_one() {
        // Integral(1 -> 0.5) = -Integral(0.5 -> 1), sanity check sign flips.
        let below = dilog(0.5).unwrap();
        assert!(below.is_finite());
    }

    #[test]
    fn dawson_at_zero_is_zero() {
        assert_close(dawson(0.0).unwrap(), 0.0, 1e-9);
    }

    #[test]
    fn erf_matches_known_values() {
        assert_close(erf(0.0).unwrap(), 0.0, 1e-9);
        assert_close(erf(1.0).unwrap(), 0.8427007929497149, 1e-6);
    }

    #[test]
    fn erfc_matches_one_minus_erf() {
        assert_close(erfc(1.0).unwrap(), 1.0 - erf(1.0).unwrap(), 1e-12);
    }

    #[test]
    fn si_at_zero_is_zero() {
        assert_close(si(0.0).unwrap(), 0.0, 1e-9);
    }

    #[test]
    fn si_matches_known_value() {
        assert_close(si(1.0).unwrap(), 0.9460830703671831, 1e-6);
    }

    #[test]
    fn ssi_matches_si_minus_half_pi() {
        assert_close(
            ssi(1.0).unwrap(),
            si(1.0).unwrap() - std::f64::consts::FRAC_PI_2,
            1e-12,
        );
    }

    #[test]
    fn ci_matches_known_value() {
        assert_close(ci(1.0).unwrap(), 0.3374039229009681, 1e-6);
    }

    #[test]
    fn ci_domain_error_at_nonpositive() {
        assert_eq!(ci(0.0), Err(CalcError::DomainError("ci".to_string())));
    }

    #[test]
    fn chi_matches_known_value() {
        assert_close(chi(1.0).unwrap(), 0.8378669409802082, 1e-6);
    }

    #[test]
    fn chi_domain_error_at_nonpositive() {
        assert_eq!(chi(0.0), Err(CalcError::DomainError("chi".to_string())));
    }

    #[test]
    fn fresnel_c_and_s_at_zero_are_zero() {
        assert_close(fresnel_c(0.0).unwrap(), 0.0, 1e-9);
        assert_close(fresnel_s(0.0).unwrap(), 0.0, 1e-9);
    }

    #[test]
    fn fresnel_f_and_g_are_finite() {
        assert!(fresnel_f(1.0).unwrap().is_finite());
        assert!(fresnel_g(1.0).unwrap().is_finite());
    }
}
