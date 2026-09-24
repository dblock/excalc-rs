# Financial

**Status: planned, not yet implemented.** See [../../AGENTS.md](../../AGENTS.md#porting-from-the-original-pascal-engine) for porting conventions.

## Time value of money

| Function | Meaning |
|----------|---------|
| `pv(payment, rate, term)` | Present value of a series of payments |
| `fv(payment, rate, term)` | Future value of a series of payments |
| `pmt(principal, rate, term)` | Payment amount to amortize `principal` over `term` periods |
| `npv(rate, cashflow1, cashflow2, ...)` | Net present value of a series of cash flows |
| `rate(futureValue, presentValue, term)` | Periodic interest rate implied by a present/future value pair |
| `cterm(rate, futureValue, presentValue)` | Number of compounding periods needed to reach `futureValue` |
| `term(payment, rate, futureValue)` | Number of payments needed to reach `futureValue` |

## Extended variants (support a payment-timing flag)

These mirror the functions above but add an extra `ptype` parameter (`0` = payments at end of period, `1` = payments at start of period):

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
| `sln(initialValue, residue, time)` | Straight-line depreciation per period |
| `syd(initialValue, residue, period, time)` | Sum-of-the-years-digits depreciation for a given `period` |
| `ddb(cost, salvage, life, period)` | Double-declining-balance depreciation for `period` |
| `fdb(cost, salvage, life, period, month)` | Fixed-declining-balance depreciation, with partial first-year `month` convention |
