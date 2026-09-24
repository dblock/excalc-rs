# Statistics

## Variadic (any number of arguments)

| Function | Aliases | Meaning | Example | Result |
|----------|---------|---------|---------|--------|
| `sum(a, b, ...)` | | Sum of all arguments | `sum(1, 2, 3, 4)` | `10` |
| `average(a, b, ...)` | `avg` | Arithmetic mean (requires at least 1 argument) | `average(2, 4, 6)` | `4` |
| `product(a, b, ...)` | `prod` | Product of all arguments | `product(1, 2, 3, 4)` | `24` |
| `min(a, b, ...)` | | Smallest argument (requires at least 1 argument) | `min(3, 1, 2)` | `1` |
| `max(a, b, ...)` | | Largest argument (requires at least 1 argument) | `max(3, 1, 2)` | `3` |
| `median(a, b, ...)` | | Middle value once sorted (average of the two middle values if the count is even) | `median(3, 1, 2)` | `2` |
| `mode(a, b, ...)` | | Most frequently occurring value (ties broken towards the smallest) | `mode(1, 2, 2, 3)` | `2` |
| `variance(a, b, ...)` | | Population variance | `variance(2, 4, 4, 4, 5, 5, 7, 9)` | `4` |
| `stddev(a, b, ...)` | | Population standard deviation, `sqrt(variance(...))` | `stddev(2, 4, 4, 4, 5, 5, 7, 9)` | `2` |
| `percentile(p, a, b, ...)` | | `p`-th percentile of the data (`p` in `[0, 100]`, linear interpolation between closest ranks) | `percentile(50, 1, 2, 3)` | `2` |
| `covariance(x1, y1, x2, y2, ...)` | | Population covariance of two data series given as interleaved `(x, y)` pairs | `covariance(1, 2, 2, 4, 3, 6)` | `~1.333` |
| `correlation(x1, y1, x2, y2, ...)` | | Pearson correlation coefficient of two data series given as interleaved `(x, y)` pairs | `correlation(1, 2, 2, 4, 3, 6)` | `1` |
| `skewness(a, b, ...)` | | Population skewness (third standardized moment); positive for a longer right tail, negative for a longer left tail | `skewness(1, 2, 2, 3, 10)` | `~1.361` |
| `kurtosis(a, b, ...)` | | Population excess kurtosis (fourth standardized moment minus `3`); `0` matches the normal distribution | `kurtosis(1, 2, 3, 4, 5)` | `-1.3` |

## Fixed-arity

| Function | Meaning | Domain | Example | Result |
|----------|---------|--------|---------|--------|
| `harmonic(n)` | Harmonic number `H(n) = 1 + 1/2 + 1/3 + ... + 1/n` | `n` a positive integer | `harmonic(4)` | `~2.083` |
| `binom(n, k)` | Binomial coefficient, "n choose k" | `n, k` non-negative integers, `k <= n` | `binom(5, 2)` | `10` |
