# Numeric Integration

Ported from the "Numeric Integration" chapter of the original [manual](../../HISTORY.md) and the `fSum`/`Tegral`/`Gauss` functions in `MCalc.pas`. See [AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions.

Unlike every other function in this port, these take an **arbitrary expression** and a bare **integration variable** as their first two arguments, rather than plain numbers — the expression is evaluated repeatedly at sample points across `[lowerBound, upperBound]` with the variable substituted each time. All eight share the same shape:

```
rule(expression, variable, lowerBound, upperBound, n_or_tolerance)
```

- `variable` must be a bare, non-reserved variable (`x`, `t`, ... — not `pi`/`e`, not a compound expression); this matches the original's requirement.
- `lowerBound`/`upperBound` can be given in either order (the result is negated if reversed), and are equal to `0` if `lowerBound == upperBound`.
- Any error produced while evaluating `expression` at a sample point (e.g. a domain error from a nested function) propagates out of the integration call.

## Named composite rules

These apply a fixed quadrature formula uniformly across `n` equal-width sub-intervals of `[lowerBound, upperBound]` (`n` must be a positive whole number):

| Function | Meaning | Example |
|----------|---------|---------|
| `trapezoid(expr, var, a, b, n)` (aliases `trapez`, `trapezoide`) | Trapezoidal rule (order 2) | `trapezoid(log(x^3), x, 1, 10, 64)` → `~18.272118017450193` |
| `simpson(expr, var, a, b, n)` | Simpson's rule (order 4) | `simpson(log(x^3), x, 1, 10, 64)` → `~18.274048637598288` |
| `newton(expr, var, a, b, n)` | Newton-Cotes 3/8 rule (order 4) | `newton(log(x^3), x, 1, 10, 64)` → `~18.27404883248503` |
| `boole(expr, var, a, b, n)` | Boole's rule (order 6) | `boole(log(x^3), x, 1, 10, 64)` → `~18.274048988489618` |
| `ordersix(expr, var, a, b, n)` (alias `ordresix`) | 6th-order Newton-Cotes rule | `ordersix(log(x^3), x, 1, 10, 64)` → `~18.274048988543118` |
| `weddle(expr, var, a, b, n)` | Weddle's rule (order 8) | `weddle(log(x^3), x, 1, 10, 64)` → `~18.274048988612115` (the manual's own worked example: matches the true value, `~18.2740489886122`, to 12 decimals, vs. only 5 for `simpson` at the same step count) |

## Adaptive quadrature

| Function | Meaning | Example |
|----------|---------|---------|
| `int(expr, var, a, b, tolerance)` (alias `gauss`) | Numerically integrates `expr` to within `tolerance` (a positive number — smaller means more precise, more sample points), automatically refining until two successive estimates agree | `int(log(x^3), x, 1, 10, 0.0000001)` → `~18.274048987232057` |

The original's `int`/`gauss` (`Tegral`/`Gauss` in `MCalc.pas`) use Hairer's 30-point Gauss-Kronrod quadrature with Aitken extrapolation — a specific, hardcoded-coefficient algorithm. This port instead evaluates them "in spirit" with an adaptive composite Simpson's rule that doubles its sub-interval count until two successive estimates agree within the requested tolerance (the same technique used as a stand-in for [advanced/special functions](advanced.md#a-note-on-numeric-integration) that depend on a numeric integral in the original). A tolerance that can't be reached within the refinement budget (e.g. an unreasonably tight tolerance, or a highly oscillatory/discontinuous integrand) is reported as a numeric overflow error rather than silently returning an imprecise result.

## Deviation from the original

The original's named composite rules (`fSum`) step a `while` loop with a floating-point `<=` comparison (`while xj <= b - h`), which can silently run one sub-interval short or long due to floating-point accumulation. This port instead requires `n` to be a positive whole number and loops exactly `n` times — more predictable, and mathematically equivalent to what the original intended.
