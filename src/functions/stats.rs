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
