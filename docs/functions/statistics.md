# Statistics

## Variadic (any number of arguments)

| Function | Aliases | Meaning | Example | Result |
|----------|---------|---------|---------|--------|
| `sum(a, b, ...)` | | Sum of all arguments | `sum(1, 2, 3, 4)` | `10` |
| `average(a, b, ...)` | `avg` | Arithmetic mean (requires at least 1 argument) | `average(2, 4, 6)` | `4` |
| `product(a, b, ...)` | `prod` | Product of all arguments | `product(1, 2, 3, 4)` | `24` |
| `min(a, b, ...)` | | Smallest argument (requires at least 1 argument) | `min(3, 1, 2)` | `1` |
| `max(a, b, ...)` | | Largest argument (requires at least 1 argument) | `max(3, 1, 2)` | `3` |

## Fixed-arity

| Function | Meaning | Domain | Example | Result |
|----------|---------|--------|---------|--------|
| `harmonic(n)` | Harmonic number `H(n) = 1 + 1/2 + 1/3 + ... + 1/n` | `n` a positive integer | `harmonic(4)` | `~2.083` |
| `binom(n, k)` | Binomial coefficient, "n choose k" | `n, k` non-negative integers, `k <= n` | `binom(5, 2)` | `10` |
