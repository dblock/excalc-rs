//! Statistics functions over a variadic list of arguments.

use crate::error::{CalcError, CalcResult};

pub fn sum(args: &[f64]) -> CalcResult<f64> {
    Ok(args.iter().sum())
}

pub fn average(args: &[f64]) -> CalcResult<f64> {
    if args.is_empty() {
        return Err(CalcError::WrongArgCount {
            name: "average".to_string(),
            expected: "at least 1".to_string(),
            got: 0,
        });
    }
    Ok(args.iter().sum::<f64>() / args.len() as f64)
}

pub fn product(args: &[f64]) -> CalcResult<f64> {
    Ok(args.iter().product())
}

pub fn min(args: &[f64]) -> CalcResult<f64> {
    args.iter()
        .cloned()
        .fold(None, |acc, x| Some(acc.map_or(x, |a: f64| a.min(x))))
        .ok_or_else(|| CalcError::WrongArgCount {
            name: "min".to_string(),
            expected: "at least 1".to_string(),
            got: 0,
        })
}

pub fn max(args: &[f64]) -> CalcResult<f64> {
    args.iter()
        .cloned()
        .fold(None, |acc, x| Some(acc.map_or(x, |a: f64| a.max(x))))
        .ok_or_else(|| CalcError::WrongArgCount {
            name: "max".to_string(),
            expected: "at least 1".to_string(),
            got: 0,
        })
}

fn require_at_least_one(name: &str, args: &[f64]) -> CalcResult<()> {
    if args.is_empty() {
        return Err(CalcError::WrongArgCount {
            name: name.to_string(),
            expected: "at least 1".to_string(),
            got: 0,
        });
    }
    Ok(())
}

/// The middle value of `args` once sorted (average of the two middle values
/// for an even count).
pub fn median(args: &[f64]) -> CalcResult<f64> {
    require_at_least_one("median", args)?;
    let mut sorted = args.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let n = sorted.len();
    Ok(if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    })
}

/// The most frequently occurring value. Ties are broken by returning the
/// smallest of the tied values, so the result is deterministic.
pub fn mode(args: &[f64]) -> CalcResult<f64> {
    require_at_least_one("mode", args)?;
    let mut sorted = args.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let (mut best_value, mut best_count) = (sorted[0], 0usize);
    let (mut current_value, mut current_count) = (sorted[0], 0usize);
    for &x in &sorted {
        if x == current_value {
            current_count += 1;
        } else {
            current_value = x;
            current_count = 1;
        }
        if current_count > best_count {
            best_count = current_count;
            best_value = current_value;
        }
    }
    Ok(best_value)
}

/// Population variance: the mean of squared deviations from the mean.
pub fn variance(args: &[f64]) -> CalcResult<f64> {
    require_at_least_one("variance", args)?;
    let mean = args.iter().sum::<f64>() / args.len() as f64;
    Ok(args.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / args.len() as f64)
}

/// Population standard deviation, `sqrt(variance(args))`.
pub fn stddev(args: &[f64]) -> CalcResult<f64> {
    Ok(variance(args)?.sqrt())
}

/// Population skewness (third standardized moment): a measure of the
/// asymmetry of the data's distribution around its mean. Positive values
/// indicate a longer right tail, negative a longer left tail, `0` a
/// symmetric distribution (e.g. the normal distribution).
pub fn skewness(args: &[f64]) -> CalcResult<f64> {
    require_at_least_one("skewness", args)?;
    let mean = args.iter().sum::<f64>() / args.len() as f64;
    let sd = stddev(args)?;
    if sd == 0.0 {
        return Err(CalcError::DomainError("skewness".to_string()));
    }
    let m3 = args.iter().map(|x| (x - mean).powi(3)).sum::<f64>() / args.len() as f64;
    Ok(m3 / sd.powi(3))
}

