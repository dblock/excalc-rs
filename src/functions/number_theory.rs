//! Number theory functions, ported from the "IV. General Functions" and
//! "VII. Primes and Numbers" chapters of the original manual.

use crate::error::{CalcError, CalcResult};

/// Deterministic Miller-Rabin primality test, valid for the full `u64` range
/// (the witness set `{2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37}` is a known
/// deterministic base for all `n < 3,317,044,064,679,887,385,961,981`, which
/// covers all of `u64`).
fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n == p {
            return true;
        }
        if n.is_multiple_of(p) {
            return false;
        }
    }
    let mut d = n - 1;
    let mut r = 0u32;
    while d.is_multiple_of(2) {
        d /= 2;
        r += 1;
    }
    'witness: for &a in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 0..r - 1 {
            x = mod_mul(x, x, n);
            if x == n - 1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

/// `(a * b) % m` without overflowing `u64`, via `u128` intermediates.
fn mod_mul(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// `(base ^ exp) % m`, via repeated squaring.
fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut result = 1u64;
    base %= m;
    while exp > 0 {
        if exp % 2 == 1 {
            result = mod_mul(result, base, m);
        }
        exp /= 2;
        base = mod_mul(base, base, m);
    }
    result
}

/// Parses a non-negative integer argument, erroring with `DomainError(name)`
/// if it's negative or not a whole number.
fn non_negative_integer(name: &str, x: f64) -> CalcResult<u64> {
    if x < 0.0 || x.fract() != 0.0 || x > u64::MAX as f64 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(x as u64)
}

fn gcd_two(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd_two(b, a % b)
    }
}

/// Greatest common divisor of all arguments (Euclid's algorithm, folded
/// pairwise across the argument list).
pub fn gcd(args: &[f64]) -> CalcResult<f64> {
    if args.len() < 2 {
        return Err(CalcError::WrongArgCount {
            name: "gcd".to_string(),
            expected: "at least 2".to_string(),
            got: args.len(),
        });
    }
    let mut acc = non_negative_integer("gcd", args[0])?;
    for &a in &args[1..] {
        acc = gcd_two(acc, non_negative_integer("gcd", a)?);
    }
    Ok(acc as f64)
}

/// Least common multiple of all arguments: `lcm(x, y) = (x / gcd(x, y)) * y`.
pub fn lcm(args: &[f64]) -> CalcResult<f64> {
    if args.len() < 2 {
        return Err(CalcError::WrongArgCount {
            name: "lcm".to_string(),
            expected: "at least 2".to_string(),
            got: args.len(),
        });
    }
    let mut acc = non_negative_integer("lcm", args[0])?;
    for &a in &args[1..] {
        let b = non_negative_integer("lcm", a)?;
        if acc == 0 || b == 0 {
            acc = 0;
            continue;
        }
        let g = gcd_two(acc, b);
        acc = acc
            .checked_div(g)
            .and_then(|q| q.checked_mul(b))
            .ok_or(CalcError::Overflow)?;
    }
    Ok(acc as f64)
}

/// The `n`-th Fibonacci number, computed iteratively (`fib(0) = 0`, `fib(1) = 1`).
pub fn fib(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("fib", n)?;
    let (mut a, mut b) = (0f64, 1f64);
    for _ in 0..n {
        let next = a + b;
        a = b;
        b = next;
        if b.is_infinite() {
            return Err(CalcError::Overflow);
        }
    }
    Ok(a)
}

/// The closest prime `<= n` (returns `n` itself if `n` is prime); the
/// original manual calls this `Prime`. Exposed to the calculator as `prime?`
/// (Scheme-style predicate naming, since it also confirms primality when
/// `isprime(n) == n`).
pub fn isprime(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("prime?", n)?;
    let mut candidate = n;
    loop {
        if is_prime_u64(candidate) {
            return Ok(candidate as f64);
        }
        if candidate == 0 {
            return Err(CalcError::DomainError("prime?".to_string()));
        }
        candidate -= 1;
    }
}

/// The Mobius function `mu(n)`: `1` for a squarefree number with an even
/// count of distinct prime factors, `-1` for an odd count, `0` if any prime
/// factor repeats. `moebius(1) = 1` by convention.
pub fn moebius(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("moebius", n)?;
    if n == 0 {
        return Err(CalcError::DomainError("moebius".to_string()));
    }
    if n == 1 {
        return Ok(1.0);
    }
    let mut m = n;
    let mut distinct_factors = 0;
    let mut p = 2u64;
    while p * p <= m {
        if m % p == 0 {
            m /= p;
            if m % p == 0 {
                return Ok(0.0);
            }
            distinct_factors += 1;
        }
        p += 1;
    }
    if m > 1 {
        distinct_factors += 1;
    }
    Ok(if distinct_factors % 2 == 0 { 1.0 } else { -1.0 })
}

