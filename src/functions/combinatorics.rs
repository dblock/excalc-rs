//! Combinatorics functions: counting permutations, combinations-adjacent
//! sequences (Catalan numbers), explicit factorial, and multinomial
//! coefficients. `binom` (combinations) already lives in `stats.rs`.

use crate::error::{CalcError, CalcResult};

/// Validates that `x` is a non-negative whole number representable as a
/// `u64`, erroring with `DomainError(name)` otherwise.
fn non_negative_u64(name: &str, x: f64) -> CalcResult<u64> {
    if x < 0.0 || x.fract() != 0.0 || x > u64::MAX as f64 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(x as u64)
}

/// `n!`, the product of all positive integers up to `n`. Equivalent to the
/// postfix `!` operator, exposed as a named function for use as an ordinary
/// argument (e.g. `sum(factorial(3), factorial(4))`).
pub fn factorial(n: f64) -> CalcResult<f64> {
    let n = non_negative_u64("factorial", n)?;
    let mut result: f64 = 1.0;
    for i in 2..=n {
        result *= i as f64;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
    }
    Ok(result)
}

/// Number of ways to arrange `r` items out of `n` distinct items, where
/// order matters: `P(n, r) = n! / (n - r)!`.
pub fn perm(n: f64, r: f64) -> CalcResult<f64> {
    let n = non_negative_u64("perm", n)?;
    let r = non_negative_u64("perm", r)?;
    if r > n {
        return Err(CalcError::DomainError("perm".to_string()));
    }
    let mut result: f64 = 1.0;
    for i in (n - r + 1)..=n {
        result *= i as f64;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
    }
    Ok(result)
}

/// The `n`-th Catalan number, `C(n) = binom(2n, n) / (n + 1)`, counting
/// e.g. balanced-parenthesization or binary-tree-shape structures of size
/// `n`. Computed via the multiplicative recurrence
/// `C(n) = C(n-1) * 2(2n-1) / (n+1)` to avoid overflowing on the way to a
/// modestly large `n`.
pub fn catalan(n: f64) -> CalcResult<f64> {
    let n = non_negative_u64("catalan", n)?;
    let mut result: f64 = 1.0;
    for i in 1..=n {
        result *= (2 * (2 * i - 1)) as f64;
        result /= (i + 1) as f64;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
    }
    Ok(result.round())
}

/// The multinomial coefficient `n! / (k1! * k2! * ... * km!)`, generalizing
/// `binom` to more than two groups. Requires `k1 + k2 + ... + km = n`, all
/// non-negative whole numbers.
pub fn multinomial(args: &[f64]) -> CalcResult<f64> {
    if args.len() < 2 {
        return Err(CalcError::WrongArgCount {
            name: "multinomial".to_string(),
            expected: "at least 2".to_string(),
            got: args.len(),
        });
    }
    let n = non_negative_u64("multinomial", args[0])?;
    let mut ks = Vec::with_capacity(args.len() - 1);
    let mut sum = 0u64;
    for &k in &args[1..] {
        let k = non_negative_u64("multinomial", k)?;
        sum += k;
        ks.push(k);
    }
    if sum != n {
        return Err(CalcError::DomainError("multinomial".to_string()));
    }
    // Compute n! / (k1! * k2! * ...) via a single running product over the
    // multinomial's standard combinatorial construction, rather than
    // factorial(n) directly, to reduce overflow risk for large n.
    let mut remaining = n;
    let mut result: f64 = 1.0;
    for k in ks {
        result *= perm(remaining as f64, k as f64)?;
        result /= factorial(k as f64)?;
        remaining -= k;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
    }
    Ok(result.round())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factorial_matches_known_values() {
        assert_eq!(factorial(0.0).unwrap(), 1.0);
        assert_eq!(factorial(5.0).unwrap(), 120.0);
    }

    #[test]
    fn factorial_domain_and_overflow() {
        assert_eq!(
            factorial(-1.0),
            Err(CalcError::DomainError("factorial".to_string()))
        );
        assert_eq!(
            factorial(2.5),
            Err(CalcError::DomainError("factorial".to_string()))
        );
        assert_eq!(factorial(2000.0), Err(CalcError::Overflow));
    }

    #[test]
    fn perm_matches_known_values() {
        assert_eq!(perm(5.0, 2.0).unwrap(), 20.0);
        assert_eq!(perm(5.0, 0.0).unwrap(), 1.0);
        assert_eq!(perm(5.0, 5.0).unwrap(), 120.0);
    }

    #[test]
    fn perm_domain_errors() {
        assert_eq!(
            perm(5.0, 6.0),
            Err(CalcError::DomainError("perm".to_string()))
        );
        assert_eq!(
            perm(-1.0, 1.0),
            Err(CalcError::DomainError("perm".to_string()))
        );
    }

    #[test]
    fn catalan_matches_known_values() {
        assert_eq!(catalan(0.0).unwrap(), 1.0);
        assert_eq!(catalan(1.0).unwrap(), 1.0);
        assert_eq!(catalan(2.0).unwrap(), 2.0);
        assert_eq!(catalan(3.0).unwrap(), 5.0);
        assert_eq!(catalan(10.0).unwrap(), 16796.0);
    }

    #[test]
    fn multinomial_matches_known_values() {
        // 10! / (2! 3! 5!) = 2520
        assert_eq!(multinomial(&[10.0, 2.0, 3.0, 5.0]).unwrap(), 2520.0);
        // Degenerates to binom for two groups.
        assert_eq!(multinomial(&[5.0, 2.0, 3.0]).unwrap(), 10.0);
    }

    #[test]
    fn multinomial_requires_groups_to_sum_to_n() {
        assert_eq!(
            multinomial(&[10.0, 2.0, 3.0]),
            Err(CalcError::DomainError("multinomial".to_string()))
        );
    }

    #[test]
    fn multinomial_requires_at_least_two_arguments() {
        assert_eq!(
            multinomial(&[10.0]),
            Err(CalcError::WrongArgCount {
                name: "multinomial".to_string(),
                expected: "at least 2".to_string(),
                got: 1,
            })
        );
    }
}
