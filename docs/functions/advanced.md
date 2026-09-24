# Advanced / Special Functions

**Status: planned, not yet implemented.** See [../../AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions. Formulas and examples below are sourced from the "VI. Special Functions and Series" and "VIII. Integrals" chapters of the original [manual](../../HISTORY.md).

## Special functions

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `gamma(x)` | Euler's Gamma function `Γ(x) = Int(e^(-t) * t^(x-1), t, 0, Infinity)` for positive `x`; extended to negative/non-integer `x` via `Γ(x-1) = Γ(x) / (x-1)`. Note `Γ(1/2) = sqrt(pi)` and `x! = Γ(x+1)` | `R \ Z- -> R` | `gamma(0.5)` → `1.77244070463831` |
| `beta(a, b)` | Euler's Beta function, defined from the product of two Gamma functions (itself a double integral) | `R x R -> R` | `beta(1, 2)` → `0.5` |
| `pochhammer(x, n)` | The Pochhammer symbol / rising factorial, `x * (x+1) * ... * (x+n-1)` | `R x R -> R` | `pochhammer(5, 3)` → `210` |
| `ellipticE(k)` / `ellipticE(k, z)` | Incomplete elliptic integral of the second kind, `Int(sqrt(1 - k^2*t^2) / sqrt(1 - t^2), t, 0, z)`; `ellipticE(k) = ellipticE(k, 1)` | `R x R -> R` | `ellipticE(1)` → `1` |
| `ellipticF(k)` / `ellipticF(k, z)` | Incomplete elliptic integral of the first kind — the original manual calls the complete/incomplete pair `EllipticK`, `EllipticK(k, z) = Int(1 / sqrt(1-t^2) / sqrt(1-k^2*t^2), t, 0, z)`, `EllipticK(k) = EllipticK(k, 1)` | `R x R -> R` | `ellipticF(0.01)` → `1.57083559891215` |
| `ellipticCE(k)` | Complementary complete elliptic integral of the second kind, `EllipticE(1, sqrt(1-k^2))` | `R -> R` | `ellipticCE(1)` → `1.57079632679490` |
| `ellipticCK(k)` | Complementary complete elliptic integral of the first kind, `EllipticK(1, sqrt(1-k^2))` | `R -> R` | `ellipticCK(1)` → `5.99158934050168` |
| `bth(x, y, n)` | Binomial theorem `(x+y)^n`, computed via the iterative Pascal's-triangle-style expansion (finite sum of `n+1` terms) | `R x R x N -> R` | `bth(2, 3, 4)` → `625` |
| `bman(x, y, n)` | Binomial theorem `(x+y)^n`, computed the "standard" way (add then raise to a power) | `R x R x R -> R` | `bman(2, 3, 4)` → `625` |

## Integral special functions

These are named integrals with closed-form or well-known series approximations, distinct from the general-purpose quadrature rules below:

| Function | Meaning | Formula |
|----------|---------|---------|
| `dilog(x)` | Dilogarithm integral | `Int(ln(t) / (1-t), t, 1, x)` |
| `dawson(x)` | Dawson integral | `exp(-x^2) * Int(exp(t^2), t, 0, x)` |
| `erf(x)` | Error function | `2/sqrt(pi) * Int(exp(-t^2), t, 0, x)` |
| `erfc(x)` | Complementary error function | `1 - erf(x)` |
| `si(x)` | Sine integral | `Int(sin(t)/t, t, 0, x)` |
| `ssi(x)` | Shifted sine integral | `si(x) - pi/2` |
| `ci(x)` | Cosine integral | `gamma(x) + ln(x) + Int((cos(t)-1)/t, t, 0, x)` |
| `chi(x)` | Hyperbolic cosine integral | `gamma(x) + ln(x) + Int((cosh(t)-1)/t, t, 0, x)` |
| `fresnelC(x)` | Fresnel cosine integral | `Int(cos(pi/2 * t^2), t, 0, x)` |
| `fresnelS(x)` | Fresnel sine integral | `Int(sin(pi/2 * t^2), t, 0, x)` |
| `fresnelF(x)` | Fresnel cosine auxiliary | derived from `fresnelC` (see manual for full formula) |
| `fresnelG(x)` | Fresnel sine auxiliary | derived from `fresnelS` (see manual for full formula) |

## Numeric integration

Named quadrature rules for numerically integrating an expression over a variable and range:

| Rule | Meaning |
|------|---------|
| `trapezoid(...)` | Trapezoidal rule (order 2) |
| `simpson(...)` | Simpson's rule (order 4) |
| `newton(...)` | Newton-Cotes rule (order 4) |
| `boole(...)` | Boole's rule (order 6) |
| `ordersix(...)` | 6th-order Newton-Cotes rule |
| `weddle(...)` | Weddle's rule (order 8) — the manual example (`weddle(log(x^3), x, 1, 10, 64)`) gets 12 correct decimals vs. 5 for `simpson` at the same step count |
| `gauss(...)` | Gaussian quadrature |
| `int(...)` | "Exact" Gauss integral approximation, automatically picking precision based on the requested tolerance; the manual notes this is the most precise built-in method short of symbolic integration (Maple/Mathematica) |

Syntax for all of the above (per the manual): `method(expression, variable, lowerBound, upperBound [, step_or_tolerance])`.