/// The Mersenne number `2^p - 1` for a given exponent `p`.
pub fn mersenne(p: f64) -> CalcResult<f64> {
    let p = non_negative_integer("mersenne", p)?;
    if p == 0 || p > 1023 {
        return Err(CalcError::DomainError("mersenne".to_string()));
    }
    let value = 2f64.powi(p as i32) - 1.0;
    // Defensive: unreachable given the `p <= 1023` domain check above (the
    // largest representable value, `2^1023 - 1`, is still finite).
    if value.is_infinite() {
        return Err(CalcError::Overflow);
    }
    Ok(value)
}

/// Known Mersenne prime generators (exponents `p` for which `2^p - 1` is
/// prime), up to a generator bound that keeps `2^p - 1` representable
/// exactly in a `u64` (`p <= 63`).
fn mersenne_generators() -> impl Iterator<Item = u64> {
    (2..=63u64).filter(|&p| is_prime_u64(p) && is_prime_u64((1u64 << p) - 1))
}

/// The closest known perfect number `<= n`: for every Mersenne prime
/// generator `p`, `2^(p-1) * (2^p - 1)` is an even perfect number, and every
/// known even perfect number arises this way.
pub fn perfect(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("perfect", n)?;
    let mut best: Option<u64> = None;
    for p in mersenne_generators() {
        let mersenne_number = (1u64 << p) - 1;
        let Some(value) = (1u64 << (p - 1)).checked_mul(mersenne_number) else {
            break;
        };
        if value > n {
            break;
        }
        best = Some(value);
    }
    best.map(|v| v as f64)
        .ok_or_else(|| CalcError::DomainError("perfect".to_string()))
}

/// The `k`-th Fermat number, `2^(2^k) + 1` (only known to be prime for
/// `k <= 4`; this port computes the formula generically for any `k >= 0`).
pub fn fermat(k: f64) -> CalcResult<f64> {
    let k = non_negative_integer("fermat", k)?;
    if k > 9 {
        // 2^(2^10) already vastly exceeds f64's range; anything beyond
        // k = 9 (2^512 + 1) is not meaningfully representable.
        return Err(CalcError::Overflow);
    }
    let exponent = 1u64 << k;
    let value = 2f64.powi(exponent as i32) + 1.0;
    // Defensive: unreachable given the `k <= 9` domain check above (the
    // largest representable value, `2^512 + 1`, is still finite).
    if value.is_infinite() {
        return Err(CalcError::Overflow);
    }
    Ok(value)
}

/// The smallest safe prime `>= n` (a prime `p` where `(p - 1) / 2` is also
/// prime).
pub fn safeprime(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("safeprime", n)?;
    let mut candidate = n.max(2);
    loop {
        if is_prime_u64(candidate) && candidate >= 3 && is_prime_u64((candidate - 1) / 2) {
            return Ok(candidate as f64);
        }
        candidate = candidate.checked_add(1).ok_or(CalcError::Overflow)?;
    }
}

/// Count of primes `<= n`.
pub fn primec(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("primec", n)?;
    if n < 2 {
        return Ok(0.0);
    }
    Ok((2..=n).filter(|&i| is_prime_u64(i)).count() as f64)
}

/// The `n`-th prime in the conventional 1-indexed enumeration
/// (`prime(1) = 2`, `prime(2) = 3`, `prime(3) = 5`, ...), except the
/// original manual overrides `primen(1)` to be `1` instead of `2`.
pub fn primen(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("primen", n)?;
    if n == 0 {
        return Err(CalcError::DomainError("primen".to_string()));
    }
    if n == 1 {
        return Ok(1.0);
    }
    let mut count = 0u64;
    let mut candidate = 1u64;
    while count < n {
        candidate = candidate.checked_add(1).ok_or(CalcError::Overflow)?;
        if is_prime_u64(candidate) {
            count += 1;
        }
    }
    Ok(candidate as f64)
}

/// The closest known Mersenne generator (prime `p` such that `2^p - 1` is
/// prime) `<= n`.
pub fn mersennegen(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("mersennegen", n)?;
    mersenne_generators()
        .filter(|&p| p <= n)
        .last()
        .map(|p| p as f64)
        .ok_or_else(|| CalcError::DomainError("mersennegen".to_string()))
}

