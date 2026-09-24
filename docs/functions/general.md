# General / Rounding Functions

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
| `sign(x)` | Sign of `x`: `-1`, `0`, or `1` | `R -> {-1, 0, 1}` | `sign(-5)` | `-1` |
| `clamp(x, lo, hi)` | Restricts `x` to `[lo, hi]` | `R x R x R -> R`, `lo <= hi` | `clamp(15, 0, 10)` | `10` |
| `lerp(a, b, t)` | Linear interpolation from `a` to `b` at `t` | `R x R x R -> R` | `lerp(0, 10, 0.5)` | `5` |
| `roundto(x, n)` | Rounds `x` to `n` decimal places (`n` may be negative) | `R x Z -> R` | `roundto(3.14159, 2)` | `3.14` |
| `floordiv(a, b)` | Integer division rounding towards negative infinity | `R x R -> Z`, `b != 0` | `floordiv(-7, 2)` | `-4` |
| `mod2(a, b)` | Floored modulo; remainder takes the sign of `b` | `R x R -> R`, `b != 0` | `mod2(-7, 3)` | `2` |

Notes:

- `round` uses "round half up" (`floor(x + 0.5)`), matching the three worked examples in the original manual: `round(-2.1) = -2`, `round(-2.5) = -2`, `round(-2.6) = -3`. Ties round towards positive infinity, not Rust's `f64::round` (which rounds ties away from zero, so `(-2.5_f64).round() == -3.0`).
- `intg` and `trunc` are documented identically in the original manual (`Intg` and `Trunc` both return "the integer portion of a value"), so this port treats `intg` as a plain alias of `trunc` rather than inventing a second, subtly different behavior.
- `random(x)` requires `x >= 0`; `random(0)` deterministically returns `0`.
- `roundto` uses the same round-half-up convention as `round`; `n` must be a whole number (positive, negative, or zero) or it's a domain error.
- `floordiv`/`mod2` complement the existing `/` and `mod` operators, which truncate towards zero (so their remainder takes the sign of the dividend, e.g. `-7 mod 3 == -1`); `floordiv`/`mod2` instead round/remainder towards the divisor's sign, the convention used by Python's `//`/`%` and many number-theoretic algorithms.
