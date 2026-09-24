Expression Calculator (Rust)
=============================

<img src="docs/images/small.jpg" alt="Expression Calculator NT" align="right" width="120">

[![CI](https://github.com/dblock/excalc-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/dblock/excalc-rs/actions/workflows/ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/dblock/excalc-rs/badge.svg?branch=master)](https://coveralls.io/github/dblock/excalc-rs?branch=master)

A portable expression calculator, built as a CLI tool and an MCP server so AI coding agents like Claude and GitHub Copilot can outsource arithmetic instead of computing it themselves — saving tokens and avoiding LLM math mistakes.

Spiritual successor to the Vestris Inc. shareware Expression Calculator that I wrote in 1996 in Pascal. It was also pressed on a CD-ROM, translated, and sold in Germany under the name Global Calculator in 1997. See [HISTORY.md](HISTORY.md).

## Install

```bash
cargo install excalc
```

This installs the `excalc`, `calc` (a shorter alias for `excalc`), and `excalc-mcp` (see [MCP Server](#mcp-server)) binaries to `~/.cargo/bin` (make sure it's on your `PATH`). Requires a [Rust toolchain](https://rustup.rs/).

## Usage

```
calc "2 + 2 * 3"
# 8

calc "sqrt(16) + sin(pi/2)"
# 5

calc "1/0"
# error, exit code 1
```

Run `calc --help` (or `excalc --help`) for full usage.

### Examples

See [docs/](docs/README.md) for full reference documentation, including domains and error conditions for each function. See [DESIGN.md](DESIGN.md#function-catalog) for what's implemented vs. planned.

**Arithmetic operators** ([details](docs/functions/operators.md)):

```
calc "2 + 2 * 3"          # 8         (addition and multiplication, standard precedence)
calc "10 - 4 / 2"         # 8         (subtraction and division)
calc "7 mod 3"            # 1         (modulo)
calc "2 ^ 10"             # 1024      (power)
calc "2 \ 9"              # 3         (n-th root: the 2nd root of 9)
calc "5!"                 # 120       (factorial)
calc "50%"                # 0.5       (percent)
calc "(2 + 3) * 4"        # 20        (grouping)
calc -- "-5 + 3"          # -2        (unary minus needs -- so clap doesn't treat it as a flag)
```

**Standard math** ([details](docs/functions/standard-math.md)), all angles in radians (see `deg`/`rad` below for degree conversion):

```
calc "sqrt(16)"           # 4         (square root)
calc "ln(e)"              # 1         (natural log)
calc "log(100)"           # 2         (base 10)
calc "logn(8, 2)"         # 3         (log base 2 of 8)
calc "sin(pi/2)"          # 1         (sine)
calc "cos(0)"             # 1         (cosine)
calc "tan(pi/4)"          # ~1        (tangent)
calc "asin(1)"            # ~1.5708   (arcsine; pi/2)
calc "acos(0)"            # ~1.5708   (arccosine; pi/2)
calc "atan(1)"            # ~0.7854   (arctangent; pi/4)
calc "sinh(1)"            # ~1.1752   (hyperbolic sine)
calc "cosh(0)"            # 1         (hyperbolic cosine)
calc "tanh(0)"            # 0         (hyperbolic tangent)
calc "sec(0)"             # 1         (secant)
calc "csc(pi/2)"          # 1         (cosecant)
calc "cot(pi/4)"          # ~1        (cotangent)
calc "deg(pi)"            # 180       (radians to degrees)
calc "rad(180)"           # ~3.1416   (degrees to radians; pi)
calc "sind(30)"           # 0.5       (sine, argument in degrees)
calc "cosd(60)"           # 0.5       (cosine, argument in degrees)
calc "asind(0.5)"         # 30        (arcsine, result in degrees)
```

**Statistics** ([details](docs/functions/statistics.md)), variadic unless noted:

```
calc "sum(1, 2, 3, 4)"       # 10       (sum)
calc "average(2, 4, 6)"      # 4        (average)
calc "product(1, 2, 3, 4)"   # 24       (product)
calc "min(3, 1, 2)"          # 1        (minimum)
calc "max(3, 1, 2)"          # 3        (maximum)
calc "harmonic(4)"           # ~2.0833  (harmonic mean)
calc "binom(5, 2)"           # 10       (5 choose 2)
```

**General / rounding** ([details](docs/functions/general.md)):

```
calc "abs(-2)"            # 2         (absolute value)
calc "frac(1.345)"        # 0.345     (fractional part)
calc "intg(2.1)"          # 2         (alias of trunc)
calc "round(-2.5)"        # -2        (ties round towards +infinity)
calc "trunc(2.1)"         # 2         (truncate towards zero)
calc "ceil(-2.1)"         # -2        (round up)
calc "floor(-2.1)"        # -3        (round down)
```

`random(x)` returns a uniformly distributed random number in `[0, x]`, e.g. `calc "random(8)"` — omitted above since its result isn't deterministic.

**Number theory** ([details](docs/functions/number-theory.md)):

```
calc "gcd(3213, 24)"       # 3        (greatest common divisor)
calc "lcm(14, 4)"          # 28       (least common multiple)
calc "fib(10)"             # 55       (10th Fibonacci number)
calc "prime?(86)"          # 83        (closest prime <= 86)
calc "moebius(2)"          # -1        (Mobius function)
calc "mersenne(7)"         # 127       (2^7 - 1)
calc "perfect(1231)"       # 496       (closest known perfect number <= 1231)
calc "fermat(4)"           # 65537     (2^(2^4) + 1)
calc "safeprime(12)"       # 23        (smallest safe prime >= 12)
calc "primec(86)"          # 23        (count of primes <= 86)
calc "primen(86)"          # 443       (the 86th prime)
calc "mersennegen(12)"     # 7         (closest Mersenne generator <= 12)
calc "mersgen(7)"          # 127       (Mersenne number for generator 7)
calc "genmers(127)"        # 7         (generator for Mersenne number 127)
calc "sigma(100, 0)"       # 9         (number of divisors of 100)
calc "tau(9)"              # 13        (sum of divisors of 9)
calc "phi(12)"             # 4         (Euler's totient of 12)
```

**Comparison and logical operators** ([details](docs/functions/logic.md)):

```
calc "2 = 3"               # 0        (equality test)
calc "3 > 2"               # 1        (greater than)
calc "3 < 2"               # 0        (less than)
calc "2 xor 4"             # 6        (bitwise xor)
calc "2 xnor 4"            # -7       (bitwise xnor)
calc "3 and 9"             # 1        (bitwise and)
calc "3 & 9"               # 1        (& is a synonym for and)
calc "3 nand 9"            # -2       (bitwise nand)
calc "2 or 4"              # 6        (bitwise or)
calc "2 nor 4"             # -7       (bitwise nor)
calc "not(1)"              # -2       (bitwise not)
calc "shl(2, 1)"           # 4        (2 * 2^1)
calc "shr(2, 1)"           # 1        (2 / 2^1)
```

**Constants**: `pi` and `e` (case-insensitive).

**Variables**:

```
calc "x := 5; x * 2"                # 10        (assign then use in a later statement)
calc "x := 5
y := x^2 + 1
y"                                   # 26        (statements can also be newline-separated)
calc "x := 41 + 1"                   # 42        (assignment's value is the assigned value)
calc "pi := 5"                       # error: cannot assign to reserved constant: pi
```

`name := expr` assigns to a variable, visible to later statements in the same input (separated by `;` or a newline); the value of the last statement is the result. Assignment is a statement, not an expression — it can't be nested inside a larger expression or chained (`x := y := 5`). Variables don't persist across separate `calc` invocations or MCP tool calls; `pi`/`e` are reserved and can't be reassigned.

**Advanced / special functions** ([details](docs/functions/advanced.md)):

```
calc "gamma(0.5)"          # ~1.7724407046383086 (Euler's gamma function; sqrt(pi))
calc "beta(1, 2)"          # 0.5      (Euler's beta function)
calc "pochhammer(5, 3)"    # ~210     (rising factorial)
calc "bth(2, 3, 4)"        # 625      (x+y)^n via binomial expansion
calc "bman(2, 3, 4)"       # 625      (x+y)^n the "standard" way
calc "ellipticE(1)"        # ~1       (complete elliptic integral of the second kind)
calc "ellipticF(0.01)"     # ~1.5708355989121519 (incomplete elliptic integral of the first kind)
calc "ellipticCE(1)"       # ~1.5707963267948966 (complementary elliptic integral of the second kind; pi/2)
calc "ellipticCK(1)"       # ~1.5707963267948966 (complementary elliptic integral of the first kind; pi/2)
calc "dilog(1)"            # 0        (dilogarithm)
calc "dawson(0)"           # 0        (Dawson function)
calc "erf(1)"              # ~0.8427007929497227 (error function)
calc "erfc(1)"             # ~0.15729920705027733 (complementary error function)
calc "si(1)"               # ~0.9460830703671972 (sine integral)
calc "ssi(1)"              # ~-0.6247132564276994 (shifted sine integral)
calc "ci(1)"               # ~0.3374039229009618 (cosine integral)
calc "chi(1)"              # ~0.8378669409802157 (hyperbolic cosine integral)
calc "fresnelC(1)"         # ~0.7798934003768329 (Fresnel cosine integral)
calc "fresnelS(1)"         # ~0.4382591473903456 (Fresnel sine integral)
```

**Numeric integration** ([details](docs/functions/numeric-integration.md)) — takes an expression and a bare variable, unlike every other function:

```
calc "trapezoid(log(x^3), x, 1, 10, 64)"     # ~18.272118017450193 (trapezoidal rule)
calc "simpson(log(x^3), x, 1, 10, 64)"       # ~18.274048637598288 (Simpson's rule)
calc "newton(log(x^3), x, 1, 10, 64)"        # ~18.27404883248503  (Newton-Cotes 3/8 rule)
calc "boole(log(x^3), x, 1, 10, 64)"         # ~18.274048988489618 (Boole's rule)
calc "ordersix(log(x^3), x, 1, 10, 64)"      # ~18.274048988543118 (6th-order Newton-Cotes rule)
calc "weddle(log(x^3), x, 1, 10, 64)"        # ~18.274048988612115 (Weddle's rule)
calc "int(log(x^3), x, 1, 10, 0.0000001)"    # ~18.274048987232057 (adaptive quadrature, alias gauss)
```

**Financial functions** ([details](docs/functions/financial.md)):

```
calc "pv(1000, 0.08, 5)"                     # ~3992.7100370780886 (present value)
calc "fv(1000, 0.08, 5)"                     # ~5866.600960000006 (future value)
calc "pmt(10000, 0.08, 5)"                   # ~2504.564545668364 (payment per period)
calc "npv(0.1, 100, 200, 300)"               # ~481.59278737791124 (net present value)
calc "rate(2000, 1000, 10)"                  # ~0.07177346253629313 (interest rate per period)
calc "cterm(0.1, 2000, 1000)"                # ~7.272540897341713 (periods to compound to a future value)
calc "term(100, 0.01, 5000)"                 # ~40.74890715609402 (periods to reach a future value via deposits)
calc "sln(10000, 1000, 5)"                   # 1800    (straight-line depreciation)
calc "syd(10000, 1000, 5, 1)"                # 3000    (sum-of-the-years-digits depreciation)
calc "ddb(10000, 1000, 5, 1)"                # 4000    (double-declining-balance depreciation)
calc "db(50000, 10000, 5, 1, 3)"             # ~3440.254204028806 (fixed-declining-balance depreciation)
calc "irate(5, 2504.5645456684, -10000, 0, 0)"   # ~0.08   (interest rate, payment-timing aware)
calc "nper(0.08, 2504.5645456684, -10000, 0, 0)" # ~5      (number of periods, payment-timing aware)
calc "paymt(0.08, 5, -10000, 0, 0)"          # ~2504.564545668364 (payment, payment-timing aware)
calc "ipaymt(0.08, 1, 5, -10000, 0, 0)"      # 800     (interest portion of a payment)
calc "ppaymt(0.08, 1, 5, -10000, 0, 0)"      # ~1704.5645456683642 (principal portion of a payment)
```

**Errors** exit with status `1` and print a message, e.g.:

```
calc "1/0"                # error: division by zero
calc "sqrt(-1)"           # error: domain error in sqrt
calc "unknownfn(1)"       # error: unknown function: unknownfn
```

## MCP Server

`excalc-mcp` exposes the evaluator as an `evaluate` tool over stdio via [MCP](https://modelcontextprotocol.io/), for AI agents that support it instead of shelling out to the CLI. It's installed by default (see [Install](#install)).

If you only want the CLI and not the MCP server (skipping the `rmcp`/`tokio` dependencies it pulls in):

```bash
cargo install excalc --no-default-features
```

```bash
copilot mcp add excalc -- excalc-mcp   # GitHub Copilot CLI
claude mcp add excalc -- excalc-mcp    # Claude Code
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

Without installing, you can also run it straight from a checkout of this repo:

```
cargo run -- "2 + 2 * 3"
# 8
```

If you already have another `calc` on your `PATH`, check `which calc` after installing — `cargo install` won't warn you if it shadows an existing command.

See [CHANGELOG.md](CHANGELOG.md) for release history, [RELEASING.md](RELEASING.md) for how to cut a new release, and [CONTRIBUTING.md](CONTRIBUTING.md) for how to build, test, and contribute. See [DESIGN.md](DESIGN.md) for the grammar, function catalog, and what's ported vs. deferred vs. skipped.

## License

MIT License, see [LICENSE](LICENSE) for details.
