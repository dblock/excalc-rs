# General / Rounding Functions

**Status: planned, not yet implemented.** See [../../AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions. Sourced from the "IV. General Functions" chapter of the original [manual](../../HISTORY.md) (`abs`, `frac`, `intg`, `round`, `trunc`, `ceil`, `floor`, `random` were part of that chapter but weren't previously tracked in `docs/`).

| Function | Meaning | Domain | Example | Result |
|----------|---------|--------|---------|--------|
| `abs(x)` | Absolute value | `R -> R+` | `abs(-2)` | `2` |
| `frac(x)` | Fractional part, `x - trunc(x)` | `R -> Z` | `frac(1.345)` | `0.345` |
| `intg(x)` | Integer portion of a value (towards zero) | `R -> Z` | `intg(2.1)` | `2` |
| `round(x)` | Closest integer, half rounds toward zero | `R -> Z` | `round(-2.5)` | `-2` |
| `trunc(x)` | Integer portion of a value (towards zero) | `R -> Z` | `trunc(2.1)` | `2` |
| `ceil(x)` | Ceiling, closest integer `>= x` | `R -> N` | `ceil(-2.1)` | `-2` |
| `floor(x)` | Floor, closest integer `<= x` | `R -> N` | `floor(-2.1)` | `-3` |
| `random(x)` | Random number in `[0, x]` | `R+ -> R+` | `random(8)` | non-deterministic |

Notes from the original manual:

- `round` examples: `round(-2.1) = -2`, `round(-2.5) = -2`, `round(-2.6) = -3` (rounds half away from `.5` toward the nearer integer, not always "round half up").
- `intg` and `trunc` are documented identically in the original (`Intg` and `Trunc` both return "the integer portion of a value"); they may be aliases of the same operation rather than two distinct behaviors — worth confirming against `common/MCalc.pas` before implementing both.