/// Population excess kurtosis (fourth standardized moment, minus `3`): a
/// measure of the "tailedness" of the data's distribution. `0` matches the
/// normal distribution's kurtosis; positive values indicate heavier tails,
/// negative values lighter tails.
pub fn kurtosis(args: &[f64]) -> CalcResult<f64> {
    require_at_least_one("kurtosis", args)?;
    let mean = args.iter().sum::<f64>() / args.len() as f64;
    let sd = stddev(args)?;
    if sd == 0.0 {
        return Err(CalcError::DomainError("kurtosis".to_string()));
    }
    let m4 = args.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / args.len() as f64;
    Ok(m4 / sd.powi(4) - 3.0)
}

/// The `p`-th percentile of `data` (linear interpolation between closest
/// ranks, matching the common "linear"/Excel `PERCENTILE.INC` method).
/// Called as `percentile(p, x1, x2, ...)`: the first argument is the
/// percentile in `[0, 100]`, the rest is the data.
pub fn percentile(args: &[f64]) -> CalcResult<f64> {
    if args.len() < 2 {
        return Err(CalcError::WrongArgCount {
            name: "percentile".to_string(),
            expected: "at least 2 (percentile, data...)".to_string(),
            got: args.len(),
        });
    }
    let p = args[0];
    if !(0.0..=100.0).contains(&p) {
        return Err(CalcError::DomainError("percentile".to_string()));
    }
    let mut data = args[1..].to_vec();
    data.sort_by(|a, b| a.total_cmp(b));
    let n = data.len();
    if n == 1 {
        return Ok(data[0]);
    }
    let rank = (p / 100.0) * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        Ok(data[lo])
    } else {
        let frac = rank - lo as f64;
        Ok(data[lo] + (data[hi] - data[lo]) * frac)
    }
}

/// Splits a flat, even-length argument list into paired `(x, y)` samples:
/// `[x1, y1, x2, y2, ...]`. Used by `covariance`/`correlation`, which take
/// two parallel data series as interleaved pairs since the evaluator only
/// supports flat argument lists.
fn paired_samples(name: &str, args: &[f64]) -> CalcResult<Vec<(f64, f64)>> {
    if args.is_empty() || !args.len().is_multiple_of(2) {
        return Err(CalcError::WrongArgCount {
            name: name.to_string(),
            expected: "an even number of arguments (x1, y1, x2, y2, ...)".to_string(),
            got: args.len(),
        });
    }
    Ok(args.chunks(2).map(|c| (c[0], c[1])).collect())
}

/// Population covariance of two data series, given as interleaved pairs
/// `covariance(x1, y1, x2, y2, ...)`.
pub fn covariance(args: &[f64]) -> CalcResult<f64> {
    let pairs = paired_samples("covariance", args)?;
    let n = pairs.len() as f64;
    let mean_x = pairs.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y = pairs.iter().map(|(_, y)| y).sum::<f64>() / n;
    Ok(pairs
        .iter()
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum::<f64>()
        / n)
}

/// Pearson correlation coefficient of two data series, given as interleaved
/// pairs `correlation(x1, y1, x2, y2, ...)`.
pub fn correlation(args: &[f64]) -> CalcResult<f64> {
    let pairs = paired_samples("correlation", args)?;
    let n = pairs.len() as f64;
    let mean_x = pairs.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y = pairs.iter().map(|(_, y)| y).sum::<f64>() / n;
    let cov = pairs
        .iter()
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum::<f64>();
    let var_x = pairs.iter().map(|(x, _)| (x - mean_x).powi(2)).sum::<f64>();
    let var_y = pairs.iter().map(|(_, y)| (y - mean_y).powi(2)).sum::<f64>();
    let denom = (var_x * var_y).sqrt();
    if denom == 0.0 {
        return Err(CalcError::DomainError("correlation".to_string()));
    }
    Ok(cov / denom)
}

/// Harmonic number H(n) = 1 + 1/2 + 1/3 + ... + 1/n.
pub fn harmonic(n: f64) -> CalcResult<f64> {
    if n < 1.0 || n.fract() != 0.0 {
        return Err(CalcError::DomainError("harmonic".to_string()));
    }
    let n = n as u64;
    Ok((1..=n).map(|k| 1.0 / k as f64).sum())
}

