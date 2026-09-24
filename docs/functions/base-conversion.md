# Base Conversion

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `hex(n)` | Format `n` as a `0x`-prefixed lowercase hexadecimal string | `N -> string` | `hex(255)` → `"0xff"` |
| `oct(n)` | Format `n` as a `0o`-prefixed octal string | `N -> string` | `oct(8)` → `"0o10"` |
| `bin(n)` | Format `n` as a `0b`-prefixed binary string | `N -> string` | `bin(10)` → `"0b1010"` |

`0x`/`0o`/`0b`-prefixed literals parse a hexadecimal/octal/binary number as an ordinary numeric value, usable anywhere a number is (`0xff + 1` → `256`):

| Literal | Meaning | Example |
|---------|---------|---------|
| `0x...` (or `0X...`) | Hexadecimal literal | `0xff` → `255` |
| `0o...` (or `0O...`) | Octal literal | `0o17` → `15` |
| `0b...` (or `0B...`) | Binary literal | `0b1010` → `10` |

## Notes and implementation details

- `hex`/`oct`/`bin` require a single non-negative whole-number argument (representable exactly as a `u64`); fractional or negative input is a domain error.
- The result of `hex`/`oct`/`bin` is prefixed (`0x`/`0o`/`0b`) so it can be pasted directly back in as a literal, e.g. `hex(255)` → `"0xff"`, and `0xff` evaluates back to `255`.
- Unlike every other function, `hex`/`oct`/`bin` return text, not a number. Their result can only appear as the entire, final statement of an expression — it can't be combined into further arithmetic (`1 + hex(255)` is an error) and can't be assigned to a variable (variables are always numbers).
- `0x`/`0o`/`0b` literals are the inverse operation: they're parsed directly by the lexer into an ordinary numeric value, so they behave exactly like any other number (`0xff * 2` → `510`) and can be freely mixed into arithmetic, assigned to variables, and passed to any function. Different bases can be mixed freely in the same expression (`0xff + 0b1010` → `265`), and the prefix letter is case-insensitive (`0Xff` and `0xFF` are equivalent).
- A bare `0` or a decimal number starting with `0` (e.g. `012`) is unaffected — only a `0` immediately followed by `x`/`X`, `o`/`O`, or `b`/`B` is treated as a radix prefix.
