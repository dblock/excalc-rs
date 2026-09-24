# Combinatorics

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `factorial(n)` | `n!`, product of all positive integers up to `n` (same as postfix `n!`, but usable as an ordinary function argument) | `N -> N` | `factorial(5)` → `120` |
| `perm(n, r)` | Number of ways to arrange `r` items out of `n` distinct items, order matters | `n, r` non-negative integers, `r <= n` | `perm(5, 2)` → `20` |
| `catalan(n)` | The `n`-th Catalan number | `N -> N` | `catalan(3)` → `5` |
| `multinomial(n, k1, k2, ...)` | Multinomial coefficient `n! / (k1! k2! ... km!)`, generalizing [`binom`](statistics.md) to more than two groups | `k1 + k2 + ... + km = n`, all non-negative integers | `multinomial(10, 2, 3, 5)` → `2520` |
| `stirling1(n, k)` | Unsigned Stirling number of the first kind, `\|s(n, k)\|`: permutations of `n` elements with exactly `k` cycles | `N x N -> N` | `stirling1(4, 2)` → `11` |
| `stirling2(n, k)` | Stirling number of the second kind, `S(n, k)`: ways to partition `n` elements into exactly `k` non-empty subsets | `N x N -> N` | `stirling2(4, 2)` → `7` |
| `derangement(n)` | Number of derangements of `n` elements (permutations with no fixed points) | `N -> N` | `derangement(5)` → `44` |
| `bell(n)` | The `n`-th Bell number: ways to partition `n` elements into any number of non-empty subsets | `N -> N` | `bell(5)` → `52` |

See also [`binom(n, k)`](statistics.md), the two-group special case ("n choose k").

## Notes and implementation details

- `factorial`/`perm`/`catalan` all guard against overflow by multiplying incrementally and checking for `f64` infinity at each step, rather than computing a naive factorial and dividing afterward.
- `catalan(n)` is computed via the multiplicative recurrence `C(n) = C(n-1) * 2(2n-1) / (n+1)` rather than `binom(2n, n) / (n+1)` directly, to avoid needlessly overflowing on the way to a modestly large `n`.
- `multinomial` requires `k1 + k2 + ... + km = n`; a mismatched sum is a domain error. Internally it's computed as a running product of binomial coefficients (`binom(n, k1) * binom(n - k1, k2) * ...`), which is algebraically equivalent to `n! / (k1! k2! ... km!)` but overflows less readily.
- `stirling1`/`stirling2` use the standard triangular recurrences (`|s(n,k)| = |s(n-1,k-1)| + (n-1)|s(n-1,k)|` and `S(n,k) = k*S(n-1,k) + S(n-1,k-1)`), building up one row of a Pascal's-triangle-like table rather than a naive combinatorial sum; both return `0` if `k > n`.
- `derangement(n)` uses the linear recurrence `D(n) = (n-1)(D(n-1) + D(n-2))` rather than the `n! * sum((-1)^k/k!)` closed form, avoiding intermediate factorial overflow.
- `bell(n) = sum_{k=0}^{n} stirling2(n, k)`, reusing the Stirling-number-of-the-second-kind implementation directly.