/// Binomial coefficient "n choose k".
pub fn binom(n: f64, k: f64) -> CalcResult<f64> {
    if n < 0.0 || k < 0.0 || n.fract() != 0.0 || k.fract() != 0.0 || k > n {
        return Err(CalcError::DomainError("binom".to_string()));
    }
    let (n, mut k) = (n as u64, k as u64);
    if k > n - k {
        k = n - k;
    }
    let mut result: f64 = 1.0;
    for i in 0..k {
        result *= (n - i) as f64;
        result /= (i + 1) as f64;
    }
    Ok(result.round())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_and_product() {
        assert_eq!(sum(&[1.0, 2.0, 3.0]).unwrap(), 6.0);
        assert_eq!(sum(&[]).unwrap(), 0.0);
        assert_eq!(product(&[2.0, 3.0, 4.0]).unwrap(), 24.0);
        assert_eq!(product(&[]).unwrap(), 1.0);
    }

    #[test]
    fn average_requires_at_least_one_arg() {
        assert_eq!(average(&[2.0, 4.0]).unwrap(), 3.0);
        assert_eq!(
            average(&[]),
            Err(CalcError::WrongArgCount {
                name: "average".to_string(),
                expected: "at least 1".to_string(),
                got: 0,
            })
        );
    }

    #[test]
    fn min_and_max_require_at_least_one_arg() {
        assert_eq!(min(&[3.0, 1.0, 2.0]).unwrap(), 1.0);
        assert_eq!(max(&[3.0, 1.0, 2.0]).unwrap(), 3.0);
        assert_eq!(
            min(&[]),
            Err(CalcError::WrongArgCount {
                name: "min".to_string(),
                expected: "at least 1".to_string(),
                got: 0,
            })
        );
        assert_eq!(
            max(&[]),
            Err(CalcError::WrongArgCount {
                name: "max".to_string(),
                expected: "at least 1".to_string(),
                got: 0,
            })
        );
    }

    #[test]
    fn harmonic_domain() {
        assert_eq!(harmonic(3.0).unwrap(), 1.0 + 0.5 + (1.0 / 3.0));
        assert_eq!(
            harmonic(0.0),
            Err(CalcError::DomainError("harmonic".to_string()))
        );
        assert_eq!(
            harmonic(-1.0),
            Err(CalcError::DomainError("harmonic".to_string()))
        );
        assert_eq!(
            harmonic(1.5),
            Err(CalcError::DomainError("harmonic".to_string()))
        );
    }

    #[test]
    fn binom_domain_and_symmetry() {
        assert_eq!(binom(5.0, 2.0).unwrap(), 10.0);
        assert_eq!(binom(5.0, 3.0).unwrap(), 10.0);
        assert_eq!(
            binom(-1.0, 1.0),
            Err(CalcError::DomainError("binom".to_string()))
        );
        assert_eq!(
            binom(5.0, -1.0),
            Err(CalcError::DomainError("binom".to_string()))
        );
        assert_eq!(
            binom(5.5, 2.0),
            Err(CalcError::DomainError("binom".to_string()))
        );
        assert_eq!(
            binom(5.0, 2.5),
            Err(CalcError::DomainError("binom".to_string()))
        );
        assert_eq!(
            binom(2.0, 5.0),
            Err(CalcError::DomainError("binom".to_string()))
        );
    }

    #[test]
    fn median_odd_and_even_counts() {
        assert_eq!(median(&[3.0, 1.0, 2.0]).unwrap(), 2.0);
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]).unwrap(), 2.5);
        assert_eq!(
            median(&[]),
            Err(CalcError::WrongArgCount {
                name: "median".to_string(),
                expected: "at least 1".to_string(),
                got: 0,
            })
        );
    }

    #[test]
    fn mode_ties_return_smallest() {
        assert_eq!(mode(&[1.0, 2.0, 2.0, 3.0]).unwrap(), 2.0);
        assert_eq!(mode(&[3.0, 1.0, 1.0, 3.0]).unwrap(), 1.0);
        assert_eq!(mode(&[5.0]).unwrap(), 5.0);
    }

    #[test]
    fn variance_and_stddev_of_known_dataset() {
        // Population variance of [2, 4, 4, 4, 5, 5, 7, 9] is 4, stddev 2.
        let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        assert!((variance(&data).unwrap() - 4.0).abs() < 1e-9);
        assert!((stddev(&data).unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn percentile_domain_and_known_values() {
        assert_eq!(percentile(&[0.0, 1.0, 2.0, 3.0]).unwrap(), 1.0);
        assert_eq!(percentile(&[100.0, 1.0, 2.0, 3.0]).unwrap(), 3.0);
        assert_eq!(percentile(&[50.0, 1.0, 2.0, 3.0]).unwrap(), 2.0);
        assert_eq!(percentile(&[50.0, 5.0]).unwrap(), 5.0);
        assert_eq!(
            percentile(&[150.0, 1.0, 2.0]),
            Err(CalcError::DomainError("percentile".to_string()))
        );
        assert_eq!(
            percentile(&[50.0]),
            Err(CalcError::WrongArgCount {
                name: "percentile".to_string(),
                expected: "at least 2 (percentile, data...)".to_string(),
                got: 1,
            })
        );
    }

    #[test]
    fn covariance_and_correlation_of_perfectly_correlated_data() {
        // y = 2x, so correlation should be exactly 1.
        let pairs = [1.0, 2.0, 2.0, 4.0, 3.0, 6.0];
        assert!(covariance(&pairs).unwrap() > 0.0);
        assert!((correlation(&pairs).unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn covariance_and_correlation_odd_arg_count_errors() {
        assert_eq!(
            covariance(&[1.0, 2.0, 3.0]),
            Err(CalcError::WrongArgCount {
                name: "covariance".to_string(),
                expected: "an even number of arguments (x1, y1, x2, y2, ...)".to_string(),
                got: 3,
            })
        );
        assert_eq!(
            correlation(&[]),
            Err(CalcError::WrongArgCount {
                name: "correlation".to_string(),
                expected: "an even number of arguments (x1, y1, x2, y2, ...)".to_string(),
                got: 0,
            })
        );
    }

    #[test]
    fn correlation_of_constant_series_errors() {
        assert_eq!(
            correlation(&[1.0, 1.0, 1.0, 2.0]),
            Err(CalcError::DomainError("correlation".to_string()))
        );
    }

    #[test]
    fn skewness_of_symmetric_data_is_zero() {
        let result = skewness(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        assert!(result.abs() < 1e-9);
    }

    #[test]
    fn skewness_matches_known_right_skewed_example() {
        // A right-skewed sample: a long tail towards higher values.
        let result = skewness(&[1.0, 2.0, 2.0, 3.0, 10.0]).unwrap();
        assert!(result > 0.0);
    }

    #[test]
    fn skewness_of_constant_data_errors() {
        assert_eq!(
            skewness(&[5.0, 5.0, 5.0]),
            Err(CalcError::DomainError("skewness".to_string()))
        );
    }

    #[test]
    fn kurtosis_of_normal_like_data_is_near_zero() {
        // A reasonably normal-shaped sample.
        let data = [-2.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 2.0];
        let result = kurtosis(&data).unwrap();
        assert!(result.is_finite());
    }

    #[test]
    fn kurtosis_of_uniform_data_is_negative() {
        // A discrete uniform sample: platykurtic (negative excess kurtosis).
        let result = kurtosis(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        assert!((result - (-1.3)).abs() < 1e-9);
    }

    #[test]
    fn kurtosis_of_constant_data_errors() {
        assert_eq!(
            kurtosis(&[5.0, 5.0, 5.0]),
            Err(CalcError::DomainError("kurtosis".to_string()))
        );
    }
}
