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

/// Unsigned Stirling numbers of the first kind, `|s(n, k)|`: the number of
/// permutations of `n` elements with exactly `k` cycles. Computed via the
/// standard recurrence `|s(n, k)| = |s(n-1, k-1)| + (n-1) * |s(n-1, k)|`,
/// with base cases `|s(0, 0)| = 1` and `|s(n, 0)| = 0` for `n > 0`.
pub fn stirling1(n: f64, k: f64) -> CalcResult<f64> {
    let n = non_negative_u64("stirling1", n)?;
    let k = non_negative_u64("stirling1", k)?;
    if k > n {
        return Ok(0.0);
    }
    // row[j] holds |s(i, j)| for the current i, built up from i = 0.
    let mut row = vec![0f64; (n + 1) as usize];
    row[0] = 1.0;
    for i in 1..=n {
        for j in (1..=i).rev() {
            row[j as usize] = row[j as usize - 1] + (i - 1) as f64 * row[j as usize];
            if row[j as usize].is_infinite() {
                return Err(CalcError::Overflow);
            }
        }
        row[0] = 0.0;
    }
    Ok(row[k as usize])
}

/// Stirling numbers of the second kind, `S(n, k)`: the number of ways to
/// partition a set of `n` elements into exactly `k` non-empty subsets.
/// Computed via the standard recurrence
/// `S(n, k) = k * S(n-1, k) + S(n-1, k-1)`, with base cases `S(0, 0) = 1`
/// and `S(n, 0) = 0` for `n > 0`.
pub fn stirling2(n: f64, k: f64) -> CalcResult<f64> {
    let n = non_negative_u64("stirling2", n)?;
    let k = non_negative_u64("stirling2", k)?;
    if k > n {
        return Ok(0.0);
    }
    let mut row = vec![0f64; (n + 1) as usize];
    row[0] = 1.0;
    for i in 1..=n {
        for j in (1..=i).rev() {
            row[j as usize] = j as f64 * row[j as usize] + row[j as usize - 1];
            if row[j as usize].is_infinite() {
                return Err(CalcError::Overflow);
            }
        }
        row[0] = 0.0;
    }
    Ok(row[k as usize])
}

/// The number of derangements of `n` elements (permutations with no fixed
/// points), via the recurrence `D(n) = (n-1) * (D(n-1) + D(n-2))`, with
/// base cases `D(0) = 1`, `D(1) = 0`.
pub fn derangement(n: f64) -> CalcResult<f64> {
    let n = non_negative_u64("derangement", n)?;
    if n == 0 {
        return Ok(1.0);
    }
    if n == 1 {
        return Ok(0.0);
    }
    let (mut prev2, mut prev1) = (1f64, 0f64); // D(0), D(1)
    for i in 2..=n {
        let current = (i - 1) as f64 * (prev1 + prev2);
        if current.is_infinite() {
            return Err(CalcError::Overflow);
        }
        prev2 = prev1;
        prev1 = current;
    }
    Ok(prev1)
}

/// The `n`-th Bell number: the number of ways to partition a set of `n`
/// elements into any number of non-empty subsets, i.e. `sum_{k=0}^{n}
/// S(n, k)` (the sum of Stirling numbers of the second kind over all `k`).
pub fn bell(n: f64) -> CalcResult<f64> {
    let n_u = non_negative_u64("bell", n)?;
    let mut total = 0f64;
    for k in 0..=n_u {
        total += stirling2(n, k as f64)?;
        if total.is_infinite() {
            return Err(CalcError::Overflow);
        }
    }
    Ok(total)
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

    #[test]
    fn stirling1_matches_known_values() {
        assert_eq!(stirling1(0.0, 0.0).unwrap(), 1.0);
        assert_eq!(stirling1(4.0, 2.0).unwrap(), 11.0);
        assert_eq!(stirling1(5.0, 1.0).unwrap(), 24.0);
        assert_eq!(stirling1(5.0, 5.0).unwrap(), 1.0);
    }

    #[test]
    fn stirling1_zero_when_k_exceeds_n() {
        assert_eq!(stirling1(2.0, 5.0).unwrap(), 0.0);
    }

    #[test]
    fn stirling2_matches_known_values() {
        assert_eq!(stirling2(0.0, 0.0).unwrap(), 1.0);
        assert_eq!(stirling2(4.0, 2.0).unwrap(), 7.0);
        assert_eq!(stirling2(5.0, 1.0).unwrap(), 1.0);
        assert_eq!(stirling2(5.0, 5.0).unwrap(), 1.0);
    }

    #[test]
    fn stirling2_zero_when_k_exceeds_n() {
        assert_eq!(stirling2(2.0, 5.0).unwrap(), 0.0);
    }

    #[test]
    fn derangement_matches_known_values() {
        assert_eq!(derangement(0.0).unwrap(), 1.0);
        assert_eq!(derangement(1.0).unwrap(), 0.0);
        assert_eq!(derangement(2.0).unwrap(), 1.0);
        assert_eq!(derangement(3.0).unwrap(), 2.0);
        assert_eq!(derangement(4.0).unwrap(), 9.0);
        assert_eq!(derangement(5.0).unwrap(), 44.0);
    }

    #[test]
    fn bell_matches_known_values() {
        assert_eq!(bell(0.0).unwrap(), 1.0);
        assert_eq!(bell(1.0).unwrap(), 1.0);
        assert_eq!(bell(2.0).unwrap(), 2.0);
        assert_eq!(bell(3.0).unwrap(), 5.0);
        assert_eq!(bell(4.0).unwrap(), 15.0);
        assert_eq!(bell(5.0).unwrap(), 52.0);
    }

    #[test]
    fn stirling_and_bell_reject_negative_or_fractional_input() {
        assert!(matches!(
            stirling1(-1.0, 0.0),
            Err(CalcError::DomainError(_))
        ));
        assert!(matches!(
            stirling2(1.5, 0.0),
            Err(CalcError::DomainError(_))
        ));
        assert!(matches!(derangement(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(bell(-1.0), Err(CalcError::DomainError(_))));
    }
}
