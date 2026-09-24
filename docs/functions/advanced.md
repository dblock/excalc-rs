# Advanced / Special Functions

Formulas and examples below are sourced from the "VI. Special Functions and Series" chapter of the original [manual](../../HISTORY.md); see [AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions and the note on numeric integration below.

## A note on numeric integration

In the original Pascal engine, most of these functions (`ellipticE`, `ellipticF`, `dilog`, `erf`, `erfc`, `si`, `ssi`, `ci`, `chi`, `dawson`, `fresnelC`, `fresnelS`) are thin wrappers around a generic numeric integration engine (`int(expression, variable, lowerBound, upperBound, tolerance)`), and `gamma` itself is a hand-rolled Riemann-sum integral (`ShortGamma`/`Factor` in `MCalc.pas`) rather than a closed-form approximation. This category ports the same underlying formulas, but evaluates the integrals with a private, ad hoc adaptive Simpson's-rule integrator rather than the general-purpose quadrature engine now implemented in [Numeric integration](numeric-integration.md) (`gamma` keeps its original fixed-step Riemann sum, since that's what the pre-existing worked example was computed with).

**TODO:** now that general numeric integration exists (see [numeric-integration.md](numeric-integration.md)), consider routing these through the shared engine, or through dedicated closed-form approximations (e.g. the Lanczos approximation for `gamma`, rational/continued-fraction approximations for `erf`) for better precision and performance than the current adaptive-Simpson/Riemann-sum stand-ins.

## Special functions

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `gamma(x)` | Euler's Gamma function `Γ(x) = Int(e^(-t) * t^(x-1), t, 0, Infinity)` for positive `x`; extended to negative/non-integer `x` via `Γ(x-1) = Γ(x) / (x-1)`. Note `Γ(1/2) = sqrt(pi)` and `x! = Γ(x+1)`. Domain error at `x <= 0` when `x` is an integer (a pole of the true Gamma function) | `R \ Z<=0 -> R` | `gamma(0.5)` → `1.77244070463831` |
| `beta(a, b)` | Euler's Beta function, `Γ(a) * Γ(b) / Γ(a+b)` | `R x R -> R` | `beta(1, 2)` → `0.5` |
| `pochhammer(x, n)` | The Pochhammer symbol / rising factorial, `Γ(x+n) / Γ(x)`; domain error at `x = n = 0` (the ratio is `0/0`) | `R x R -> R` | `pochhammer(5, 3)` → `210` |
| `ellipticE(k)` / `ellipticE(k, z)` | Incomplete elliptic integral of the second kind, `Int(sqrt(1 - k^2*t^2) / sqrt(1 - t^2), t, 0, z)`; `ellipticE(k) = ellipticE(k, 1)`. Domain error if `z` is outside `[-1, 1]` or `abs(k*z) > 1` | `R x [-1, 1] -> R` | `ellipticE(1)` → `1` |
| `ellipticF(k)` / `ellipticF(k, z)` (alias `ellipticK`) | Incomplete elliptic integral of the first kind — the original manual calls the complete/incomplete pair `EllipticK`, `EllipticK(k, z) = Int(1 / sqrt(1-t^2) / sqrt(1-k^2*t^2), t, 0, z)`, `EllipticK(k) = EllipticK(k, 1)`. Domain error if `z` is outside `[-1, 1]` or `abs(k*z) > 1`; diverges (numeric overflow) as `abs(k*z) -> 1` | `R x [-1, 1] -> R` | `ellipticF(0.01)` → `1.57083559891215` |
| `ellipticCE(k)` | Complementary complete elliptic integral of the second kind, `ellipticE(sqrt(1-k^2), 1)` | `[-1, 1] -> R` | `ellipticCE(1)` → `1.57079632679490` |
| `ellipticCK(k)` | Complementary complete elliptic integral of the first kind, `ellipticF(sqrt(1-k^2), 1)`; diverges (numeric overflow) at `k = 0` | `[-1, 1] -> R` | `ellipticCK(1)` → `1.57079632679490` |
| `bth(x, y, n)` | Binomial theorem `(x+y)^n`, computed via the iterative Pascal's-triangle-style expansion (finite sum of `n+1` terms) for integer `n`, `Power(x+y, n)` directly otherwise; negative integer `n` uses `1 / bth(x, y, -n)` | `R x R x N -> R` | `bth(2, 3, 4)` → `625` |
| `bman(x, y, n)` | Binomial theorem `(x+y)^n`, computed the "standard" way (add then raise to a power) | `R x R x R -> R` | `bman(2, 3, 4)` → `625` |

