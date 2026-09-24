# Comparison and Logical Operators

Sourced from the "II. Operators" and "III. Logical Operators" chapters of the original [manual](../../HISTORY.md), adapted per [AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) porting conventions (see notes below on what changed from the original).

## Comparison

These return `1` for true and `0` for false, rather than a boolean type — there is no boolean type in the engine.

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `=` | Tests equal values | `2 = 3` | `0` |
| `>` | Tests if leftmost is bigger | `3 > 2` | `1` |
| `<` | Tests if leftmost is smaller | `3 < 2` | `0` |

Note: in the original, `=` assigns a variable rather than testing equality, and `?` is the equality test. Since this port has no assignment operator, `=` was repurposed as equality and `?` was dropped entirely (see [AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine): flag anything that looks quirky before reproducing it).

## Logical / bitwise

All operands are truncated to a signed 64-bit integer (`i64`) before the bitwise operation is performed; results wrap using standard two's-complement semantics (matching Rust's native bitwise operators) rather than raising an overflow error.

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `xor` | Bitwise xor | `2 xor 4` | `6` |
| `xnor` | Bitwise xnor (opposite of `xor`) | `2 xnor 4` | `-7` |
| `and` / `&` | Bitwise and | `3 and 9` | `1` |
| `nand` | Bitwise nand (opposite of `and`) | `3 nand 9` | `-2` |
| `or` | Bitwise or | `2 or 4` | `6` |
| `nor` | Bitwise nor (opposite of `or`) | `2 nor 4` | `-7` |
| `not(x)` | Bitwise not (inverts each bit) | `not(1)` | `-2` |
| `shl(x, y)` | Shift `x` left by `y` bits, i.e. `x * 2^y` (wraps on overflow) | `shl(2, 1)` | `4` |
| `shr(x, y)` | Shift `x` right by `y` bits, i.e. `x / 2^y` | `shr(2, 1)` | `1` |

The original supported single-letter shortcuts (`a`, `o`, `x`, ...) for these as keyboard shortcuts from the Windows UI; this port uses full words (`and`, `or`, `xor`, ...) instead, with one exception: `&` is kept as a shorthand for `and` since it's a common, unambiguous convention in modern calculators and programming languages.