/// The Mersenne number `2^p - 1` for a given Mersenne generator `p` (`p`
/// must itself be a known Mersenne generator).
pub fn mersgen(p: f64) -> CalcResult<f64> {
    let p = non_negative_integer("mersgen", p)?;
    if !mersenne_generators().any(|g| g == p) {
        return Err(CalcError::DomainError("mersgen".to_string()));
    }
    Ok(((1u64 << p) - 1) as f64)
}

/// The generator `p` for a given Mersenne number `m` (i.e. the inverse of
/// `mersgen`).
pub fn genmers(m: f64) -> CalcResult<f64> {
    let m = non_negative_integer("genmers", m)?;
    mersenne_generators()
        .find(|&p| (1u64 << p) - 1 == m)
        .map(|p| p as f64)
        .ok_or_else(|| CalcError::DomainError("genmers".to_string()))
}

/// Sum of the `k`-th power of positive divisors of `n`: `sigma(n, 0)` is the
/// number of divisors, `sigma(n, 1)` is their sum (a.k.a. `tau`).
pub fn sigma(n: f64, k: f64) -> CalcResult<f64> {
    let n = non_negative_integer("sigma", n)?;
    let k = non_negative_integer("sigma", k)?;
    if n == 0 {
        return Err(CalcError::DomainError("sigma".to_string()));
    }
    let mut total = 0f64;
    for d in 1..=n {
        if n % d == 0 {
            total += (d as f64).powi(k as i32);
            if total.is_infinite() {
                return Err(CalcError::Overflow);
            }
        }
    }
    Ok(total)
}

/// Sum of positive divisors of `n` (i.e. `sigma(n, 1)`).
pub fn tau(n: f64) -> CalcResult<f64> {
    sigma(n, 1.0)
}

