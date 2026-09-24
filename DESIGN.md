# Design

A Rust reimplementation of the core engine from
[excalc](https://github.com/dblock/excalc) (Vestris Inc. Expression
Calculator, Pascal/Delphi, mid-90s), rebuilt as a portable calculator tool
for AI coding agents: outsource math to a small, deterministic binary
instead of having an LLM compute it (and burn tokens getting it wrong).

This is a from-scratch Rust design "in spirit" of the original, not a
line-by-line port. Class/function names, error messages, and precedence
have been reworked where the 90s original was UI-driven, quirky, or simply
of its time.

## Scope

Ported from `common/MCalc.pas` in the original repo, grouped by the
milestone that introduces them:

- **v1 (this pass):** core arithmetic engine, standard math functions,
  statistics functions.
- **Later passes:** number theory, advanced/special functions, financial
  functions, comparison/logic operators.
- **Not ported:** anything GUI-only (2D/3D plotting, drawing, Windows
  registry-based user-function storage, the Delphi `TCalcThread` threading
  model). None of that applies to a headless CLI/MCP tool.

## Grammar

Conventional precedence, loosest to tightest binding:

1. Additive: `+  -`
2. Multiplicative: `*  /  mod`
3. Unary prefix: `-x`
4. Power (`^`, right-assoc) / root (`\`, left-assoc, same tier)
5. Postfix: `!` (factorial), `%` (percent)
6. Primary: numbers, variables/constants, `name(args, ...)`, `(expr)`

**This differs from the original**, which bound `%` tighter than `* /` and
bound root looser than power, in a chain shaped by 90s calculator-button UI
design (`Term → Percentage → AnyRoot → Power → TenPower → Factor →
Operator`). v1 normalizes to precedence that matches what someone typing an
expression today would expect. See `git log` / conversation history for the
exact original chain if we ever need to reference it.

`n \ x` means "the n-th root of x", e.g. `2 \ 9` = 3, `3 \ 27` = 3.

## Numeric model

v1 uses `f64` throughout. Arbitrary-precision support (`math`/big-decimal
style, for exact integer/rational results) is planned but not yet
implemented — tracked as a follow-up, not blocking v1.

## Angle units

v1 trig functions operate in **radians only**. The original supported a
degree/radian mode toggle (`CalcMode`) that converted before/after every
trig call. Deferred to a follow-up (e.g. `sin_deg`, or a context-level angle
unit setting) since most programmatic callers expect radians by default.

## Known quirks / deviations from the original noted during the port

- Original had a dead duplicate branch (`if ct('fv') ... else if ct('fv')
  ...`) in the function arg-count table — a copy-paste artifact, dropped.
- Original used both full operator words (`and`, `or`, `xor`, ...) and
  single-letter shortcuts (`a`, `o`, `x`, ...) for the same logic operators,
  apparently keyboard shortcuts from the original UI. v1 will use full
  words only when logic operators are ported.
- `mod` is spelled out as a word token in v1, rather than a single
  character, since we're normalizing surface syntax anyway.

## Function catalog

### v1: Standard math (radians)

`sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh sec csc cot
asec acsc acot sech csch coth asech acsch acoth sqrt ln log logn`

### v1: Statistics (variadic unless noted)

`sum average(avg) product(prod) min max harmonic(n) binom(n, k)`

### Planned: Number theory

`gcd lcm fib(onacci) isprime factor mersenne perfect moebius fermat`

### Planned: Advanced / special functions

`gamma beta elliptice ellipticf pochhammer` and numeric integration
(`trapezoid`, `simpson`, `newton`, `boole`, `ordersix`, `weddle`, `gauss`)

### Planned: Financial

`pv fv pmt npv nper rate term cterm sln syd ddb irate paymt fval ipaymt
ppaymt pval`

### Planned: Operators

Comparison (`= < > !=`) and bitwise/logic (`and or xor nor xnor nand`),
deferred from v1 since they're peripheral to "outsource arithmetic".

## Interfaces

- **Library** (`excalc::evaluate`) — the core.
- **CLI** (`excalc "2 + 2 * 3"`) — done in v1.
- **MCP server** — planned; will expose an `evaluate` tool over stdio so
  Claude/Copilot/etc. can call it uniformly instead of shelling out.
