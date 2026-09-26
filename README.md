Expression Calculator (CLI, MCP, Rust)
======================================

[![Test](https://github.com/dblock/excalc-rs/actions/workflows/test.yml/badge.svg)](https://github.com/dblock/excalc-rs/actions/workflows/test.yml)
[![Coverage Status](https://coveralls.io/repos/github/dblock/excalc-rs/badge.svg?branch=master)](https://coveralls.io/github/dblock/excalc-rs?branch=master)

An expression calculator CLI tool and MCP server built for AI coding agents like Claude and GitHub Copilot to outsource arithmetic, saving tokens and avoiding LLM math hallucinations. Also usable as a one-shot command, piped from stdin, or as an interactive REPL.

A mathematical beast with 214 functions and operators across standard math, trigonometry, statistics, probability distributions, financial (NPV, IRR), number theory (GCD, totient, primality), base conversion, combinatorics, numeric integration, root finding, geometry (n-dimensional distance/dot/norm), unit conversion, and advanced/special functions (gamma, elliptic integrals, dilogarithm, and more).

## Install

```bash
cargo install excalc
```

This installs the `excalc`, `calc` (a shorter alias for `excalc`), and `excalc-mcp` (see [MCP Server](#mcp-server)) binaries to `~/.cargo/bin` (make sure it's on your `PATH`). Requires a [Rust toolchain](https://rustup.rs/). Works on macOS, Linux, and Windows.

### macOS/Linux

Install via [Homebrew](https://brew.sh/):

```bash
brew tap dblock/excalc-rs https://github.com/dblock/excalc-rs
brew install excalc
```

### Windows

Download and run the MSI installer from the [latest release](https://github.com/dblock/excalc-rs/releases/latest) (installs `excalc.exe`, `calc.exe`, and `excalc-mcp.exe`, with options to add them to your `PATH` and to register `excalc-mcp` with [GitHub Copilot CLI](https://docs.github.com/en/copilot/how-tos/copilot-cli) and [Claude Code](https://docs.claude.com/en/docs/claude-code/overview) if either is installed, both on by default), or install it silently from the command line with the [GitHub CLI](https://cli.github.com/):

```powershell
gh release download --repo dblock/excalc-rs --pattern "*.msi" --output excalc.msi
msiexec /i excalc.msi /quiet
```

Or with PowerShell alone (no `gh` required):

```powershell
$asset = (Invoke-RestMethod https://api.github.com/repos/dblock/excalc-rs/releases/latest).assets | Where-Object name -like "*.msi"
Invoke-WebRequest $asset.browser_download_url -OutFile excalc.msi
msiexec /i excalc.msi /quiet
```

## Usage

```
excalc "2 + 2 * 3"
  # 8

excalc "sqrt(16) + sin(pi/2)"
  # 5

excalc "1/0"
  # error, exit code 1

echo "2 + 2 * 3" | excalc
  # 8, also reads the expression from stdin when no argument is given

excalc
  # starts an interactive REPL when run with no expression and stdin is a
  # terminal; type expressions one per line (Tab-completes function names,
  # and shows/advances through argument hints inside a call's parentheses),
  # `help` for usage, `about` for version/author/license info, `vars` to
  # list assigned variables, and `exit`/`quit`/Ctrl-D to leave. Command
  # history persists across sessions in ~/.excalc_history by default;
  # override with --history-file <path>, $EXCALC_HISTORY_FILE, or disable
  # with --no-history.
```

Run `excalc --help` (or `calc --help`) for full usage.

### Examples

See [docs/](docs/README.md) for full reference documentation, including domains and error conditions for each function.

**Arithmetic operators** ([details](docs/functions/operators.md)):

```
excalc "2 + 2 * 3"   # 8    (addition and multiplication, standard precedence)
excalc "10 - 4 / 2"  # 8    (subtraction and division)
excalc "7 mod 3"     # 1    (modulo)
excalc "2 ^ 10"      # 1024 (power)
excalc "2 \ 9"       # 3    (n-th root: the 2nd root of 9)
excalc "5!"          # 120  (factorial)
excalc "50%"         # 0.5  (percent)
excalc "(2 + 3) * 4" # 20   (grouping)
excalc -- "-5 + 3"   # -2   (unary minus needs -- so clap doesn't treat it as a flag)
```

**Number literals** ([details](docs/functions/operators.md)):

```
excalc "1,234 + 1"     # 1,235   (comma thousands-grouping; echoed back in the result)
excalc "1_000_000 / 4" # 250_000 (underscore grouping works the same way)
excalc "sum(1,2)"      # 3       (still 2 args: a group must be exactly 3 digits)
excalc "\$1 + 10"      # $11     (currency symbols: ignored as input, echoed in the result)
excalc "£1,000 + 234"  # £1,234  (currency and grouping combine)
```

Digit literals may use `,` or `_` as a thousands separator (e.g. `1,234` or `1_234_567`); a group after a separator must be exactly 3 digits, so short argument lists like `sum(1,2)` are unaffected. Numbers may also be prefixed with a currency symbol (`$`, `£`, `€`, `¥`), which is ignored for math purposes. Whichever separator/currency is used *first* in an expression is echoed back in the printed result (independently of each other); math across different currencies isn't tracked or rejected — only the first symbol seen anywhere is echoed back. Plain input prints plainly.

**Standard math** ([details](docs/functions/standard-math.md)), all angles in radians (see `deg`/`rad` below for degree conversion):

```
excalc "sqrt(16)"       # 4       (square root)
excalc "cbrt(-8)"       # -2      (cube root)
excalc "nthroot(27, 3)" # 3       (general n-th root)
excalc "isqrt(17)"      # 4       (integer square root)
excalc "ln(e)"          # 1       (natural log)
excalc "log(100)"       # 2       (base 10)
excalc "log2(8)"        # 3       (base 2)
excalc "logn(8, 2)"     # 3       (log base 2 of 8)
excalc "hypot(3, 4)"    # 5       (euclidean distance)
excalc "sin(pi/2)"      # 1       (sine)
excalc "cos(0)"         # 1       (cosine)
excalc "tan(pi/4)"      # ~1      (tangent)
excalc "asin(1)"        # ~1.5708 (arcsine; pi/2)
excalc "acos(0)"        # ~1.5708 (arccosine; pi/2)
excalc "atan(1)"        # ~0.7854 (arctangent; pi/4)
excalc "atan2(1, 1)"    # ~0.7854 (two-argument arctangent; pi/4)
excalc "sinh(1)"        # ~1.1752 (hyperbolic sine)
excalc "cosh(0)"        # 1       (hyperbolic cosine)
excalc "tanh(0)"        # 0       (hyperbolic tangent)
excalc "sec(0)"         # 1       (secant)
excalc "csc(pi/2)"      # 1       (cosecant)
excalc "cot(pi/4)"      # ~1      (cotangent)
excalc "deg(pi)"        # 180     (radians to degrees)
excalc "rad(180)"       # ~3.1416 (degrees to radians; pi)
excalc "sind(30)"       # 0.5     (sine, argument in degrees)
excalc "cosd(60)"       # 0.5     (cosine, argument in degrees)
excalc "asind(0.5)"     # 30      (arcsine, result in degrees)
```

**Statistics** ([details](docs/functions/statistics.md)), variadic unless noted:

```
excalc "sum(1, 2, 3, 4)"                  # 10      (sum)
excalc "average(2, 4, 6)"                 # 4       (average)
excalc "product(1, 2, 3, 4)"              # 24      (product)
excalc "min(3, 1, 2)"                     # 1       (minimum)
excalc "max(3, 1, 2)"                     # 3       (maximum)
excalc "median(3, 1, 2)"                  # 2       (median)
excalc "mode(1, 2, 2, 3)"                 # 2       (mode)
excalc "variance(2, 4, 4, 4, 5, 5, 7, 9)" # 4  (population variance)
excalc "stddev(2, 4, 4, 4, 5, 5, 7, 9)"   # 2  (population standard deviation)
excalc "percentile(50, 1, 2, 3)"          # 2  (50th percentile)
excalc "covariance(1, 2, 2, 4, 3, 6)"     # ~1.3333 (population covariance of (x,y) pairs)
excalc "correlation(1, 2, 2, 4, 3, 6)"    # 1  (Pearson correlation of (x,y) pairs)
excalc "skewness(1, 2, 2, 3, 10)"         # ~1.361 (population skewness)
excalc "kurtosis(1, 2, 3, 4, 5)"          # -1.3   (population excess kurtosis)
excalc "harmonic(4)"                      # ~2.0833 (harmonic mean)
excalc "binom(5, 2)"                      # 10      (5 choose 2)
```

**Probability distributions** ([details](docs/functions/probability.md)):

```
excalc "binomialpdf(10, 0.5, 5)" # ~0.246 (P(X = 5) for Binomial(10, 0.5))
excalc "binomialcdf(10, 0.5, 5)" # ~0.623 (P(X <= 5) for Binomial(10, 0.5))
excalc "poissonpdf(4, 2)"        # ~0.1465 (P(X = 2) for Poisson(4))
excalc "poissoncdf(4, 2)"        # ~0.2381 (P(X <= 2) for Poisson(4))
excalc "normalpdf(0, 0, 1)"      # ~0.3989 (density of the standard normal at 0)
excalc "normalcdf(1.96, 0, 1)"   # ~0.975  (P(Z <= 1.96), standard normal)
excalc "zscore(85, 70, 10)"      # 1.5     (standard score)
```

**Combinatorics** ([details](docs/functions/combinatorics.md)):

```
excalc "factorial(5)"             # 120   (5!, same as postfix 5!)
excalc "perm(5, 2)"               # 20    (permutations of 2 out of 5)
excalc "catalan(3)"               # 5     (3rd Catalan number)
excalc "multinomial(10, 2, 3, 5)" # 2520  (10! / (2! 3! 5!))
excalc "stirling1(4, 2)"          # 11    (unsigned Stirling number, 1st kind)
excalc "stirling2(4, 2)"          # 7     (Stirling number, 2nd kind)
excalc "derangement(5)"           # 44    (derangements of 5 elements)
excalc "bell(5)"                  # 52    (5th Bell number)
```

**General / rounding** ([details](docs/functions/general.md)):

```
excalc "abs(-2)"             # 2     (absolute value)
excalc "frac(1.345)"         # 0.345 (fractional part)
excalc "intg(2.1)"           # 2     (alias of trunc)
excalc "round(-2.5)"         # -2    (ties round towards +infinity)
excalc "trunc(2.1)"          # 2     (truncate towards zero)
excalc "ceil(-2.1)"          # -2    (round up)
excalc "floor(-2.1)"         # -3    (round down)
excalc "sign(-5)"            # -1    (sign)
excalc "clamp(15, 0, 10)"    # 10 (restrict to a range)
excalc "lerp(0, 10, 0.5)"    # 5  (linear interpolation)
excalc "roundto(3.14159, 2)" # 3.14  (round to 2 decimal places)
excalc "floordiv(-7, 2)"     # -4    (integer division towards -infinity)
excalc "mod2(-7, 3)"         # 2     (floored modulo; remainder takes sign of divisor)
```

`random(x)` returns a uniformly distributed random number in `[0, x]`, e.g. `excalc "random(8)"` — omitted above since its result isn't deterministic.

**Number theory** ([details](docs/functions/number-theory.md)):

```
excalc "gcd(3213, 24)"      # 3     (greatest common divisor)
excalc "lcm(14, 4)"         # 28    (least common multiple)
excalc "fib(10)"            # 55    (10th Fibonacci number)
excalc "lucas(10)"          # 123   (10th Lucas number)
excalc "prime?(86)"         # 83    (closest prime <= 86)
excalc "moebius(2)"         # -1    (Mobius function)
excalc "mersenne(7)"        # 127   (2^7 - 1)
excalc "perfect(1231)"      # 496   (closest known perfect number <= 1231)
excalc "fermat(4)"          # 65537 (2^(2^4) + 1)
excalc "safeprime(12)"      # 23    (smallest safe prime >= 12)
excalc "primec(86)"         # 23    (count of primes <= 86)
excalc "primen(86)"         # 443   (the 86th prime)
excalc "mersennegen(12)"    # 7     (closest Mersenne generator <= 12)
excalc "mersgen(7)"         # 127   (Mersenne number for generator 7)
excalc "genmers(127)"       # 7     (generator for Mersenne number 127)
excalc "sigma(100, 0)"      # 9     (number of divisors of 100)
excalc "tau(9)"             # 13    (sum of divisors of 9)
excalc "phi(12)"            # 4     (Euler's totient of 12)
excalc "primorial(10)"      # 210   (product of primes <= 10)
excalc "digitsum(12345)"    # 15    (sum of decimal digits)
excalc "digitalroot(12345)" # 6  (repeated digit sum)
excalc "palindrome?(12321)" # 1  (true; reads the same forwards and backwards)
excalc "nextprime(10)"      # 11    (smallest prime > 10)
excalc "triangular(10)"     # 55    (10th triangular number)
excalc "pentagonal(10)"     # 145   (10th pentagonal number)
excalc "hexagonal(10)"      # 190   (10th hexagonal number)
excalc "carmichael(561)"    # 80    (Carmichael function of 561)
excalc "aliquot(220)"       # 284   (sum of proper divisors of 220)
excalc "amicable?(220)"     # 1     (true; 220 and 284 are an amicable pair)
excalc "coprime?(14, 15)"   # 1     (true; gcd(14, 15) = 1)
excalc "order(2, 5)"        # 4     (multiplicative order of 2 mod 5)
excalc "jacobi(1001, 9907)" # -1 (Jacobi symbol)
```

**Base conversion** ([details](docs/functions/base-conversion.md)):

```
excalc "hex(255)"   # "0xff"    (format as hexadecimal)
excalc "oct(8)"     # "0o10"    (format as octal)
excalc "bin(10)"    # "0b1010"  (format as binary)
excalc "0xff"       # 255     (hexadecimal literal)
excalc "0o17"       # 15      (octal literal)
excalc "0b1010"     # 10      (binary literal)
excalc "0xff + 1"   # 256     (radix literals are just numbers, usable anywhere)
excalc "0XA + 0b10" # 12      (uppercase prefixes and mixed bases both work)
```

`hex`/`oct`/`bin` accept a single non-negative integer and return text, not a number — the result can't be used inside a larger expression (`1 + hex(255)` is an error), and only the last statement of an `excalc` invocation may be text. `0x`/`0o`/`0b`-prefixed literals go the other way, parsing a hex/octal/binary number as an ordinary numeric value usable anywhere.

**Comparison and logical operators** ([details](docs/functions/logic.md)):

```
excalc "2 = 3"            # 0  (equality test)
excalc "3 > 2"            # 1  (greater than)
excalc "3 < 2"            # 0  (less than)
excalc "2 xor 4"          # 6  (bitwise xor)
excalc "2 xnor 4"         # -7 (bitwise xnor)
excalc "3 and 9"          # 1  (bitwise and)
excalc "3 & 9"            # 1  (& is a synonym for and)
excalc "3 nand 9"         # -2 (bitwise nand)
excalc "2 or 4"           # 6  (bitwise or)
excalc "2 nor 4"          # -7 (bitwise nor)
excalc "not(1)"           # -2 (bitwise not)
excalc "shl(2, 1)"        # 4  (2 * 2^1)
excalc "shr(2, 1)"        # 1  (2 / 2^1)
excalc "popcount(255)"    # 8  (number of set bits)
excalc "bitlen(256)"      # 9  (bits needed to represent 256)
excalc "bitreverse(1, 4)" # 8  (reverse the lowest 4 bits)
```

**Constants**: `pi` and `e` (case-insensitive).

**Variables**:

```
excalc "x := 5; x * 2"  # 10 (assign then use in a later statement)
excalc "x := 5
y := x^2 + 1
y"                    # 26 (statements can also be newline-separated)
excalc "x := 41 + 1"    # 42 (assignment's value is the assigned value)
excalc "pi := 5"        # error: cannot assign to reserved constant: pi
```

`name := expr` assigns to a variable, visible to later statements in the same input (separated by `;` or a newline); the value of the last statement is the result. Assignment is a statement, not an expression — it can't be nested inside a larger expression or chained (`x := y := 5`). Variables don't persist across separate `excalc` invocations or MCP tool calls; `pi`/`e` are reserved and can't be reassigned.

**User-defined functions**:

```
excalc "f(x) := x^2 + 1; f(3)"          # 10 (define then call in a later statement)
excalc "double(x) := x * 2
quad(x) := double(double(x))
quad(3)"                              # 12 (functions can call other functions)
excalc "fact(n) := n < 2 ? 1 : n * fact(n - 1); fact(10)"  # 3628800 (real terminating recursion)
excalc "f(x) := f(x); f(1)"             # error: function call recursion limit exceeded: f
excalc "sqrt(x) := x"                   # error: cannot redefine built-in function: sqrt
```

`name(params) := expr` defines a function, visible to later statements the same way a variable assignment is; calling it evaluates `expr` with each parameter bound to the corresponding argument (evaluated in the *caller's* scope, so a parameter can't accidentally see itself). Functions can call themselves or each other; combined with the `cond ? then : else` conditional below, a self-recursive call can stop at a computed base case instead of always recursing. There's no fixed call-count limit; instead, each nested call checks actual remaining stack space and errors gracefully (`function call recursion limit exceeded`) once it's running low, rather than crashing with a native stack overflow. Function names can't collide with built-in functions or `pi`/`e`, and definitions aren't saved to disk — like variables, they don't persist across separate `excalc` invocations or MCP tool calls.

**Conditionals** ([details](docs/functions/logic.md)):

```
excalc "3 > 2 ? 10 : 20"   # 10
excalc "if(3 > 2, 10, 20)" # 10 (exact equivalent function form)
excalc "1 ? 2 : 0 ? 3 : 4" # 2 (right-associative: a ? b : (c ? d : e))
```

`cond ? then : else` (and its exact equivalent, `if(cond, then, else)`) evaluates `cond`, then evaluates and returns *only* the taken branch — `then` if `cond` is nonzero, `else` otherwise — so the untaken branch is never evaluated, matching every other language's short-circuiting ternary/`if`. This is what lets self-recursive functions actually terminate (see `fact` above) instead of always recursing to the stack limit.

**Advanced / special functions** ([details](docs/functions/advanced.md)):

```
excalc "gamma(0.5)"       # ~1.7724407046383086  (Euler's gamma function; sqrt(pi))
excalc "beta(1, 2)"       # 0.5                  (Euler's beta function)
excalc "pochhammer(5, 3)" # ~210                 (rising factorial)
excalc "bth(2, 3, 4)"     # 625                  (x+y)^n via binomial expansion
excalc "bman(2, 3, 4)"    # 625                  (x+y)^n the "standard" way
excalc "ellipticE(1)"     # ~1                   (complete elliptic integral of the second kind)
excalc "ellipticF(0.01)"  # ~1.5708355989121519  (incomplete elliptic integral of the first kind)
excalc "ellipticCE(1)"    # ~1.5707963267948966  (complementary elliptic integral of the second kind; pi/2)
excalc "ellipticCK(1)"    # ~1.5707963267948966  (complementary elliptic integral of the first kind; pi/2)
excalc "dilog(1)"         # 0                    (dilogarithm)
excalc "dawson(0)"        # 0                    (Dawson function)
excalc "erf(1)"           # ~0.8427007929497227  (error function)
excalc "erfc(1)"          # ~0.15729920705027733 (complementary error function)
excalc "si(1)"            # ~0.9460830703671972  (sine integral)
excalc "ssi(1)"           # ~-0.6247132564276994 (shifted sine integral)
excalc "ci(1)"            # ~0.3374039229009618  (cosine integral)
excalc "chi(1)"           # ~0.8378669409802157  (hyperbolic cosine integral)
excalc "fresnelC(1)"      # ~0.7798934003768329  (Fresnel cosine integral)
excalc "fresnelS(1)"      # ~0.4382591473903456  (Fresnel sine integral)
```

**Numeric integration** ([details](docs/functions/numeric-integration.md)) — takes an expression and a bare variable, unlike every other function:

```
excalc "trapezoid(log(x^3), x, 1, 10, 64)"  # ~18.272118017450193 (trapezoidal rule)
excalc "simpson(log(x^3), x, 1, 10, 64)"    # ~18.274048637598288 (Simpson's rule)
excalc "newton(log(x^3), x, 1, 10, 64)"     # ~18.27404883248503  (Newton-Cotes 3/8 rule)
excalc "boole(log(x^3), x, 1, 10, 64)"      # ~18.274048988489618 (Boole's rule)
excalc "ordersix(log(x^3), x, 1, 10, 64)"   # ~18.274048988543118 (6th-order Newton-Cotes rule)
excalc "weddle(log(x^3), x, 1, 10, 64)"     # ~18.274048988612115 (Weddle's rule)
excalc "int(log(x^3), x, 1, 10, 0.0000001)" # ~18.274048987232057 (adaptive quadrature, alias gauss)
```

**Root finding** ([details](docs/functions/root-finding.md)) — also takes an expression and a bare variable:

```
excalc "bisect(x^2 - 2, x, 0, 2, 0.000001)" # ~1.4142141342163086 (bisection method; approximates sqrt(2))
excalc "secant(x^2 - 2, x, 0, 2, 0.000001)" # ~1.4142135623730947 (secant method; approximates sqrt(2))
```

**Geometry** ([details](docs/functions/geometry.md)) — `distance`/`manhattan`/`dot` work in any dimension, split their argument list in half between the two points/vectors:

```
excalc "distance(0, 0, 3, 4)"       # 5       (2D Euclidean distance)
excalc "distance(0, 0, 0, 1, 1, 1)" # ~1.732  (3D Euclidean distance)
excalc "manhattan(0, 0, 3, 4)"      # 7       (2D Manhattan/taxicab distance)
excalc "dot(1, 2, 3, 4)"            # 11      (2D dot product)
excalc "norm(3, 4)"                 # 5       (Euclidean magnitude of a vector)
excalc "triarea(3, 4, 5)"           # 6       (triangle area via Heron's formula)
excalc "circlearea(2)"              # ~12.566 (area of a circle)
excalc "circumference(2)"           # ~12.566 (circumference of a circle)
excalc "spherevol(3)"               # ~113.097 (volume of a sphere)
excalc "spherearea(3)"              # ~113.097 (surface area of a sphere)
```

**Unit conversion** ([details](docs/functions/units.md)):

```
excalc "c2f(100)" # 212     (Celsius to Fahrenheit)
excalc "f2c(212)" # 100     (Fahrenheit to Celsius)
excalc "km2mi(1)" # ~0.6214 (kilometers to miles)
excalc "mi2km(1)" # 1.609344 (miles to kilometers)
excalc "kg2lb(1)" # ~2.2046 (kilograms to pounds)
excalc "lb2kg(1)" # 0.45359237 (pounds to kilograms)
excalc "m2ft(1)"  # ~3.2808 (meters to feet)
excalc "ft2m(1)"  # 0.3048  (feet to meters)
```

**Financial functions** ([details](docs/functions/financial.md)):

```
excalc "pv(1000, 0.08, 5)"                         # ~3992.7100370780886  (present value)
excalc "fv(1000, 0.08, 5)"                         # ~5866.600960000006   (future value)
excalc "pmt(10000, 0.08, 5)"                       # ~2504.564545668364   (payment per period)
excalc "npv(0.1, 100, 200, 300)"                   # ~481.59278737791124  (net present value)
excalc "rate(2000, 1000, 10)"                      # ~0.07177346253629313 (interest rate per period)
excalc "cterm(0.1, 2000, 1000)"                    # ~7.272540897341713   (periods to compound to a future value)
excalc "term(100, 0.01, 5000)"                     # ~40.74890715609402   (periods to reach a future value via deposits)
excalc "sln(10000, 1000, 5)"                       # 1800                 (straight-line depreciation)
excalc "syd(10000, 1000, 5, 1)"                    # 3000                 (sum-of-the-years-digits depreciation)
excalc "ddb(10000, 1000, 5, 1)"                    # 4000                 (double-declining-balance depreciation)
excalc "db(50000, 10000, 5, 1, 3)"                 # ~3440.254204028806   (fixed-declining-balance depreciation)
excalc "irate(5, 2504.5645456684, -10000, 0, 0)"   # ~0.08                (interest rate, payment-timing aware)
excalc "nper(0.08, 2504.5645456684, -10000, 0, 0)" # ~5                   (number of periods, payment-timing aware)
excalc "paymt(0.08, 5, -10000, 0, 0)"              # ~2504.564545668364   (payment, payment-timing aware)
excalc "ipaymt(0.08, 1, 5, -10000, 0, 0)"          # 800                  (interest portion of a payment)
excalc "ppaymt(0.08, 1, 5, -10000, 0, 0)"          # ~1704.5645456683642  (principal portion of a payment)
```

**Errors** exit with status `1` and print a message, e.g.:

```
excalc "1/0"          # error: division by zero
excalc "sqrt(-1)"     # error: domain error in sqrt
excalc "unknownfn(1)" # error: unknown function: unknownfn
```

## MCP Server

`excalc-mcp` exposes the evaluator as an `evaluate` tool over stdio via [MCP](https://modelcontextprotocol.io/), for AI agents that support it instead of shelling out to the CLI. It's installed by default (see [Install](#install)).

If you only want the CLI and not the MCP server (skipping the `rmcp`/`tokio` dependencies it pulls in):

```bash
cargo install excalc --no-default-features
```

```bash
copilot mcp add excalc -- excalc-mcp  # GitHub Copilot CLI
claude mcp add excalc -- excalc-mcp   # Claude Code
```

For other clients (Claude Desktop, VS Code, etc.), add this to their MCP config file (e.g. `mcp.json` or `mcp-config.json`):

```json
{
  "mcpServers": {
    "excalc": {
      "command": "excalc-mcp"
    }
  }
}
```

Once connected, just ask your agent a math question in plain language (no special syntax or keyword needed) and it'll call the tool directly instead of shelling out or computing it itself, e.g.:

```
> what's sqrt(binom(10, 3) * average(2, 4, 6, 8)) + logn(81, 3) - sin(pi/6) * 2 ?
27.49489742783178
```

returns `27.49489742783178`.

The `evaluate` tool takes a single `expression` string argument and returns the numeric result as text, or a tool error with the same message the CLI would print (e.g. `division by zero`, `domain error in sqrt`).

## Development

Without installing, you can also run it straight from a checkout of this repo:

```
cargo run -- "2 + 2 * 3"
  # 8
```

If you already have another `calc` on your `PATH`, check `which calc` after installing — `cargo install` won't warn you if it shadows an existing command.

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to build, test, and contribute, [RELEASING.md](RELEASING.md) for how to cut a new release, and [CHANGELOG.md](CHANGELOG.md) for release history. See [port/DESIGN.md](port/DESIGN.md) for the grammar, function catalog, and what's ported vs. deferred vs. skipped. See [OTHERS.md](OTHERS.md) for how this compares to other command-line calculators (`bc`, `dc`, GNU `units`, Qalculate!, Numbat, Frink, and more).

## History

Spiritual successor to the Vestris Inc. shareware Expression Calculator written in 1996 in Pascal. It was also pressed on a CD-ROM, translated, and sold in Germany under the name Global Calculator in 1997. See [HISTORY.md](HISTORY.md).

![Expression Calculator NT](docs/images/small.jpg)

## License

MIT License, see [LICENSE](LICENSE) for details.
