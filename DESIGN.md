# Design

A Rust reimplementation of the core engine from [excalc](https://github.com/dblock/excalc) (Vestris Inc. Expression Calculator, Pascal/Delphi, mid-90s), rebuilt as a portable calculator tool for AI coding agents: outsource math to a small, deterministic binary instead of having an LLM compute it (and burn tokens getting it wrong).

This is a from-scratch Rust design "in spirit" of the original, not a line-by-line port. Class/function names, error messages, and precedence have been reworked where the 90s original was UI-driven, quirky, or simply of its time.

See [docs/](docs/README.md) for detailed per-function reference documentation (domains, formulas, examples), split by category. See the [Function catalog](#function-catalog) below for what's implemented (v1) vs. planned per category.

## Scope

Ported from the original engine, grouped by the milestone that introduces them:

- **v1 (this pass):** core arithmetic engine, standard math functions, statistics functions, general/rounding functions, number theory functions, comparison/logical operators, advanced/special functions (gamma, beta, Pochhammer, elliptic integrals, named integral functions like `erf`/`dilog`/Fresnel integrals), financial functions (time value of money, depreciation, and their extended payment-timing-aware variants), general-purpose numeric integration (named composite quadrature rules plus adaptive `int`/`gauss`) — this closes out every function category from the original.
- **Not ported:** anything GUI-only (2D/3D plotting, drawing, Windows registry-based user-function storage, the Delphi `TCalcThread` threading model). None of that applies to a headless CLI/MCP tool.

## Grammar

Conventional precedence, loosest to tightest binding:

1. Comparison: `=  >  <`
2. Logical or-family: `or  nor  xor  xnor`
3. Logical and-family: `and` (`&` synonym) `  nand`
4. Additive: `+  -`
5. Multiplicative: `*  /  mod`
6. Unary prefix: `-x`
7. Power (`^`, right-assoc) / root (`\`, left-assoc, same tier)
8. Postfix: `!` (factorial), `%` (percent)
9. Primary: numbers, variables/constants, `name(args, ...)`, `(expr)`

**This differs from the original**, which bound `%` tighter than `* /` and bound root looser than power, in a chain shaped by 90s calculator-button UI design (`Term → Percentage → AnyRoot → Power → TenPower → Factor → Operator`). v1 normalizes to precedence that matches what someone typing an expression today would expect. See `git log` / conversation history for the exact original chain if we ever need to reference it.

`n \ x` means "the n-th root of x", e.g. `2 \ 9` = 3, `3 \ 27` = 3.

Above expressions sits one more layer: a full input is a `;`/newline-separated sequence of **statements**, each either `name := expr` (variable assignment) or a plain expression; the value of the last statement is the result — see [Variables](README.md#variables). Assignment is deliberately a statement, not an expression, so it can't be chained (`x := y := 5`) or nested inside a larger expression; this keeps `:=` unambiguous and out of the operator-precedence table above.

## Numeric model

v1 uses `f64` throughout. Arbitrary-precision support (`math`/big-decimal style, for exact integer/rational results) is not blocking v1 — tracked in [#1](https://github.com/dblock/excalc-rs/issues/1).

## Angle units

The core circular trig functions operate in **radians only**. The original supported a global degree/radian mode toggle (`CalcMode`) that converted before/after every trig call; v1 deliberately doesn't reintroduce that kind of implicit, session-wide state (it would silently change trig behavior for callers who don't expect it). Instead degree support is stateless and explicit: [`deg`](docs/functions/standard-math.md#degreeradian-conversion)/`rad` convert between units, and `sind`/`cosd`/`tand`/`asind`/`acosd`/`atand` are degree-native convenience wrappers around the six basic circular functions.

## Known quirks / deviations from the original noted during the port

- Original had a dead duplicate branch (`if ct('fv') ... else if ct('fv') ...`) in the function arg-count table — a copy-paste artifact, dropped.
- Original used both full operator words (`and`, `or`, `xor`, ...) and single-letter shortcuts (`a`, `o`, `x`, ...) for the same logic operators, apparently keyboard shortcuts from the original UI. v1 uses full words only, with one exception: `&` is kept as a shorthand for `and` since it's a common, unambiguous convention in modern calculators and programming languages.
- Original's `=` assigned a value to a variable (not equality), and `?` tested equality. v1 gives `=` the more intuitive equality meaning and drops `?` entirely; variable assignment instead uses a dedicated `:=` statement (see [Variables](docs/README.md#variables)), kept out of expression grammar so it's never ambiguous with equality.
- `mod` is spelled out as a word token in v1, rather than a single character, since we're normalizing surface syntax anyway.
- Original's `Ci`/`Chi` (cosine/hyperbolic-cosine integral) formulas reference an undefined variable `G` (presumably meant to be the Euler-Mascheroni constant), which the generic variable-lookup mechanism silently defaults to `0` — almost certainly a bug, since the results are meaningless without the real constant. v1 hardcodes the actual Euler-Mascheroni constant instead.
- Advanced/special functions that depend on a numeric integral in the original (`ellipticE`, `ellipticF`, `dilog`, `erf`, `erfc`, `si`, `ssi`, `ci`, `chi`, `dawson`, `fresnelC`/`fresnelS`) are routed through the same shared adaptive-quadrature engine as the general-purpose numeric integration functions below (`gamma` keeps its own fixed-step Riemann sum instead, to match its pre-existing worked example); see [docs/functions/advanced.md](docs/functions/advanced.md) for details.
- The original's actual dispatched financial functions use Delphi's standard `Math.Power`, which errors on a negative base with a non-integer exponent - unlike a separate (and unused by the dispatch table) `Power` helper, which silently returns `0` for any non-positive base. v1 uses `f64::powf`, and maps any resulting `NaN` to a domain error and any resulting infinity to an overflow error, rather than silently returning `0` or panicking.
- The original manual/doc draft named the fixed-declining-balance depreciation function `fdb`, but the actual dispatched token is `db` — v1 uses `db` to match the real implementation.
- Numeric integration functions (`trapezoid`, `simpson`, `newton`, `boole`, `ordersix`, `weddle`, `int`, `gauss`) are the only functions in v1 whose first two arguments are an unevaluated expression and a bare variable rather than plain numbers; the evaluator special-cases these by name before generic argument evaluation, substituting the variable in a scoped copy of the evaluation context for each sample point (see `eval.rs`'s `eval_composite_integration`/`eval_adaptive_integration`). The original's `int`/`gauss` use Hairer's 30-point Gauss-Kronrod quadrature with Aitken extrapolation; v1 ports these "in spirit" with an adaptive composite Simpson's rule instead of reproducing that specific algorithm's hardcoded coefficient tables — see [docs/functions/numeric-integration.md](docs/functions/numeric-integration.md).
- Original's named composite rules (`fSum`) step a `while` loop with a floating-point `<=` comparison, which can silently run one sub-interval short or long due to floating-point accumulation. v1 instead requires the sub-interval count `n` to be a positive whole number and loops exactly `n` times.

## Function catalog

Full reference (domains, formulas, examples) lives in [docs/](docs/README.md); this is just the index of names per category.

### v1: Operators ([details](docs/functions/operators.md))

`+ - * / mod ^ \ ! %`, constants `pi e`

### v1: Standard math (radians, plus explicit degree conversion) ([details](docs/functions/standard-math.md))

`sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh sec csc cot asec acsc acot sech csch coth asech acsch acoth sqrt ln log logn deg rad sind cosd tand asind acosd atand`

### v1: Statistics (variadic unless noted) ([details](docs/functions/statistics.md))

`sum average(avg) product(prod) min max harmonic(n) binom(n, k)`

### v1: General / rounding ([details](docs/functions/general.md))

`abs frac intg round trunc ceil floor random`

### v1: Number theory ([details](docs/functions/number-theory.md))

`gcd lcm fib(onacci) prime? moebius mersenne perfect fermat safeprime primec primen mersennegen mersgen genmers sigma tau phi(eind)`

### v1: Comparison and logical operators ([details](docs/functions/logic.md))

`= > <`, `xor xnor and nand or nor not shl shr`, `&` synonym for `and`

### v1: Advanced / special functions ([details](docs/functions/advanced.md))

`gamma beta pochhammer bth bman ellipticE ellipticF(ellipticK) ellipticCE ellipticCK dilog dawson erf erfc si ssi ci chi fresnelC fresnelS fresnelF fresnelG`

### v1: Numeric integration ([details](docs/functions/numeric-integration.md))

`trapezoid(trapez, trapezoide) simpson newton boole ordersix(ordresix) weddle` (named composite rules) and `int(gauss)` (adaptive quadrature)

### v1: Financial ([details](docs/functions/financial.md))

`pv fv pmt npv rate cterm term sln syd ddb db` and extended payment-timing-aware variants `irate nper paymt fval pval ipaymt ppaymt`

## Interfaces

- **Library** (`excalc::evaluate`) — the core.
- **CLI** (`excalc "2 + 2 * 3"`, also installed as `calc`) — done in v1.
- **MCP server** (`excalc-mcp`, installed by default via `cargo install excalc`; build with `cargo build`, or exclude via `--no-default-features`) — done. Exposes a single `evaluate` tool over stdio via [rmcp](https://crates.io/crates/rmcp) so Claude/Copilot/etc. can call it uniformly instead of shelling out. See the README's [MCP Server](README.md#mcp-server) section.
