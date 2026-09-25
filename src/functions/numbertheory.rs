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

/// `lcm(a, b)`, guarding against overflow and the `a == 0 || b == 0` edge
/// case (by convention `lcm(0, x) = 0`).
fn lcm_two(a: u64, b: u64) -> CalcResult<u64> {
    if a == 0 || b == 0 {
        return Ok(0);
    }
    let g = gcd_two(a, b);
    (a / g).checked_mul(b).ok_or(CalcError::Overflow)
}

/// The prime factorization of `n` as `(prime, exponent)` pairs in
/// increasing order of prime, via trial division. `n = 1` yields an empty
/// list (the empty product).
fn prime_factorize(mut n: u64) -> Vec<(u64, u32)> {
    let mut factors = Vec::new();
    let mut p = 2u64;
    while p * p <= n {
        if n.is_multiple_of(p) {
            let mut exp = 0u32;
            while n.is_multiple_of(p) {
                n /= p;
                exp += 1;
            }
            factors.push((p, exp));
        }
        p += 1;
    }
    if n > 1 {
        factors.push((n, 1));
    }
    factors
}

/// Parses an argument as a plain (possibly negative) integer, erroring with
/// `DomainError(name)` if it's not a whole number or doesn't fit in `i64`.
fn integer_arg(name: &str, x: f64) -> CalcResult<i64> {
    if !x.is_finite() || x.fract() != 0.0 || x.abs() > i64::MAX as f64 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(x as i64)
}

/// Requires at least two arguments, erroring with `WrongArgCount(name)`
/// otherwise. Shared by `gcd`/`lcm`, which both fold pairwise over at
/// least two values.
fn require_at_least_two(name: &str, args: &[f64]) -> CalcResult<()> {
    if args.len() < 2 {
        return Err(CalcError::WrongArgCount {
            name: name.to_string(),
            expected: "at least 2".to_string(),
            got: args.len(),
        });
    }
    Ok(())
}

/// Greatest common divisor of all arguments (Euclid's algorithm, folded
/// pairwise across the argument list).
pub fn gcd(args: &[f64]) -> CalcResult<f64> {
    require_at_least_two("gcd", args)?;
    let mut acc = non_negative_integer("gcd", args[0])?;
    for &a in &args[1..] {
        acc = gcd_two(acc, non_negative_integer("gcd", a)?);
    }
    Ok(acc as f64)
}

