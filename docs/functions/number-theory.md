# Number Theory

**Status: v1 (implemented).** Formulas and examples below are sourced from the "IV. General Functions" and "VII. Primes and Numbers" chapters of the original [manual](../../HISTORY.md).

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `gcd(a, b, ...)` | Greatest common divisor of all arguments | `Z x Z -> Z` | `gcd(3213, 24)` → `3` |
| `lcm(a, b, ...)` | Least common multiple of all arguments | `Z x Z -> Z` | `lcm(14, 4)` → `28` |
| `fib(n)` (alias `fibonacci`) | The `n`-th Fibonacci number | `N -> N` | `fib(56)` → `2.258e11` |
| `isprime(n)` | The closest prime `<= n` (returns `n` itself if `n` is prime) — the original manual calls this `Prime` | `N -> N` | `isprime(86)` → `83` |
| `moebius(n)` | Möbius function `μ(n)` — `1` for an even number of distinct prime factors, `-1` for an odd number, `0` if any prime factor repeats | `N -> {-1, 0, 1}` | `moebius(2)` → `-1` |
| `mersenne(p)` | The Mersenne number `2^p - 1` for exponent `p` | `[1, 1023] -> N` | `mersenne(7)` → `127` |
| `perfect(n)` | The closest known perfect number `<= n` (a number equal to the sum of its own positive divisors, e.g. `6 = 1+2+3`) | `N+ -> N+` | `perfect(1231)` → `496` |
| `fermat(k)` | The `k`-th Fermat number, `2^(2^k) + 1` (only known to be prime for `k <= 4`) | `[0, 9] -> N` | `fermat(4)` → `65537` |
| `safeprime(n)` | The smallest safe prime `>= n` (a prime `p` where `(p-1)/2` is also prime) | `N -> N` | `safeprime(12)` → `23` |
| `primec(n)` | Count of primes `<= n` | `N -> Z` | `primec(86)` → `23` |
| `primen(n)` | The `n`-th prime, with `primen(1) = 1` overriding the conventional `prime(1) = 2` | `N+ -> N` | `primen(86)` → `443` |
| `mersennegen(n)` | Closest known Mersenne generator (prime `p` such that `2^p - 1` is prime) `<= n` | `N -> {2, 3, 5, 7, 13, ...}` | `mersennegen(12)` → `7` |
| `mersgen(p)` | The Mersenne number for a given Mersenne generator `p` | `{2, 3, 5, 7, 13, ...} -> N` | `mersgen(7)` → `127` |
| `genmers(m)` | The generator for a given Mersenne number `m` | `{Mersenne primes} -> {2, 3, 5, 7, 13, ...}` | `genmers(127)` → `7` |
| `sigma(n, k)` | Sum of the `k`-th power of positive divisors of `n` | `N+ x N -> N` | `sigma(100, 0)` → `9` |
| `tau(n)` | Sum of positive divisors of `n` (i.e. `sigma(n, 1)`) | `N+ -> N` | `tau(9)` → `13` |
| `phi(n)` (alias `eind`) | Euler's totient function, the count of positive integers `<= n` coprime to `n` | `N+ -> N` | `phi(12)` → `4` |

## Notes and implementation details

- `gcd(x, y)` uses Euclid's algorithm: repeatedly `t = x mod y; x = y; y = t;` until `y = 0`, result is `x`. Folded pairwise across all arguments for the variadic form.
- `lcm(x, y) = (x / gcd(x, y)) * y`, folded pairwise; any zero argument makes the whole result `0`.
- `fib` is computed iteratively via a running pair of sums, not naive recursion or matrix exponentiation, to avoid overflow/perf issues for large `n`.
- Primality (`isprime`, `mersenne`-family, `safeprime`, `primec`, `primen`) uses a deterministic Miller-Rabin test, valid for the entire `u64` range rather than a hardcoded list of known primes.
- `mersenne(p)` is the literal `2^p - 1`, regardless of whether the result is actually prime; use `mersennegen`/`mersgen`/`genmers` to work specifically with *Mersenne primes* (i.e. Mersenne numbers with a prime exponent that also happen to be prime).
- `mersennegen`/`mersgen`/`genmers` search generators `p` in `[2, 63]` (the largest range for which `2^p - 1` fits exactly in a `u64`), rather than a hardcoded list of known Mersenne primes.
- `perfect` relates directly to Mersenne primes: if `2^p - 1` is prime, then `2^(p-1) * (2^p - 1)` is an even perfect number, and every known even perfect number arises this way.
- `fermat(k)` computes the formula generically for any `k` in `[0, 9]` (`2^512 + 1` at `k = 9` is the largest representable in `f64`), even though only `k <= 4` are known to be prime.
