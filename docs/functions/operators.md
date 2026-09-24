# Operators

See [../../DESIGN.md](../../DESIGN.md#grammar) for the full precedence table.

## Arithmetic

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `+` | Addition | `2 + 3` | `5` |
| `-` | Subtraction | `5 - 3` | `2` |
| `*` | Multiplication | `4 * 3` | `12` |
| `/` | Division | `10 / 4` | `2.5` |
| `mod` | Modulo (remainder) | `7 mod 3` | `1` |

Division and modulo by zero return a `division by zero` error rather than `NaN`/`inf`.

## Power and root

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `^` | Power (right-associative) | `2 ^ 10` | `1024` |
| `\` | N-th root (left-associative) | `2 \ 9` | `3` |

`n \ x` means "the n-th root of x", i.e. `x ^ (1/n)`. For example `3 \ 27` is the cube root of 27, which is `3`.

## Postfix

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `!` | Factorial (non-negative integers only) | `5!` | `120` |
| `%` | Percent (divides by 100) | `50%` | `0.5` |

## Unary

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `-` | Negation | `-5 + 3` | `-2` |

Because the CLI uses [`clap`](https://docs.rs/clap) for argument parsing, an expression that starts with `-` looks like a flag to the shell. Use `--` to separate it, or lead with `0 -`:

```bash
excalc -- "-5 + 3"
excalc "0 - 5 + 3"
```

## Grouping

Parentheses `( )` override precedence as usual: `(2 + 3) * 4` is `20`.

## Constants

| Name | Value |
|------|-------|
| `pi` | π (3.14159...) |
| `e`  | Euler's number (2.71828...) |

Constants are case-insensitive and cannot currently be reassigned.
