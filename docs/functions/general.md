# General / Rounding Functions

Sourced from the "IV. General Functions" chapter of the original [manual](../../HISTORY.md).

| Function | Meaning | Domain | Example | Result |
|----------|---------|--------|---------|--------|
| `abs(x)` | Absolute value | `R -> R+` | `abs(-2)` | `2` |
| `frac(x)` | Fractional part, `x - trunc(x)` | `R -> R` | `frac(1.345)` | `0.345` |
| `intg(x)` | Alias of `trunc` | `R -> Z` | `intg(2.1)` | `2` |
| `round(x)` | Closest integer, ties round towards positive infinity | `R -> Z` | `round(-2.5)` | `-2` |
| `trunc(x)` | Integer portion of a value (towards zero) | `R -> Z` | `trunc(2.1)` | `2` |
| `ceil(x)` | Ceiling, closest integer `>= x` | `R -> Z` | `ceil(-2.1)` | `-2` |
| `floor(x)` | Floor, closest integer `<= x` | `R -> Z` | `floor(-2.1)` | `-3` |
| `random(x)` | Uniformly distributed random number in `[0, x]` | `R+ -> R+` | `random(8)` | non-deterministic, in `[0, 8]` |

Notes:

- `round` uses "round half up" (`floor(x + 0.5)`), matching the three worked examples in the original manual: `round(-2.1) = -2`, `round(-2.5) = -2`, `round(-2.6) = -3`. Ties round towards positive infinity, not Rust's `f64::round` (which rounds ties away from zero, so `(-2.5_f64).round() == -3.0`).
- `intg` and `trunc` are documented identically in the original manual (`Intg` and `Trunc` both return "the integer portion of a value"), so this port treats `intg` as a plain alias of `trunc` rather than inventing a second, subtly different behavior.
- `random(x)` requires `x >= 0`; `random(0)` deterministically returns `0`.
