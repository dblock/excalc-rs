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
}
