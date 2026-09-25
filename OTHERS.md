# Other Command-Line Calculators

Research notes comparing `excalc-rs` (214 functions/operators as of this writing: standard math, trig, stats, probability distributions, financial functions, number theory, base conversion, combinatorics, numeric integration, root finding, geometry, unit conversion, and advanced/special functions) against the existing landscape of command-line calculators. This is a working document, not user-facing documentation.

## Objective comparison

Links in the table and notes point to the tool's project, official documentation, or a primary standards/reference source where available.

Facts about `excalc-rs` confirmed directly from the source (`Cargo.toml`, `src/main.rs`, `src/eval.rs`) rather than assumed:

- **Precision**: native `f64` throughout — no arbitrary-precision/bignum support.
- **Input modes**: one-shot CLI argument, piped stdin, or an interactive REPL (`excalc`/`calc` with no expression while stdin is a terminal) — the REPL has persistent line history across sessions, Tab completion of function names and argument-name hints inside a call's parentheses (via `rustyline`), and `help`/`about`/`vars`/`exit`/`quit` commands.
- **Expression style**: algebraic/infix only — no RPN mode.
- **Variables**: supports `name = expr` statement-style assignment within a single input (visible to later statements in the same invocation), plus **user-defined functions** (`name(params) := expr`, session-only like variables, can call themselves or each other; deep recursion is caught via a runtime remaining-stack-space check rather than a fixed call-count limit) and a **conditional** (`cond ? then : else` / `if(cond, then, else)`, short-circuiting so only the taken branch is evaluated) — no `while`/loop constructs, but a self-recursive function can stop at a computed base case via the conditional.
- **Units**: has unit *conversion* functions (`c2f`, `km2mi`, `kg2lb`, `m2ft`, etc.) but no unit *type system* — units aren't tracked through arbitrary expressions, and there's no dimensional-mismatch checking.
- **Currency/live data**: none — no live exchange rates, no historical currency conversion.
- **Symbolic computation**: none — purely numeric evaluation, no symbolic differentiation/integration/equation-solving over unknowns.

