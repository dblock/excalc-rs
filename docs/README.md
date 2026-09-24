# Documentation

Reference documentation for excalc's grammar and function catalog, organized by category. See the top-level [DESIGN.md](../DESIGN.md) for the overall scope, grammar precedence, and numeric model decisions.

Each page below marks its functions as **v1** (implemented, usable today) or **planned** (not yet implemented).

- [Operators](functions/operators.md) — arithmetic, root, factorial, percent (v1)
- [Standard math](functions/standard-math.md) — trig, hyperbolic, log/exponential functions (v1)
- [Statistics](functions/statistics.md) — sum, average, min/max, combinatorics (v1)
- [General / rounding](functions/general.md) — abs, frac, round, trunc, ceil, floor, random (v1)
- [Comparison and logical operators](functions/logic.md) — `= > <`, `xor xnor and nand or nor not shl shr &` (v1)
- [Number theory](functions/number-theory.md) — gcd, lcm, primes, Fibonacci, Mersenne, perfect numbers, Möbius, Fermat (v1)
- [Advanced / special functions](functions/advanced.md) — gamma, beta, elliptic integrals, Pochhammer, integral special functions (v1); general-purpose numeric integration (planned)
- [Financial](functions/financial.md) — present/future value, payments, depreciation

Formulas, domains, and examples for the planned pages above are transcribed from the original [Expression Calculator 2.43 Users Guide](../HISTORY.md), the Vestris Inc. Pascal/Delphi manual this project is a "spirit port" of.

For CLI usage and installation, see the [README](../README.md).