/// Euler's totient function: the count of positive integers `<= n` that are
/// coprime to `n`.
pub fn phi(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("phi", n)?;
    if n == 0 {
        return Err(CalcError::DomainError("phi".to_string()));
    }
    let mut result = n;
    let mut m = n;
    let mut p = 2u64;
    while p * p <= m {
        if m % p == 0 {
            while m % p == 0 {
                m /= p;
            }
            result -= result / p;
        }
        p += 1;
    }
    if m > 1 {
        result -= result / m;
    }
    Ok(result as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_matches_manual_example() {
        assert_eq!(gcd(&[3213.0, 24.0]).unwrap(), 3.0);
        assert_eq!(gcd(&[12.0, 18.0, 30.0]).unwrap(), 6.0);
        assert_eq!(
            gcd(&[1.0]),
            Err(CalcError::WrongArgCount {
                name: "gcd".to_string(),
                expected: "at least 2".to_string(),
                got: 1,
            })
        );
    }

    #[test]
    fn lcm_matches_manual_example() {
        assert_eq!(lcm(&[14.0, 4.0]).unwrap(), 28.0);
        assert_eq!(lcm(&[4.0, 6.0, 10.0]).unwrap(), 60.0);
        assert_eq!(lcm(&[0.0, 5.0]).unwrap(), 0.0);
        assert_eq!(
            lcm(&[5.0]),
            Err(CalcError::WrongArgCount {
                name: "lcm".to_string(),
                expected: "at least 2".to_string(),
                got: 1,
            })
        );
        // Two large coprime primes whose product overflows u64.
        assert_eq!(lcm(&[4294967311.0, 4294967357.0]), Err(CalcError::Overflow));
    }

    #[test]
    fn fib_matches_manual_example() {
        assert_eq!(fib(0.0).unwrap(), 0.0);
        assert_eq!(fib(1.0).unwrap(), 1.0);
        assert_eq!(fib(10.0).unwrap(), 55.0);
        assert!((fib(56.0).unwrap() - 225851433717.0).abs() < 1.0);
        assert_eq!(fib(1500.0), Err(CalcError::Overflow));
    }

    #[test]
    fn isprime_matches_manual_example() {
        assert_eq!(isprime(86.0).unwrap(), 83.0);
        assert_eq!(isprime(2.0).unwrap(), 2.0);
        assert_eq!(
            isprime(-1.0),
            Err(CalcError::DomainError("prime?".to_string()))
        );
        assert_eq!(
            isprime(0.0),
            Err(CalcError::DomainError("prime?".to_string()))
        );
    }

    #[test]
    fn moebius_matches_manual_example() {
        assert_eq!(moebius(1.0).unwrap(), 1.0);
        assert_eq!(moebius(2.0).unwrap(), -1.0);
        assert_eq!(moebius(6.0).unwrap(), 1.0);
        assert_eq!(moebius(4.0).unwrap(), 0.0);
        assert_eq!(moebius(30.0).unwrap(), -1.0); // 2 * 3 * 5, 3 distinct factors
        assert_eq!(moebius(35.0).unwrap(), 1.0); // 5 * 7, with non-factor trial divisors in between
        assert_eq!(
            moebius(0.0),
            Err(CalcError::DomainError("moebius".to_string()))
        );
    }

    #[test]
    fn mersenne_is_literal_exponent() {
        assert_eq!(mersenne(7.0).unwrap(), 127.0);
        assert_eq!(mersenne(3.0).unwrap(), 7.0);
        assert_eq!(
            mersenne(0.0),
            Err(CalcError::DomainError("mersenne".to_string()))
        );
    }

    #[test]
    fn perfect_matches_manual_example() {
        assert_eq!(perfect(6.0).unwrap(), 6.0);
        assert_eq!(perfect(1231.0).unwrap(), 496.0);
        assert_eq!(
            perfect(1.0),
            Err(CalcError::DomainError("perfect".to_string()))
        );
        // u64::MAX: large enough that the generator search runs past the
        // point where `2^(p-1) * (2^p - 1)` itself overflows u64, exercising
        // the overflow-break path (as opposed to the `value > n` break).
        assert_eq!(perfect(u64::MAX as f64).unwrap(), 2305843008139952128.0);
    }

    #[test]
    fn fermat_matches_manual_example() {
        assert_eq!(fermat(4.0).unwrap(), 65537.0);
        assert_eq!(fermat(0.0).unwrap(), 3.0);
        assert_eq!(fermat(10.0), Err(CalcError::Overflow));
    }

    #[test]
    fn safeprime_matches_manual_example() {
        assert_eq!(safeprime(12.0).unwrap(), 23.0);
        assert_eq!(safeprime(23.0).unwrap(), 23.0);
    }

    #[test]
    fn primec_matches_manual_example() {
        assert_eq!(primec(86.0).unwrap(), 23.0);
        assert_eq!(primec(0.0).unwrap(), 0.0);
    }

    #[test]
    fn primen_matches_manual_example() {
        assert_eq!(primen(1.0).unwrap(), 1.0);
        assert_eq!(primen(86.0).unwrap(), 443.0);
        assert_eq!(
            primen(0.0),
            Err(CalcError::DomainError("primen".to_string()))
        );
    }

    #[test]
    fn mersennegen_matches_manual_example() {
        assert_eq!(mersennegen(12.0).unwrap(), 7.0);
    }

    #[test]
    fn mersgen_matches_manual_example() {
        assert_eq!(mersgen(7.0).unwrap(), 127.0);
        assert_eq!(
            mersgen(4.0),
            Err(CalcError::DomainError("mersgen".to_string()))
        );
    }

    #[test]
    fn genmers_matches_manual_example() {
        assert_eq!(genmers(127.0).unwrap(), 7.0);
        assert_eq!(
            genmers(100.0),
            Err(CalcError::DomainError("genmers".to_string()))
        );
    }

    #[test]
    fn sigma_matches_manual_example() {
        assert_eq!(sigma(100.0, 0.0).unwrap(), 9.0);
        assert_eq!(
            sigma(0.0, 0.0),
            Err(CalcError::DomainError("sigma".to_string()))
        );
        // 2^1030 overflows f64 (max finite exponent is 1023), exercising
        // sigma's inner overflow guard.
        assert_eq!(sigma(2.0, 1030.0), Err(CalcError::Overflow));
    }

    #[test]
    fn tau_matches_manual_example() {
        assert_eq!(tau(9.0).unwrap(), 13.0);
    }

    #[test]
    fn phi_matches_manual_example() {
        assert_eq!(phi(12.0).unwrap(), 4.0);
        assert_eq!(phi(1.0).unwrap(), 1.0);
        assert_eq!(phi(30.0).unwrap(), 8.0); // 2 * 3 * 5, multiple distinct factors
        assert_eq!(phi(35.0).unwrap(), 24.0); // 5 * 7, with non-factor trial divisors in between
        assert_eq!(phi(0.0), Err(CalcError::DomainError("phi".to_string())));
    }

    #[test]
    fn domain_errors_for_negative_or_fractional_input() {
        assert!(matches!(fib(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(fib(1.5), Err(CalcError::DomainError(_))));
        assert!(matches!(isprime(1.5), Err(CalcError::DomainError(_))));
        assert!(matches!(moebius(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(fermat(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(safeprime(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(primec(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(primen(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(mersennegen(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(sigma(-1.0, 0.0), Err(CalcError::DomainError(_))));
    }
}
