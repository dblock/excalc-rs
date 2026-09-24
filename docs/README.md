# Documentation

Reference documentation for excalc's grammar and function catalog, organized by category. See the top-level [DESIGN.md](../DESIGN.md) for the overall scope, porting status, grammar precedence, and numeric model decisions.

- [Operators](functions/operators.md) — arithmetic, root, factorial, percent
- [Standard math](functions/standard-math.md) — trig, hyperbolic, log/exponential functions
- [Statistics](functions/statistics.md) — sum, average, min/max, combinatorics
- [General / rounding](functions/general.md) — abs, frac, round, trunc, ceil, floor, random
- [Comparison and logical operators](functions/logic.md) — `= > <`, `xor xnor and nand or nor not shl shr &`
- [Number theory](functions/number-theory.md) — gcd, lcm, primes, Fibonacci, Mersenne, perfect numbers, Möbius, Fermat
- [Advanced / special functions](functions/advanced.md) — gamma, beta, elliptic integrals, Pochhammer, integral special functions
- [Numeric integration](functions/numeric-integration.md) — trapezoid, simpson, newton, boole, ordersix, weddle, gauss, int
- [Financial](functions/financial.md) — present/future value, payments, depreciation

Formulas, domains, and examples throughout are transcribed from the original [Expression Calculator 2.43 Users Guide](../HISTORY.md), the Vestris Inc. Pascal/Delphi manual this project is a "spirit port" of. Where a page's content maps to a specific manual chapter:

| Page | Manual chapter(s) |
|------|--------------------|
| [General / rounding](functions/general.md) | IV. General Functions |
| [Comparison and logical operators](functions/logic.md) | II. Operators, III. Logical Operators |
| [Number theory](functions/number-theory.md) | IV. General Functions, VII. Primes and Numbers |
| [Advanced / special functions](functions/advanced.md) | VI. Special Functions and Series |
| [Numeric integration](functions/numeric-integration.md) | Numeric Integration |

For CLI usage and installation, see the [README](../README.md).
