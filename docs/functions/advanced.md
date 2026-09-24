# Advanced / Special Functions

**Status: planned, not yet implemented.** See [../../AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions.

## Special functions

| Function | Meaning |
|----------|---------|
| `gamma(x)` | The gamma function `Γ(x)` |
| `beta(a, b)` | The beta function `B(a, b)` |
| `pochhammer(x, n)` | The Pochhammer symbol / rising factorial `(x)_n` |
| `ellipticE(k)` / `ellipticE(k, z)` | Incomplete elliptic integral of the second kind |
| `ellipticF(k)` / `ellipticF(k, z)` | Incomplete elliptic integral of the first kind |

## Numeric integration

Named quadrature rules for numerically integrating an expression over a variable and range:

| Rule | Meaning |
|------|---------|
| `trapezoid(...)` | Trapezoidal rule |
| `simpson(...)` | Simpson's rule |
| `newton(...)` | Newton-Cotes rule |
| `boole(...)` | Boole's rule |
| `ordersix(...)` | 6th-order Newton-Cotes rule |
| `weddle(...)` | Weddle's rule |
| `gauss(...)` | Gaussian quadrature |
| `int(...)` | Automatically picks a rule based on the requested tolerance |
