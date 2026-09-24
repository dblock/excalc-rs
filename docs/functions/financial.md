# Financial

**Status: v1 (implemented).** Ported from the `Sln`/`Syd`/`Cterm`/`Term`/`Pmt`/`Rate`/`Pv`/`Npv`/`Fv` and `fDB`/`DDB`/`IRATE`/`nper`/`PAYMT`/`FVAL`/`IPAYMT`/`PPAYMT`/`PVAL` functions defined directly in the original `MCalc.pas` (its separate `Finance.pas` unit's currency-scaling wrappers of the same formulas are unused by the dispatch table, so they weren't a porting source). Unlike the advanced/special functions category, none of these depend on numeric integration - they're all closed-form (or, for `irate`, a simple secant-method root find).

`rate` is a per-period interest rate expressed as a decimal (e.g. `0.08` for 8%), not a percentage.

## Time value of money

| Function | Meaning | Example |
|----------|---------|---------|
| `pv(payment, rate, term)` | Initial value (present value) of a loan or annuity that can be paid off by making `term` payments of `payment`, with interest on the unpaid amount accruing at `rate` per interval | `pv(1000, 0.08, 5)` → `3992.71` |
| `fv(payment, rate, term)` | Accumulated amount from making `term` payments of `payment`, with interest accruing on the accumulated amount at `rate` compounded per interval | `fv(1000, 0.08, 5)` → `5866.60` |
| `pmt(principal, rate, term)` | Payment amount per interval on a loan or annuity of initial value `principal`, spread over `term` intervals at `rate` per interval | `pmt(10000, 0.08, 5)` → `2504.56` |
| `npv(rate, cashflow1, cashflow2, ...)` | Net present value of a sequence of cash flows (at least one) discounted at `rate` | `npv(0.1, 100, 200, 300)` → `481.59` |
| `rate(futureValue, presentValue, term)` | Interest rate per interval such that `presentValue` compounded over `term` intervals accumulates into `futureValue` | `rate(2000, 1000, 10)` → `0.0718` |
| `cterm(rate, futureValue, presentValue)` | Number of compounding periods required for `presentValue` to accumulate into `futureValue` at `rate`, with no periodic deposits | `cterm(0.1, 2000, 1000)` → `7.27` |
| `term(payment, rate, futureValue)` | Number of compounding periods required to accumulate `futureValue` by making periodic deposits of `payment` at `rate` per period | `term(100, 0.01, 5000)` → `40.75` |

## Extended variants (support a payment-timing flag)

These mirror the functions above but add an extra `ptype` parameter (`0` = payments at end of period, `1` = payments at start of period). Per the original: `irate` extends `rate`, `nper` extends both `cterm` and `term`, `paymt` extends `pmt`, `fval` extends `fv`, `pval` extends `pv`, and `ppaymt`/`ipaymt` split a payment into its principal/interest portions:

| Function | Meaning | Example |
|----------|---------|---------|
| `irate(nper, pmt, pv, fv, ptype)` | Interest rate solved numerically (secant method) | `irate(5, 2504.56, -10000, 0, 0)` → `~0.08` |
| `nper(rate, pmt, pv, fv, ptype)` | Number of periods | `nper(0.08, 2504.56, -10000, 0, 0)` → `~5` |
| `paymt(rate, nper, pv, fv, ptype)` | Payment amount | `paymt(0.08, 5, -10000, 0, 0)` → `2504.56` |
| `fval(rate, nper, pmt, pv, ptype)` | Future value | `fval(0.08, 5, 2504.56, -10000, 0)` → `~0` |
| `pval(rate, nper, pmt, fv, ptype)` | Present value | `pval(0.08, 5, 2504.56, 0, 0)` → `~-10000` |
| `ipaymt(rate, per, nper, pv, fv, ptype)` | Interest portion of payment number `per` | `ipaymt(0.08, 1, 5, -10000, 0, 0)` → `800` |
| `ppaymt(rate, per, nper, pv, fv, ptype)` | Principal portion of payment number `per` | `ppaymt(0.08, 1, 5, -10000, 0, 0)` → `1704.56` |

`ipaymt`/`ppaymt` always sum to `paymt` for the same arguments.

## Depreciation

| Function | Meaning | Example |
|----------|---------|---------|
| `sln(cost, salvage, life)` | Straight-line depreciation per interval for an item of initial value `cost` that has a value of `salvage` after `life` intervals | `sln(10000, 1000, 5)` → `1800` |
| `syd(cost, salvage, life, period)` | Sum-of-the-years-digits depreciation amount for a given (positive, whole) `period`, on an item with initial `cost` and a final `salvage` value at the end of `life` intervals | `syd(10000, 1000, 5, 1)` → `3000` |
| `ddb(cost, salvage, life, period)` | Double-declining-balance depreciation of an asset for `period`, with initial `cost` and final `salvage` value at the end of `life` | `ddb(10000, 1000, 5, 1)` → `4000` |
| `db(cost, salvage, life, period, month)` | Fixed-declining-balance depreciation; `month` (partial first-year convention, whole number `1`-`12`) can be omitted and defaults to `12`. `life` and `period` must be whole numbers, with `1 <= period <= life + 1` | `db(50000, 10000, 5, 1, 3)` → `3440.25` |

**Note on naming:** the original manual/doc draft for this page called the last function `fdb`, but the original engine's actual token (see `MCalc.pas`'s function name table) is `db` - this port uses `db` to match the real implementation.