| Tool | Language | REPL / One-shot | Precision | Units as type | RPN | User-defined functions | Approx. scope | Notable feature |
|---|---|---|---|---|---|---|---|---|
| [**excalc-rs**](https://github.com/dblock/excalc-rs) | Rust | Both (one-shot/stdin/REPL) | `f64` | No (conversion functions only) | No | Yes (session-only, with `if`/ternary conditionals) | 214 named functions across math/stats/finance/number theory/combinatorics/geometry/units | Huge flat catalog of named functions covering many domains in one process |
| [`bc`](https://www.gnu.org/software/bc/) | C | Both | Arbitrary precision (`scale`) | No | No | Yes | Basic arithmetic + C-like scripting language | POSIX-standardized; `-l` math library |
| [`dc`](https://www.gnu.org/software/bc/manual/html_node/dc.html) | C (orig. B) | Both | Arbitrary precision | No | Yes | Yes (macros) | Minimal — stack ops, arithmetic, macros | Oldest Unix calculator; first program run on PDP-11 Unix |
| [GNU `units`](https://www.gnu.org/software/units/) | C | Both | Native | Yes (conversion db only) | No | No | Unit conversion only, no general math | ~3,000+ unit database, dimensional analysis |
| [Qalculate! / `qalc`](https://qalculate.github.io/) | C++ | Both | Extended precision | Yes (partial) | No | Yes | Extensive — math, units, currency, symbolic algebra, matrices | Symbolic calculus, live currency rates |
| [Insect](https://github.com/sharkdp/insect) (archived) | PureScript | REPL | 30 sig. digits | Yes (implicit conversion) | No | Yes | ~50 functions + full unit system | Superseded by Numbat |
| [Numbat](https://github.com/sharkdp/numbat) | Rust | REPL + scripting | Native (typed) | Yes (compile-time type) | No | Yes | Scientific stdlib + full unit type system | Units as static types; custom unit defs |
| [Frink](https://frinklang.org/) | Java | Both, full language | Arbitrary precision (exact/rational) | Yes (through arbitrary computation) | No | Yes | Extensive — huge unit/constant db, interval arithmetic | Automatic unit tracking through any computation |
| [`orpie`](https://github.com/pelzlpj/orpie) | OCaml | REPL (curses) | GSL-backed | Yes (basic) | Yes | Limited | Scientific — GSL function set | Full-screen visible RPN stack |
| [Emacs Calc](https://www.gnu.org/software/emacs/manual/html_node/emacs/Calculator.html) | Emacs Lisp | REPL (in-editor) | Arbitrary precision | Yes | Yes (default) + algebraic | Yes | Extensive — symbolic algebra, matrices, units | Both RPN and algebraic modes |
| [`wcalc`](https://github.com/aidenbell/wcalc) | C | Both | Adjustable | Yes (basic conversion) | No | Limited (active vars) | Moderate — trig/log/hyperbolic + units + bases | Natural-language-ish unit conversion |
| [`python3 -c`](https://docs.python.org/3/using/cmdline.html) / [`bpython`](https://bpython-interpreter.org/) | Python | Both | Native/arbitrary (via `decimal`) | No | No | Yes (full language) | General-purpose, no domain calculator | Baseline: full programming language |
| [`awk`](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/awk.html) | C | One-shot | Native | No | No | Yes (full language) | Baseline — libm functions only | Baseline: text tool repurposed as calculator |

## Individual tool notes

**[`bc`](https://www.gnu.org/software/bc/)** — First appeared in Version 6 Unix (1975), written by Lorinda Cherry, originally a C-like front end compiling to `dc` bytecode. Formally standardized in POSIX (1991). GNU `bc` (Philip A. Nelson, 1991) became a standalone bytecode interpreter with extensions beyond POSIX. Arbitrary precision controlled by the `scale` variable; `-l` loads a math library (`sqrt`, `sin`, `cos`, `e`, `l`/ln, `a`/atan). C-like control flow, user-defined functions, arrays — a genuine small programming language, usable interactively or piped (`echo "2+2" | bc`).

**[`dc`](https://www.gnu.org/software/bc/manual/html_node/dc.html)** — Written by Lorinda Cherry and Robert Morris at Bell Labs, early 1970s; reportedly the first user program run on the PDP-11 under Unix, originally written in B, predating C. Stack-based RPN: `4 5 * p` multiplies and prints. Arbitrary precision (`k` command sets decimal places), radix changes, registers, and macros — Turing-complete despite terse syntax. `bc` was originally a friendlier front end compiling down to `dc`.

**[GNU `units`](https://www.gnu.org/software/units/)** — Pure unit-conversion tool, not a general expression calculator. Curated database exceeding 3,000 units (scientific, historical, esoteric). Supports compound unit expressions (`units "3 tablespoons" "cups"`) and full dimensional analysis, rejecting nonsensical conversions. No general math-function library, no stats/finance/number-theory functionality.

**[Qalculate! / `qalc`](https://qalculate.github.io/)** — C++ desktop/CLI calculator combining a large math function library, symbolic algebra (equation solving, integrals, derivatives), units with automatic/explicit conversion, live currency exchange rates (`--exrates`), matrices/vectors/complex numbers, and a scripting-capable expression language with user-defined variables and functions. Runs one-shot (`qalc "expression"`) or interactively (`--interactive`, with history/tab-completion), plus `--terse` for scripting. Arguably the closest existing tool to `excalc-rs` in overall breadth, though it lacks a dedicated finance/number-theory/root-finding function catalog as named built-ins (much is done via general symbolic solving instead).

**[Insect](https://github.com/sharkdp/insect) (archived)** — Terminal/web scientific calculator written in PureScript (compiles to JS), ~50 math functions (trig, hyperbolic, inverse variants, `gamma`, log family, `sum`/`product`), 30-significant-digit precision, hex/octal/binary literal input, first-class physical units with implicit conversions and dimensional-mismatch errors, variable/function definitions, REPL conveniences (history, tab completion, Unicode identifiers like `λ`, `ν`). Archived March 2025; its author (sharkdp) retired it in favor of Numbat.

**[Numbat](https://github.com/sharkdp/numbat)** — Insect's from-scratch Rust rewrite: a statically-typed scientific calculator/language where physical units are compile-time types (adding meters to seconds is a type error, not a runtime one). Ships SI, US customary, imperial, astronomical, and atomic unit systems, lets users define custom units in one line (`unit bathtub = 150 L`), full REPL (history, Ctrl+R search, `help`/`list`/`info`, session saving) plus scriptable `.nbt` files. Focus is unit-safety/correctness rather than breadth of named special functions.

**[Frink](https://frinklang.org/)** — Java-based programming language/calculator (Alan Eliasen) whose defining feature is tracking, converting, and verifying units of measure through arbitrary chains of computation. Arbitrary-precision (rational/exact and complex) arithmetic, interval arithmetic for uncertainty propagation, a large built-in database of physical constants/units, date-time arithmetic, historical currency value conversion. Runs on any JVM (desktop, Android, web applet); functions both as an interactive calculator and a full general-purpose programming language.

**[Python (`python3 -c` / `bpython`)](https://docs.python.org/3/using/cmdline.html) — baseline** — Used informally via `python3 -c "print(2**10)"` or the `math`/`statistics`/`decimal`/`fractions` standard library. No unit system, no dedicated finance/number-theory function set — every capability must be assembled from library imports and general-purpose code. Included purely as a "what a general scripting language gives you for free" baseline.

**[`awk`](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/awk.html) — baseline** — Text-processing language occasionally repurposed as a calculator, e.g. `awk 'BEGIN{print 2+2}'`. Only a handful of libm-backed math functions (`sin`, `cos`, `atan2`, `exp`, `log`, `sqrt`, `int`) built in; no units, no stats, no bignum, no real REPL. Baseline comparison only.

**[`orpie`](https://github.com/pelzlpj/orpie)** — Curses-based, full-screen terminal RPN calculator written in OCaml (rewrite of an earlier C program, `rpc`), using `ocamlgsl` (OCaml bindings to GNU Scientific Library) for its function set and `ocaml-curses` for a Mutt/Vim-inspired keyboard-driven UI. Live on-screen stack, command-name autocompletion (`'` prefix), built-in physical constants/units, `~/.orpierc` config. No graphing, limited general programmability compared to `bc`/Frink.

**[Emacs Calc (`M-x calc`)](https://www.gnu.org/software/emacs/manual/html_node/emacs/Calculator.html)** — Full scientific/symbolic calculator embedded in Emacs (Emacs Lisp), defaulting to RPN stack-based input (`2 RET 3 RET +`) but switchable to algebraic infix mode (`m a`, or `'` for one-off algebraic entry). Arbitrary precision, units (`u s` to simplify/convert, extensive built-in unit table plus user-defined units), vectors/matrices, complex numbers, intervals, symbolic algebra/calculus — arguably the most feature-complete of the terminal-adjacent tools here, constrained mainly by requiring Emacs as the host environment.

**[`wcalc`](https://github.com/aidenbell/wcalc)** — C-language natural-expression command-line calculator: C-like syntax, trig/hyperbolic/log functions, scientific constants (π, e, speed of light), unit-aware expressions (`wcalc "10 inches to cm"`), multiple numeric bases (hex/octal/binary via flags), "active" variables, adjustable precision, command history, REPL and piped/one-shot modes. Sits between `bc` (C-like, precision-focused) and Qalculate! (units-aware) in scope, without Qalculate!'s symbolic algebra or currency support.

**[GNU vs. BSD `bc`/`dc` (macOS)](https://man.openbsd.org/bc)** — macOS ships BSD-derived (not GNU) `bc`/`dc`, tracking the POSIX-defined feature subset and traditionally lacking GNU-specific extensions (additional built-in math functions, certain flags/syntax conveniences). Scripts relying on GNU `bc` extensions may not behave identically on macOS/OpenBSD. In recent years OpenBSD and others have adopted independent modern C `bc`/`dc` implementations (e.g., Gavin Howard's `bc`, now used by several Linux distros as well as toybox/busybox), further fragmenting "which `bc` you get" across platforms — a real day-to-day portability friction point.

## Links and sources

Primary project pages and documentation used for the comparison:

- [`excalc-rs` repository](https://github.com/dblock/excalc-rs)
- [GNU `bc` project](https://www.gnu.org/software/bc/) and [GNU `bc` manual](https://www.gnu.org/software/bc/manual/)
- [POSIX `bc` specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/bc.html)
- [GNU `dc` manual](https://www.gnu.org/software/bc/manual/html_node/dc.html)
- [GNU Units project](https://www.gnu.org/software/units/) and [GNU Units manual](https://www.gnu.org/software/units/manual/)
- [Qalculate! project](https://qalculate.github.io/) and [libqalculate source](https://github.com/Qalculate/libqalculate)
- [Insect source and README](https://github.com/sharkdp/insect)
- [Numbat source and README](https://github.com/sharkdp/numbat) and [Numbat documentation](https://numbat.dev/)
- [Frink language site](https://frinklang.org/)
- [`orpie` source](https://github.com/pelzlpj/orpie)
- [GNU Emacs Calc manual](https://www.gnu.org/software/emacs/manual/html_node/emacs/Calculator.html)
- [`wcalc` source](https://github.com/aidenbell/wcalc)
- [Python command-line documentation](https://docs.python.org/3/using/cmdline.html), [Python `decimal` module](https://docs.python.org/3/library/decimal.html), and [bpython](https://bpython-interpreter.org/)
- [POSIX `awk` specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/awk.html)
- [OpenBSD `bc` manual](https://man.openbsd.org/bc) for a BSD implementation reference

## Gaps and opportunities

**What `excalc-rs` does that most peers don't, in one process:**

- **Breadth in a single flat function catalog.** At 214 named functions/operators, `excalc-rs` combines domains normally split across separate tools: trigonometry and stats (found in `wcalc`, Qalculate!, Frink), probability distributions (rare outside Qalculate!'s statistics functions and R/Python libraries), **financial functions** (NPV/IRR/annuity-style — essentially absent from every tool surveyed), number theory (partially in `bc`/Qalculate!, but not a dedicated catalog), combinatorics (mostly absent as first-class named functions elsewhere), numeric integration and root-finding as built-in named operators (others can do some of this via general symbolic/scripted means, not as a curated named-function set), and geometry helpers — all callable directly by name with no scripting/library-import step.
- **A single flat, discoverable function namespace** inherited from a 1996 Pascal design intended for quick lookups (now usable one-shot, piped, or interactively via the REPL), rather than a general-purpose programming language (`bc`, Frink, Numbat) or a symbolic CAS (Qalculate!, Emacs Calc) that requires more general knowledge to invoke the same capability.

**What these tools do that `excalc-rs` currently lacks:**

- **Unit-aware dimensional analysis.** GNU `units`' 3,000+ unit database with compound-unit conversion, Numbat's compile-time unit-as-type system, Frink's automatic unit tracking through arbitrary computation chains, Insect's implicit-conversion error checking, and even `wcalc`/`orpie`/Emacs Calc's built-in unit conversion are all more rigorous than a bucket of standalone "unit conversion" functions — none of these treat units as a checked type that propagates through arbitrary expressions the way Numbat/Frink do.
- **Arbitrary-precision / bignum arithmetic.** `bc`, `dc`, and Frink offer unbounded-precision integer/decimal arithmetic; `excalc-rs` uses native `f64` throughout — a concrete, confirmed gap, since precision loss in special/statistical functions is a common failure mode for float-only calculators.
- **RPN input mode.** `dc`, Emacs Calc (default), and `orpie` all offer stack-based reverse-Polish input; `excalc-rs`, per its algebraic/infix design inherited from the 1996 Pascal original, has no RPN mode.
- **Interactive REPL amenities: live session state across restarts.** `excalc-rs` now has a REPL (line history persisted to disk across sessions via `rustyline`, Tab completion of function names and argument-name hints inside a call's parentheses, `help`/`about`/`vars`/`exit`/`quit` commands, persistent variables across lines within a session), but Qalculate!'s `--interactive` mode, Numbat's and Insect's readline-style REPLs (history *search* via Ctrl+R), and `orpie`'s always-visible curses stack still offer richer interactive ergonomics — `excalc-rs` doesn't (yet) restore variable/session state across restarts, only command history.
- **Programmability/scripting: control flow.** `bc`'s C-like `if`/`while`/functions/arrays, Qalculate!'s user-defined functions and equation solving, Numbat's and Frink's full variable/function definitions plus script files, and Emacs Calc's symbolic algebra all let users build reusable, named compositions of primitives. `excalc-rs` now supports user-defined functions (`name(params) := expr`, session-only, can call themselves/each other, guarded against runaway recursion by a runtime stack-space check) and a short-circuiting conditional (`cond ? then : else` / `if(cond, then, else)`), so a self-recursive function can stop at a computed base case — real terminating recursion like a factorial is now possible — though there's still no `while`/loop construct.
- **Currency/live exchange rates.** Qalculate!'s `--exrates` and Frink's currency-conversion-with-historical-value features pull live/historical data; `excalc-rs`'s financial functions compute NPV/IRR but can't do "150 USD in 1990 dollars to EUR today" without a similar data feed.
- **Symbolic computation.** Qalculate! and Emacs Calc support symbolic differentiation/integration and equation solving over unknowns — a capability class entirely outside `excalc-rs`'s numeric-only design.
