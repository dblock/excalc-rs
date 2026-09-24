# Comparison and Logical Operators

**Status: planned, not yet implemented.** See [../../DESIGN.md](../../DESIGN.md#scope) (listed as "comparison/logic operators", deferred from v1) and [../../AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions. Sourced from the "II. Operators" and "III. Logical Operators" chapters of the original [manual](../../HISTORY.md).

## Comparison

These return `1` for true and `0` for false, rather than a boolean type — there is no boolean type in the original engine.

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `=` | Assign a value to a variable (not equality!) | `x = 5` | `5` |
| `?` | Tests equal values | `2 ? 3` | `0` |
| `>` | Tests if leftmost is bigger | `3 > 2` | `1` |
| `<` | Tests if leftmost is smaller | `3 < 2` | `0` |

Note: in the original, `=` assigns a variable rather than testing equality; `?` is the equality test. This is unusual and worth reconsidering for a Rust port rather than carrying it over verbatim (see [AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine): flag anything that looks quirky before reproducing it).

## Logical / bitwise

All operate on truncated integers (`Z`); non-integer operands are truncated before the bitwise operation is performed.

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `xor` | Bitwise xor | `2 xor 4` | `6` |
| `xnor` | Bitwise xnor (opposite of `xor`) | `2 xnor 4` | `-7` |
| `and` / `&` | Bitwise and | `3 and 9` | `1` |
| `nand` | Bitwise nand (opposite of `and`) | `3 nand 9` | `-2` |
| `or` | Bitwise or | `2 or 4` | `6` |
| `nor` | Bitwise nor (opposite of `or`) | `2 nor 4` | `6` |
| `not(x)` | Bitwise not (inverts each bit) | `not(1)` | `2` |
| `shl(x, y)` | Shift `x` left by `y` bits, i.e. `x * 2^y` | `shl(2, 1)` | `4` |
| `shr(x, y)` | Shift `x` right by `y` bits, i.e. `x / 2^y` | `shr(2, 1)` | `1` |

DESIGN.md notes that the original supported single-letter shortcuts (`a`, `o`, `x`, ...) for these as keyboard shortcuts from the Windows UI; a Rust port should use the full words only.
