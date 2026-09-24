# Probability Distributions

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `binomialpdf(n, p, k)` | `P(X = k)` for `X ~ Binomial(n, p)` | `N x [0,1] x N -> [0,1]` | `binomialpdf(10, 0.5, 5)` → `~0.246` |
| `binomialcdf(n, p, k)` | `P(X <= k)` for `X ~ Binomial(n, p)` | `N x [0,1] x N -> [0,1]` | `binomialcdf(10, 0.5, 5)` → `~0.623` |
| `poissonpdf(lambda, k)` | `P(X = k)` for `X ~ Poisson(lambda)` | `R+ x N -> [0,1]` | `poissonpdf(4, 2)` → `~0.1465` |
| `poissoncdf(lambda, k)` | `P(X <= k)` for `X ~ Poisson(lambda)` | `R+ x N -> [0,1]` | `poissoncdf(4, 2)` → `~0.2381` |
| `normalpdf(x, mean, sd)` | Density of `X ~ Normal(mean, sd)` at `x` (not a probability; can exceed `1`) | `R x R x R+ -> R+` | `normalpdf(0, 0, 1)` → `~0.3989` |
| `normalcdf(x, mean, sd)` | `P(X <= x)` for `X ~ Normal(mean, sd)` | `R x R x R+ -> [0,1]` | `normalcdf(1.96, 0, 1)` → `~0.975` |
| `zscore(x, mean, sd)` | Standard score: how many standard deviations `x` is from `mean` | `R x R x R+ -> R` | `zscore(85, 70, 10)` → `1.5` |

## Notes and implementation details

- `binomialpdf` reuses `binom(n, k)` (the existing `n choose k` function) for the combinatorial coefficient.
- `binomialcdf`/`poissoncdf` sum the corresponding pdf from `0` to `k`; there's no closed-form shortcut, so this is `O(k)`.
- `normalcdf` is computed via `Phi(x) = 0.5 * (1 + erf((x - mean) / (sd * sqrt(2))))`, reusing the existing `erf` function rather than a separate numeric integration.
- All distribution parameters are validated: `p` must be in `[0, 1]`, `k` must be a non-negative integer (and `<= n` for the binomial functions), `lambda`/`sd` must be positive.