## Integral special functions

These are named integrals, most of which are only computable numerically (no elementary closed form):

| Function | Meaning | Formula | Example |
|----------|---------|---------|---------|
| `dilog(x)` | Dilogarithm integral | `Int(ln(t) / (1-t), t, 1, x)` | `dilog(1)` → `0` |
| `dawson(x)` | Dawson integral | `exp(-x^2) * Int(exp(t^2), t, 0, x)` | `dawson(0)` → `0` |
| `erf(x)` | Error function | `2/sqrt(pi) * Int(exp(-t^2), t, 0, x)` | `erf(1)` → `0.842700792950` |
| `erfc(x)` | Complementary error function | `1 - erf(x)` | `erfc(1)` → `0.157299207050` |
| `si(x)` | Sine integral | `Int(sin(t)/t, t, 0, x)` | `si(1)` → `0.946083070367` |
| `ssi(x)` | Shifted sine integral | `si(x) - pi/2` | `ssi(1)` → `-0.624713256428` |
| `ci(x)` | Cosine integral | `EulerGamma + ln(x) + Int((cos(t)-1)/t, t, 0, x)`, domain error at `x <= 0` | `ci(1)` → `0.337403922901` |
| `chi(x)` | Hyperbolic cosine integral | `EulerGamma + ln(x) + Int((cosh(t)-1)/t, t, 0, x)`, domain error at `x <= 0` | `chi(1)` → `0.837866940980` |
| `fresnelC(x)` | Fresnel cosine integral | `Int(cos(pi/2 * t^2), t, 0, x)` | `fresnelC(1)` → `0.779893400377` |
| `fresnelS(x)` | Fresnel sine integral | `Int(sin(pi/2 * t^2), t, 0, x)` | `fresnelS(1)` → `0.438259147390` |
| `fresnelF(x)` | Fresnel cosine auxiliary | `(1/2 - fresnelS(x))*cos(pi/2*x^2) - (1/2 - fresnelC(x))*sin(pi/2*x^2)` | `fresnelF(1)` → `0.279893400377` |
| `fresnelG(x)` | Fresnel sine auxiliary | `(1/2 - fresnelC(x))*cos(pi/2*x^2) - (1/2 - fresnelS(x))*sin(pi/2*x^2)` | `fresnelG(1)` → `-0.061740852610` |

`EulerGamma` above is the Euler-Mascheroni constant (`≈ 0.5772156649015329`). The original Pascal source reads this from a variable named `G` that the user is expected to set themselves (undefined variables default to `0`) — almost certainly a bug rather than intentional, since `ci`/`chi` are meaningless without the correct constant; this port uses the real mathematical constant directly. `dilog`, `si`, and `ci`/`chi` have removable singularities in their integrands (at `t = 1`, `t = 0`, and `t = 0` respectively), handled explicitly by substituting the analytic limit at that point.

See [Numeric integration](numeric-integration.md) for the general-purpose quadrature functions (`trapezoid`, `simpson`, `newton`, `boole`, `ordersix`, `weddle`, `gauss`, `int`) that these special functions could eventually be routed through (see the TODO above).

Syntax for all of the above (per the manual): `method(expression, variable, lowerBound, upperBound [, step_or_tolerance])`.
