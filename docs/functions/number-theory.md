# Number Theory

**Status: planned, not yet implemented.** See [../../AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions. Formulas and examples below are sourced from the "IV. General Functions" and "VII. Primes and Numbers" chapters of the original [manual](../../HISTORY.md).

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `gcd(a, b, ...)` | Greatest common divisor of all arguments | `Z x Z -> Z` | `gcd(3213, 24)` → `3` |
| `lcm(a, b, ...)` | Least common multiple of all arguments | `Z x Z -> Z` | `lcm(14, 4)` → `28` |
| `fib(n)` | The `n`-th Fibonacci number | `N -> N` | `fib(56)` → `2.258e11` |
| `isprime(n)` | The closest prime `<= n` (returns `n` itself if `n` is prime) — the original manual calls this `Prime` | `R -> N` | `isprime(86)` → `83` |
| `moebius(n)` | Möbius function `μ(n)` — `1` for an even number of distinct prime factors, `-1` for an odd number, `0` if any prime factor repeats | `N -> {-1, 0, 1}` | `moebius(2)` → `-1` |
| `mersenne(p)` | The Mersenne number `2^p - 1` for generator `p` (prime iff `p` is a Mersenne prime) | `[3, 2^32] -> {3, 7, 31, ...}` | `mersenne(145)` → `127` |
| `perfect(n)` | The closest known perfect number `<= n` (a number equal to the sum of its own positive divisors, e.g. `6 = 1+2+3`) | `N+ -> N+` | `perfect(1231)` → `496` |
| `fermat(k)` | The `k`-th known Fermat prime, `2^(2^k) + 1` (only known for `k <= 4`) | `N -> R` | `fermat(4)` → `65537` |
| `safeprime(n)` | The smallest safe prime `>= n` (a prime `p` where `(p-1)/2` is also prime) | `Z -> N` | `safeprime(12)` → `23` |

## Formulas from the original manual

- `gcd(x, y)` uses Euclid's algorithm: repeatedly `t = x mod y; x = y; y = t;` until `y = 0`, result is `x`.
- `lcm(x, y) = (x | gcd(x, y)) * y` (`|` is integer division).
- `fib` is computed iteratively via a running pair of sums, not naive recursion or matrix exponentiation, to avoid overflow/perf issues for large `n`.
- `perfect` relates directly to Mersenne primes: if `2^n - 1` is prime, then `2^(n-1) * (2^n - 1)` is an even perfect number, and every known even perfect number arises this way.

## Additional functions in the original manual not yet tracked above

These appear in the "Primes and Numbers" chapter but aren't yet represented as planned functions in this project — worth considering alongside the ones above:

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `primec(n)` | Count of primes `<= n` (`1` counts as prime in the original) | `R+ -> Z` | `primec(86)` → `23` |
| `primen(n)` | The `n`-th prime, with `primen(1) = 1` | `Z -> Z` | `primen(86)` → `443` |
| `mersennegen(n)` | Closest Mersenne generator (prime `p` such that `2^p - 1` is a Mersenne prime) `<= n` | `[3, 2^32] -> {2, 3, 5, 7, ...}` | `mersennegen(12)` → `7` |
| `mersgen(p)` | The Mersenne number for a given Mersenne generator `p` | `[Primes] -> {2, 7, 31, ...}` | `mersgen(7)` → `127` |
| `genmers(m)` | The generator for a given Mersenne number `m` | `[Mersenne] -> {primes}` | `genmers(127)` → `7` |
| `sigma(n, k)` | Sum of the `k`-th power of positive divisors of `n` | `Z x Z -> Z` | `sigma(100, 0)` → `9` |
| `tau(n)` | Sum of positive divisors of `n` (i.e. `sigma(n, 1)`) | `Z -> Z` | `tau(9)` → `13` |
| `phi(n)` / `eind(n)` | Euler's totient function, the count of positive integers `<= n` coprime to `n` | `Z -> N` | `phi(12)` → `4` |
