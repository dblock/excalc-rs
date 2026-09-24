# Design

A Rust reimplementation of the core engine from [excalc](https://github.com/dblock/excalc) (Vestris Inc. Expression Calculator, Pascal/Delphi, mid-90s), rebuilt as a portable calculator tool for AI coding agents: outsource math to a small, deterministic binary instead of having an LLM compute it (and burn tokens getting it wrong).

This is a from-scratch Rust design "in spirit" of the original, not a line-by-line port. Class/function names, error messages, and precedence have been reworked where the 90s original was UI-driven, quirky, or simply of its time.

See [docs/](docs/README.md) for detailed per-function reference documentation (domains, formulas, examples), split by category and marked v1 vs. planned.

## Scope

Ported from `common/MCalc.pas` in the original repo, grouped by the milestone that introduces them:

- **v1 (this pass):** core arithmetic engine, standard math functions, statistics functions, general/rounding functions, number theory functions, comparison/logical operators.
- **Later passes:** advanced/special functions (including named integral functions like `erf`/`dilog`/Fresnel integrals), financial functions.
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

## Numeric model

v1 uses `f64` throughout. Arbitrary-precision support (`math`/big-decimal style, for exact integer/rational results) is planned but not yet implemented — tracked as a follow-up, not blocking v1.

## Angle units

v1 trig functions operate in **radians only**. The original supported a degree/radian mode toggle (`CalcMode`) that converted before/after every trig call. Deferred to a follow-up (e.g. `sin_deg`, or a context-level angle unit setting) since most programmatic callers expect radians by default.

## Known quirks / deviations from the original noted during the port

- Original had a dead duplicate branch (`if ct('fv') ... else if ct('fv') ...`) in the function arg-count table — a copy-paste artifact, dropped.
- Original used both full operator words (`and`, `or`, `xor`, ...) and single-letter shortcuts (`a`, `o`, `x`, ...) for the same logic operators, apparently keyboard shortcuts from the original UI. v1 uses full words only, with one exception: `&` is kept as a shorthand for `and` since it's a common, unambiguous convention in modern calculators and programming languages.
- Original's `=` assigned a value to a variable (not equality), and `?` tested equality. Since v1 has no variable-assignment operator, `=` was repurposed as equality and `?` was dropped entirely.
- `mod` is spelled out as a word token in v1, rather than a single character, since we're normalizing surface syntax anyway.

## Function catalog

Full reference (domains, formulas, examples) lives in [docs/](docs/README.md); this is just the index of names per category.

### v1: Operators ([details](docs/functions/operators.md))

`+ - * / mod ^ \ ! %`, constants `pi e`

### v1: Standard math (radians) ([details](docs/functions/standard-math.md))

`sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh sec csc cot asec acsc acot sech csch coth asech acsch acoth sqrt ln log logn`

### v1: Statistics (variadic unless noted) ([details](docs/functions/statistics.md))

`sum average(avg) product(prod) min max harmonic(n) binom(n, k)`

### v1: General / rounding ([details](docs/functions/general.md))

`abs frac intg round trunc ceil floor random`

### v1: Number theory ([details](docs/functions/number-theory.md))

`gcd lcm fib(onacci) isprime moebius mersenne perfect fermat safeprime primec primen mersennegen mersgen genmers sigma tau phi(eind)`

### v1: Comparison and logical operators ([details](docs/functions/logic.md))

`= > <`, `xor xnor and nand or nor not shl shr`, `&` synonym for `and`

### Planned: Advanced / special functions ([details](docs/functions/advanced.md))

`gamma beta elliptice ellipticf pochhammer` and numeric integration (`trapezoid`, `simpson`, `newton`, `boole`, `ordersix`, `weddle`, `gauss`)

### Planned: Financial ([details](docs/functions/financial.md))

`pv fv pmt npv nper rate term cterm sln syd ddb irate paymt fval ipaymt ppaymt pval`

## Interfaces

- **Library** (`excalc::evaluate`) — the core.
- **CLI** (`excalc "2 + 2 * 3"`, also installed as `calc`) — done in v1.
- **MCP server** (`excalc-mcp`, installed by default via `cargo install excalc`; build with `cargo build`, or exclude via `--no-default-features`) — done. Exposes a single `evaluate` tool over stdio via [rmcp](https://crates.io/crates/rmcp) so Claude/Copilot/etc. can call it uniformly instead of shelling out. See the README's [MCP Server](README.md#mcp-server) section.
