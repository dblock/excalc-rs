//! Probability distribution functions: binomial and Poisson pmf/cdf, and
//! normal pdf/cdf, plus the `zscore` standardization helper.

use crate::error::{CalcError, CalcResult};
use crate::functions::advanced::erf;
use crate::functions::stats::binom;

/// Validates a probability `p` is in `[0, 1]`, erroring with
/// `DomainError(name)` otherwise.
fn check_probability(name: &str, p: f64) -> CalcResult<()> {
    if !(0.0..=1.0).contains(&p) {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(())
}

/// Validates a non-negative integer "count" argument (used for `k`),
/// erroring with `DomainError(name)` otherwise.
fn check_count(name: &str, x: f64) -> CalcResult<u64> {
    if x < 0.0 || x.fract() != 0.0 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(x as u64)
}

/// Binomial probability mass function: `P(X = k)` for `X ~ Binomial(n, p)`,
/// i.e. `C(n, k) * p^k * (1-p)^(n-k)`.
pub fn binomialpdf(n: f64, p: f64, k: f64) -> CalcResult<f64> {
    check_probability("binomialpdf", p)?;
    check_count("binomialpdf", k)?;
    if k > n {
        return Err(CalcError::DomainError("binomialpdf".to_string()));
    }
    let c = binom(n, k)?;
    Ok(c * p.powf(k) * (1.0 - p).powf(n - k))
}

/// Binomial cumulative distribution function: `P(X <= k)` for
/// `X ~ Binomial(n, p)`, summing `binomialpdf` from `0` to `k`.
pub fn binomialcdf(n: f64, p: f64, k: f64) -> CalcResult<f64> {
    let k = check_count("binomialcdf", k)?;
    let mut total = 0.0;
    for i in 0..=k {
        total += binomialpdf(n, p, i as f64)?;
    }
    Ok(total)
}

/// Poisson probability mass function: `P(X = k)` for `X ~ Poisson(lambda)`,
/// i.e. `e^(-lambda) * lambda^k / k!`.
pub fn poissonpdf(lambda: f64, k: f64) -> CalcResult<f64> {
    if lambda < 0.0 {
        return Err(CalcError::DomainError("poissonpdf".to_string()));
    }
    let k = check_count("poissonpdf", k)?;
    let mut term = (-lambda).exp();
    for i in 1..=k {
        term *= lambda / i as f64;
    }
    Ok(term)
}

/// Poisson cumulative distribution function: `P(X <= k)` for
/// `X ~ Poisson(lambda)`, summing `poissonpdf` from `0` to `k`.
pub fn poissoncdf(lambda: f64, k: f64) -> CalcResult<f64> {
    let k = check_count("poissoncdf", k)?;
    let mut total = 0.0;
    for i in 0..=k {
        total += poissonpdf(lambda, i as f64)?;
    }
    Ok(total)
}

/// Normal (Gaussian) probability density function at `x`, for
/// `X ~ Normal(mean, sd)`. Note this is a density, not a probability, and
/// may exceed `1` for small `sd`.
pub fn normalpdf(x: f64, mean: f64, sd: f64) -> CalcResult<f64> {
    if sd <= 0.0 {
        return Err(CalcError::DomainError("normalpdf".to_string()));
    }
    let z = (x - mean) / sd;
    Ok((-0.5 * z * z).exp() / (sd * (2.0 * std::f64::consts::PI).sqrt()))
}

/// Normal (Gaussian) cumulative distribution function: `P(X <= x)` for
/// `X ~ Normal(mean, sd)`, via `Phi(x) = 0.5 * (1 + erf((x-mean)/(sd*sqrt(2))))`.
pub fn normalcdf(x: f64, mean: f64, sd: f64) -> CalcResult<f64> {
    if sd <= 0.0 {
        return Err(CalcError::DomainError("normalcdf".to_string()));
    }
    let z = (x - mean) / (sd * std::f64::consts::SQRT_2);
    Ok(0.5 * (1.0 + erf(z)?))
}

/// Standard score: how many standard deviations `x` is from `mean`,
/// `(x - mean) / sd`.
pub fn zscore(x: f64, mean: f64, sd: f64) -> CalcResult<f64> {
    if sd <= 0.0 {
        return Err(CalcError::DomainError("zscore".to_string()));
    }
    Ok((x - mean) / sd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binomialpdf_matches_known_value() {
        // P(exactly 5 heads in 10 fair coin flips).
        let result = binomialpdf(10.0, 0.5, 5.0).unwrap();
        assert!((result - 0.24609375).abs() < 1e-9);
    }

    #[test]
    fn binomialpdf_rejects_bad_inputs() {
        assert!(matches!(
            binomialpdf(10.0, 1.5, 5.0),
            Err(CalcError::DomainError(_))
        ));
        assert!(matches!(
            binomialpdf(10.0, 0.5, 11.0),
            Err(CalcError::DomainError(_))
        ));
        assert!(matches!(
            binomialpdf(10.0, 0.5, 2.5),
            Err(CalcError::DomainError(_))
        ));
    }

    #[test]
    fn binomialcdf_matches_known_value() {
        let result = binomialcdf(10.0, 0.5, 5.0).unwrap();
        assert!((result - 0.623046875).abs() < 1e-9);
    }

    #[test]
    fn binomialcdf_at_n_is_one() {
        let result = binomialcdf(10.0, 0.5, 10.0).unwrap();
        assert!((result - 1.0).abs() < 1e-9);
    }

    #[test]
    fn poissonpdf_matches_known_value() {
        // Direct check against the closed form: e^-4 * 4^2 / 2! = 8 e^-4.
        let result = poissonpdf(4.0, 2.0).unwrap();
        let expected = 8.0 * (-4f64).exp();
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn poissonpdf_rejects_negative_lambda_or_fractional_k() {
        assert!(matches!(
            poissonpdf(-1.0, 2.0),
            Err(CalcError::DomainError(_))
        ));
        assert!(matches!(
            poissonpdf(4.0, 2.5),
            Err(CalcError::DomainError(_))
        ));
    }

    #[test]
    fn poissoncdf_matches_sum_of_pdfs() {
        let expected: f64 = (0..=3).map(|k| poissonpdf(4.0, k as f64).unwrap()).sum();
        assert_eq!(poissoncdf(4.0, 3.0).unwrap(), expected);
    }

    #[test]
    fn normalpdf_matches_known_value_at_mean() {
        let result = normalpdf(0.0, 0.0, 1.0).unwrap();
        let expected = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
        assert!((result - expected).abs() < 1e-12);
    }

    #[test]
    fn normalpdf_rejects_non_positive_sd() {
        assert!(matches!(
            normalpdf(0.0, 0.0, 0.0),
            Err(CalcError::DomainError(_))
        ));
    }

    #[test]
    fn normalcdf_matches_known_value_at_mean() {
        let result = normalcdf(0.0, 0.0, 1.0).unwrap();
        assert!((result - 0.5).abs() < 1e-9);
    }

    #[test]
    fn normalcdf_matches_known_95_percent_value() {
        let result = normalcdf(1.959963985, 0.0, 1.0).unwrap();
        assert!((result - 0.975).abs() < 1e-6);
    }

    #[test]
    fn zscore_matches_known_value() {
        assert_eq!(zscore(85.0, 70.0, 10.0).unwrap(), 1.5);
    }

    #[test]
    fn zscore_rejects_non_positive_sd() {
        assert!(matches!(
            zscore(1.0, 0.0, 0.0),
            Err(CalcError::DomainError(_))
        ));
    }
}