/// Least common multiple of all arguments: `lcm(x, y) = (x / gcd(x, y)) * y`.
pub fn lcm(args: &[f64]) -> CalcResult<f64> {
    require_at_least_two("lcm", args)?;
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

/// The `n`-th Lucas number: same recurrence as Fibonacci (`L(n) = L(n-1) +
/// L(n-2)`), but starting `L(0) = 2, L(1) = 1` instead of `0, 1`.
pub fn lucas(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("lucas", n)?;
    let (mut a, mut b) = (2f64, 1f64);
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

/// The primorial `n#`: the product of all primes `<= n` (`primorial(0) = 1`
/// by convention, matching the empty-product identity).
pub fn primorial(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("primorial", n)?;
    let mut result = 1f64;
    for i in 2..=n {
        if is_prime_u64(i) {
            result *= i as f64;
            if result.is_infinite() {
                return Err(CalcError::Overflow);
            }
        }
    }
    Ok(result)
}

/// Sum of the decimal digits of `n` (e.g. `digitsum(12345) = 15`).
pub fn digitsum(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("digitsum", n)?;
    Ok(digits(n).iter().sum::<u64>() as f64)
}

/// Decimal digits of `n`, most significant first (`digits(0) = [0]`).
fn digits(mut n: u64) -> Vec<u64> {
    if n == 0 {
        return vec![0];
    }
    let mut ds = Vec::new();
    while n > 0 {
        ds.push(n % 10);
        n /= 10;
    }
    ds.reverse();
    ds
}

/// The digital root of `n`: repeatedly sum its digits until a single digit
/// remains (equivalently `1 + (n - 1) mod 9` for `n > 0`, `0` for `n = 0`).
pub fn digitalroot(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("digitalroot", n)?;
    if n == 0 {
        return Ok(0.0);
    }
    Ok((1 + (n - 1) % 9) as f64)
}

/// Whether `n`'s decimal representation reads the same forwards and
/// backwards. Returns `1` (true) or `0` (false), matching the calculator's
/// other predicate-style functions.
pub fn ispalindrome(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("palindrome?", n)?;
    let ds = digits(n);
    let is_palindrome = ds.iter().eq(ds.iter().rev());
    Ok(if is_palindrome { 1.0 } else { 0.0 })
}

/// The smallest prime strictly greater than `n`.
pub fn nextprime(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("nextprime", n)?;
    let mut candidate = n.max(1);
    loop {
        candidate = candidate.checked_add(1).ok_or(CalcError::Overflow)?;
        if is_prime_u64(candidate) {
            return Ok(candidate as f64);
        }
    }
}

/// The `n`-th triangular number: `T(n) = n(n+1)/2`, the count of objects
/// arranged in an equilateral triangle with `n` objects per side.
pub fn triangular(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("triangular", n)? as f64;
    Ok(n * (n + 1.0) / 2.0)
}

/// The `n`-th pentagonal number: `P(n) = n(3n-1)/2`.
pub fn pentagonal(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("pentagonal", n)? as f64;
    Ok(n * (3.0 * n - 1.0) / 2.0)
}

/// The `n`-th hexagonal number: `H(n) = n(2n-1)`.
pub fn hexagonal(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("hexagonal", n)? as f64;
    Ok(n * (2.0 * n - 1.0))
}

/// The Carmichael function `lambda(n)`: the smallest positive integer `m`
/// such that `a^m == 1 (mod n)` for every `a` coprime to `n`. Computed via
/// the prime factorization of `n`, taking the `lcm` of `lambda(p^e)` over
/// each prime power factor (`lambda(2) = 1`, `lambda(4) = 2`,
/// `lambda(2^e) = 2^(e-2)` for `e >= 3`; `lambda(p^e) = p^(e-1)(p-1)` for
/// odd primes `p`).
pub fn carmichael(n: f64) -> CalcResult<f64> {
    let n = non_negative_integer("carmichael", n)?;
    if n == 0 {
        return Err(CalcError::DomainError("carmichael".to_string()));
    }
    if n == 1 {
        return Ok(1.0);
    }
    let mut result = 1u64;
    for (p, e) in prime_factorize(n) {
        let lambda_pe = if p == 2 {
            match e {
                1 => 1,
                2 => 2,
                _ => 1u64 << (e - 2),
            }
        } else {
            let pow = p.checked_pow(e - 1).ok_or(CalcError::Overflow)?;
            pow.checked_mul(p - 1).ok_or(CalcError::Overflow)?
        };
        result = lcm_two(result, lambda_pe)?;
    }
    Ok(result as f64)
}

/// The sum of `n`'s proper divisors (all divisors except `n` itself), i.e.
/// `sigma(n, 1) - n`. `aliquot(1) = 0` (no proper divisors).
pub fn aliquot(n: f64) -> CalcResult<f64> {
    let nn = non_negative_integer("aliquot", n)?;
    if nn == 0 {
        return Err(CalcError::DomainError("aliquot".to_string()));
    }
    let total = sigma(n, 1.0)?;
    Ok(total - nn as f64)
}

/// Whether `n` is part of an amicable pair: `n != m` where `m = aliquot(n)`
/// and `aliquot(m) == n`. Returns `1` (true) or `0` (false).
pub fn isamicable(n: f64) -> CalcResult<f64> {
    let nn = non_negative_integer("amicable?", n)?;
    if nn == 0 {
        return Err(CalcError::DomainError("amicable?".to_string()));
    }
    let m = aliquot(n)?;
    let m_u = m as u64;
    if m_u == 0 || m_u == nn {
        return Ok(0.0);
    }
    let back = aliquot(m)?;
    Ok(if back as u64 == nn { 1.0 } else { 0.0 })
}

/// Whether `a` and `b` share no common factor other than `1` (`gcd(a, b) == 1`).
/// Returns `1` (true) or `0` (false).
pub fn iscoprime(a: f64, b: f64) -> CalcResult<f64> {
    let a = non_negative_integer("coprime?", a)?;
    let b = non_negative_integer("coprime?", b)?;
    if a == 0 && b == 0 {
        return Err(CalcError::DomainError("coprime?".to_string()));
    }
    Ok(if gcd_two(a, b) == 1 { 1.0 } else { 0.0 })
}

/// The multiplicative order of `a` modulo `n`: the smallest positive `k`
/// such that `a^k == 1 (mod n)`. Requires `n >= 2` and `gcd(a, n) == 1`.
pub fn order(a: f64, n: f64) -> CalcResult<f64> {
    let n_u = non_negative_integer("order", n)?;
    if n_u < 2 {
        return Err(CalcError::DomainError("order".to_string()));
    }
    let a_u = non_negative_integer("order", a)? % n_u;
    if gcd_two(a_u, n_u) != 1 {
        return Err(CalcError::DomainError("order".to_string()));
    }
    let phi_n = phi(n)? as u64;
    for k in 1..=phi_n {
        if mod_pow(a_u, k, n_u) == 1 {
            return Ok(k as f64);
        }
    }
    // Unreachable: the multiplicative order always divides phi(n) when
    // gcd(a, n) == 1, so the loop above is guaranteed to find it.
    Err(CalcError::Overflow)
}

/// The Jacobi symbol `(a/n)` for odd positive `n`, generalizing the
/// Legendre symbol to composite moduli via the standard iterative
/// reciprocity algorithm. Returns `-1`, `0`, or `1`.
pub fn jacobi(a: f64, n: f64) -> CalcResult<f64> {
    let n_i = integer_arg("jacobi", n)?;
    if n_i <= 0 || n_i % 2 == 0 {
        return Err(CalcError::DomainError("jacobi".to_string()));
    }
    let mut nn = n_i;
    let mut aa = integer_arg("jacobi", a)?.rem_euclid(nn);
    let mut result = 1i64;
    while aa != 0 {
        while aa % 2 == 0 {
            aa /= 2;
            let r = nn % 8;
            if r == 3 || r == 5 {
                result = -result;
            }
        }
        std::mem::swap(&mut aa, &mut nn);
        if aa % 4 == 3 && nn % 4 == 3 {
            result = -result;
        }
        aa %= nn;
    }
    Ok(if nn == 1 { result as f64 } else { 0.0 })
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
    fn lucas_matches_known_values() {
        assert_eq!(lucas(0.0).unwrap(), 2.0);
        assert_eq!(lucas(1.0).unwrap(), 1.0);
        assert_eq!(lucas(2.0).unwrap(), 3.0);
        assert_eq!(lucas(10.0).unwrap(), 123.0);
        assert_eq!(lucas(1500.0), Err(CalcError::Overflow));
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

    #[test]
    fn primorial_matches_known_values() {
        assert_eq!(primorial(0.0).unwrap(), 1.0);
        assert_eq!(primorial(1.0).unwrap(), 1.0);
        assert_eq!(primorial(5.0).unwrap(), 2.0 * 3.0 * 5.0);
        assert_eq!(primorial(10.0).unwrap(), 2.0 * 3.0 * 5.0 * 7.0);
    }

    #[test]
    fn digitsum_matches_known_values() {
        assert_eq!(digitsum(12345.0).unwrap(), 15.0);
        assert_eq!(digitsum(0.0).unwrap(), 0.0);
        assert_eq!(digitsum(9.0).unwrap(), 9.0);
    }

    #[test]
    fn digitalroot_matches_known_values() {
        assert_eq!(digitalroot(0.0).unwrap(), 0.0);
        assert_eq!(digitalroot(9.0).unwrap(), 9.0);
        assert_eq!(digitalroot(12345.0).unwrap(), 6.0);
        assert_eq!(digitalroot(99.0).unwrap(), 9.0);
    }

    #[test]
    fn ispalindrome_detects_palindromes() {
        assert_eq!(ispalindrome(12321.0).unwrap(), 1.0);
        assert_eq!(ispalindrome(12345.0).unwrap(), 0.0);
        assert_eq!(ispalindrome(0.0).unwrap(), 1.0);
        assert_eq!(ispalindrome(7.0).unwrap(), 1.0);
    }

    #[test]
    fn nextprime_matches_known_values() {
        assert_eq!(nextprime(0.0).unwrap(), 2.0);
        assert_eq!(nextprime(2.0).unwrap(), 3.0);
        assert_eq!(nextprime(10.0).unwrap(), 11.0);
        assert_eq!(nextprime(13.0).unwrap(), 17.0);
    }

    #[test]
    fn new_functions_domain_errors_for_negative_or_fractional_input() {
        assert!(matches!(primorial(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(digitsum(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(digitalroot(1.5), Err(CalcError::DomainError(_))));
        assert!(matches!(ispalindrome(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(nextprime(-1.0), Err(CalcError::DomainError(_))));
    }

    #[test]
    fn triangular_matches_known_values() {
        assert_eq!(triangular(0.0).unwrap(), 0.0);
        assert_eq!(triangular(1.0).unwrap(), 1.0);
        assert_eq!(triangular(10.0).unwrap(), 55.0);
    }

    #[test]
    fn pentagonal_matches_known_values() {
        assert_eq!(pentagonal(0.0).unwrap(), 0.0);
        assert_eq!(pentagonal(1.0).unwrap(), 1.0);
        assert_eq!(pentagonal(10.0).unwrap(), 145.0);
    }

    #[test]
    fn hexagonal_matches_known_values() {
        assert_eq!(hexagonal(0.0).unwrap(), 0.0);
        assert_eq!(hexagonal(1.0).unwrap(), 1.0);
        assert_eq!(hexagonal(10.0).unwrap(), 190.0);
    }

    #[test]
    fn figurate_number_domain_errors() {
        assert!(matches!(triangular(-1.0), Err(CalcError::DomainError(_))));
        assert!(matches!(pentagonal(1.5), Err(CalcError::DomainError(_))));
        assert!(matches!(hexagonal(-1.0), Err(CalcError::DomainError(_))));
    }

    #[test]
    fn carmichael_matches_known_values() {
        assert_eq!(carmichael(1.0).unwrap(), 1.0);
        assert_eq!(carmichael(8.0).unwrap(), 2.0);
        assert_eq!(carmichael(15.0).unwrap(), 4.0);
        assert_eq!(carmichael(561.0).unwrap(), 80.0);
    }

    #[test]
    fn carmichael_rejects_zero() {
        assert!(matches!(carmichael(0.0), Err(CalcError::DomainError(_))));
    }

    #[test]
    fn aliquot_matches_known_values() {
        assert_eq!(aliquot(1.0).unwrap(), 0.0);
        assert_eq!(aliquot(6.0).unwrap(), 6.0); // 6 is perfect: 1+2+3 = 6
        assert_eq!(aliquot(10.0).unwrap(), 8.0); // 1+2+5 = 8
        assert_eq!(aliquot(7.0).unwrap(), 1.0); // prime: only divisor is 1
    }

    #[test]
    fn isamicable_finds_known_pair() {
        // 220 and 284 are the smallest amicable pair.
        assert_eq!(isamicable(220.0).unwrap(), 1.0);
        assert_eq!(isamicable(284.0).unwrap(), 1.0);
    }

    #[test]
    fn isamicable_rejects_perfect_and_primes() {
        assert_eq!(isamicable(6.0).unwrap(), 0.0); // perfect, not amicable
        assert_eq!(isamicable(7.0).unwrap(), 0.0); // prime
        assert_eq!(isamicable(10.0).unwrap(), 0.0);
    }

    #[test]
    fn iscoprime_matches_known_values() {
        assert_eq!(iscoprime(14.0, 15.0).unwrap(), 1.0);
        assert_eq!(iscoprime(14.0, 21.0).unwrap(), 0.0);
        assert_eq!(iscoprime(1.0, 5.0).unwrap(), 1.0);
    }

    #[test]
    fn iscoprime_rejects_both_zero() {
        assert!(matches!(
            iscoprime(0.0, 0.0),
            Err(CalcError::DomainError(_))
        ));
    }

    #[test]
    fn order_matches_known_values() {
        // 2 has order 4 mod 5 since 2^4 = 16 = 1 (mod 5), and no smaller
        // power works (2, 4, 3, 1).
        assert_eq!(order(2.0, 5.0).unwrap(), 4.0);
        assert_eq!(order(1.0, 7.0).unwrap(), 1.0);
    }

    #[test]
    fn order_rejects_non_coprime_args() {
        assert!(matches!(order(2.0, 4.0), Err(CalcError::DomainError(_))));
        assert!(matches!(order(2.0, 1.0), Err(CalcError::DomainError(_))));
    }

    #[test]
    fn jacobi_matches_known_values() {
        // Legendre symbol cases (n prime): known quadratic residues mod 7
        // are {1, 2, 4}.
        assert_eq!(jacobi(1.0, 7.0).unwrap(), 1.0);
        assert_eq!(jacobi(2.0, 7.0).unwrap(), 1.0);
        assert_eq!(jacobi(3.0, 7.0).unwrap(), -1.0);
        assert_eq!(jacobi(7.0, 7.0).unwrap(), 0.0);
        // A well-known composite-modulus example.
        assert_eq!(jacobi(1001.0, 9907.0).unwrap(), -1.0);
    }

    #[test]
    fn jacobi_rejects_even_or_non_positive_modulus() {
        assert!(matches!(jacobi(1.0, 8.0), Err(CalcError::DomainError(_))));
        assert!(matches!(jacobi(1.0, -3.0), Err(CalcError::DomainError(_))));
    }
}
