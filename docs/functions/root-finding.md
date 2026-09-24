# Root Finding

Like the [numeric integration functions](numeric-integration.md), these take an expression and a bare variable rather than plain numbers, evaluating the expression repeatedly at sample points to search for a zero.

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `bisect(expr, var, a, b, tolerance)` | Bisection method: repeatedly halves `[a, b]`, keeping whichever half still brackets a sign change, until the bracket is narrower than `tolerance` | `expr(a)` and `expr(b)` must have opposite signs | `bisect(x^2 - 2, x, 0, 2, 0.000001)` → `~1.4142141342163086` |
| `secant(expr, var, x0, x1, tolerance)` | Secant method: extrapolates a line through the last two samples to estimate the next guess, converging faster than bisection but without a bracketing guarantee | no bracket required, but may fail to converge | `secant(x^2 - 2, x, 0, 2, 0.000001)` → `~1.4142135623730947` |

## Notes and implementation details

- Both share the same 5-argument shape (expression, bare variable, two numeric values, tolerance) as `int`/`gauss`/the composite quadrature rules, and reuse the same argument-validation logic (the 2nd argument must be a bare, non-reserved variable).
- `bisect` requires `expr(a)` and `expr(b)` to have strictly opposite signs (a valid bracket); if either endpoint is exactly zero, it's returned immediately without iterating. A domain error is raised if no sign change is present, or if the iteration budget (200 steps) is exhausted without narrowing below `tolerance`.
- `secant` doesn't require a bracket, so it can fail to converge for a poorly chosen initial pair (e.g. one that makes two consecutive samples equal, or that diverges); both cases surface as an error rather than an infinite loop, bounded by the same 200-step iteration budget as `bisect`.
- Neither function requires (or uses) a derivative, unlike Newton-Raphson, which the calculator doesn't implement since it has no symbolic differentiation.
