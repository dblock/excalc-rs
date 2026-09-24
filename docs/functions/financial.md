# Financial

**Status: planned, not yet implemented.** See [../../AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions. Descriptions below are sourced from the "X. Financial Functions" chapter of the original [manual](../../HISTORY.md).

## Time value of money

| Function | Meaning |
|----------|---------|
| `pv(payment, rate, term)` | Initial value (present value) of a loan or annuity that can be paid off by making `term` payments of `payment`, with interest on the unpaid amount accruing at `rate` per interval |
| `fv(payment, rate, term)` | Accumulated amount from making `term` payments of `payment`, with interest accruing on the accumulated amount at `rate` compounded per interval |
| `pmt(principal, rate, term)` | Payment amount per interval on a loan or annuity of initial value `principal`, spread over `term` intervals at `rate` per interval |
| `npv(rate, cashflow1, cashflow2, ...)` | Net present value of a sequence of cash flows discounted at `rate` |
| `rate(futureValue, presentValue, term)` | Interest rate per interval such that `presentValue` compounded over `term` intervals accumulates into `futureValue` |
| `cterm(rate, futureValue, presentValue)` | Number of compounding periods required for `presentValue` to accumulate into `futureValue` at `rate`, with no periodic deposits |
| `term(payment, rate, futureValue)` | Number of compounding periods required to accumulate `futureValue` by making periodic deposits of `payment` at `rate` per period |

## Extended variants (support a payment-timing flag)

These mirror the functions above but add an extra `ptype` parameter (`0` = payments at end of period, `1` = payments at start of period). Per the manual: `irate` extends `rate`, `nper` extends both `cterm` and `term`, `paymt` extends `pmt`, and `ppaymt`/`ipaymt` split a payment into its principal/interest portions:

| Function | Meaning |
|----------|---------|
| `irate(nper, pmt, pv, fv, ptype)` | Interest rate solved numerically |
| `nper(rate, pmt, pv, fv, ptype)` | Number of periods |
| `paymt(rate, nper, pv, fv, ptype)` | Payment amount |
| `fval(rate, nper, pmt, pv, ptype)` | Future value |
| `pval(rate, nper, pmt, fv, ptype)` | Present value |
| `ipaymt(rate, per, nper, pv, fv, ptype)` | Interest portion of payment number `per` |
| `ppaymt(rate, per, nper, pv, fv, ptype)` | Principal portion of payment number `per` |

## Depreciation

| Function | Meaning |
|----------|---------|
| `sln(cost, salvage, life)` | Straight-line depreciation per interval for an item of initial value `cost` that has a value of `salvage` after `life` intervals |
| `syd(cost, salvage, life, period)` | Sum-of-the-years-digits depreciation amount for a given (positive, whole) `period`, on an item with initial `cost` and a final `salvage` value at the end of `life` intervals |
| `ddb(cost, salvage, life, period)` | Double-declining-balance depreciation of an asset for `period`, with initial `cost` and final `salvage` value at the end of `life` |
| `fdb(cost, salvage, life, period, month)` | Fixed-declining-balance depreciation — the manual calls this `Db`; `month` (partial first-year convention) can be omitted and defaults to `12` |
