# Number Theory

**Status: planned, not yet implemented.** See [../../AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions.

| Function | Meaning |
|----------|---------|
| `gcd(a, b, ...)` | Greatest common divisor of all arguments |
| `lcm(a, b, ...)` | Least common multiple of all arguments |
| `fib(n)` | The `n`-th Fibonacci number |
| `isprime(n)` | The closest prime `<= n` (returns `n` itself if `n` is prime) |
| `moebius(n)` | Möbius function `μ(n)` (`0` if `n` has a squared prime factor, else `(-1)^k` for `k` distinct prime factors) |
| `mersenne(p)` | The Mersenne number `2^p - 1` for generator `p` |
| `perfect(n)` | The closest known perfect number `<= n` |
| `fermat(k)` | The `k`-th known Fermat prime, `2^(2^k) + 1` |
| `safeprime(n)` | The smallest safe prime `>= n` (a prime `p` where `(p-1)/2` is also prime) |
