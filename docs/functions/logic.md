# Comparison and Logical Operators

## Comparison

These return `1` for true and `0` for false, rather than a boolean type — there is no boolean type in the engine.

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `=` | Tests equal values | `2 = 3` | `0` |
| `>` | Tests if leftmost is bigger | `3 > 2` | `1` |
| `<` | Tests if leftmost is smaller | `3 < 2` | `0` |

Note: in the original, `=` assigns a variable rather than testing equality, and `?` is the equality test. The port repurposes `=` as equality; `?` is instead used for the ternary conditional operator described below, and variable assignment uses a dedicated `:=` statement — see [Variables](../../README.md#variables).

## Conditionals

| Function | Meaning | Example | Result |
|----------|---------|---------|--------|
| `if(cond, then, else)` | Evaluates `cond`; returns `then` if it's nonzero, `else` otherwise | `if(3 > 2, 10, 20)` | `10` |
| `cond ? then : else` | Ternary operator; exact sugar for `if(cond, then, else)` | `3 > 2 ? 10 : 20` | `10` |

Both forms are short-circuiting: only the taken branch is evaluated, so the untaken one may reference undefined variables, divide by zero, or recurse further without being evaluated. This is what lets [user-defined functions](../../README.md#user-defined-functions) recurse to an actual base case instead of always recursing to the stack limit, e.g. `fact(n) := n < 2 ? 1 : n * fact(n - 1)`.

The condition binds at comparison precedence, so `a < b ? x : y` needs no parens; a ternary nested *inside* a condition does, e.g. `(a ? b : c) < d`. The `then`/`else` branches admit nested ternaries right-associatively, so `a ? b : c ? d : e` reads as `a ? b : (c ? d : e)`.

Note: since identifiers may end in a bare `?` (e.g. `prime?`, a Scheme-style predicate naming convention), a ternary condition that's a bare variable name needs a space before `?` (`x ? a : b`, not `x?a:b`), or the lexer reads `x?` as one identifier.

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
| `popcount(n)` | Population count: number of `1` bits in `n` | `popcount(255)` | `8` |
| `bitlen(n)` | Number of bits needed to represent `n` | `bitlen(256)` | `9` |
| `bitreverse(n, width)` | Reverses the lowest `width` bits of `n` | `bitreverse(1, 4)` | `8` |

`popcount`/`bitlen`/`bitreverse` operate on non-negative integers only (unlike the other bitwise operators above, which truncate any operand to `i64`); `bitreverse`'s `width` must be in `[1, 64]` and `n` must fit within it.

The original supported single-letter shortcuts (`a`, `o`, `x`, ...) for these as keyboard shortcuts from the Windows UI; this port uses full words (`and`, `or`, `xor`, ...) instead, with one exception: `&` is kept as a shorthand for `and` since it's a common, unambiguous convention in modern calculators and programming languages.
