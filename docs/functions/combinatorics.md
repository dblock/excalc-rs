# Combinatorics

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `factorial(n)` | `n!`, product of all positive integers up to `n` (same as postfix `n!`, but usable as an ordinary function argument) | `N -> N` | `factorial(5)` → `120` |
| `perm(n, r)` | Number of ways to arrange `r` items out of `n` distinct items, order matters | `n, r` non-negative integers, `r <= n` | `perm(5, 2)` → `20` |
| `catalan(n)` | The `n`-th Catalan number | `N -> N` | `catalan(3)` → `5` |
| `multinomial(n, k1, k2, ...)` | Multinomial coefficient `n! / (k1! k2! ... km!)`, generalizing [`binom`](statistics.md) to more than two groups | `k1 + k2 + ... + km = n`, all non-negative integers | `multinomial(10, 2, 3, 5)` → `2520` |

See also [`binom(n, k)`](statistics.md), the two-group special case ("n choose k").

## Notes and implementation details

- `factorial`/`perm`/`catalan` all guard against overflow by multiplying incrementally and checking for `f64` infinity at each step, rather than computing a naive factorial and dividing afterward.
- `catalan(n)` is computed via the multiplicative recurrence `C(n) = C(n-1) * 2(2n-1) / (n+1)` rather than `binom(2n, n) / (n+1)` directly, to avoid needlessly overflowing on the way to a modestly large `n`.
- `multinomial` requires `k1 + k2 + ... + km = n`; a mismatched sum is a domain error. Internally it's computed as a running product of binomial coefficients (`binom(n, k1) * binom(n - k1, k2) * ...`), which is algebraically equivalent to `n! / (k1! k2! ... km!)` but overflows less readily.
