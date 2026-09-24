//! Financial functions: time value of money, depreciation, and their
//! "extended" (loan-style, payment-timing-aware) variants.
//!
//! Ported from the local `Sln`/`Syd`/`Cterm`/`Term`/`Pmt`/`Rate`/`Pv`/`Npv`/`Fv`
//! functions and the `fDB`/`DDB`/`IRATE`/`nper`/`PAYMT`/`FVAL`/`IPAYMT`/
//! `PPAYMT`/`PVAL` functions defined directly in the original `MCalc.pas`
//! (not the separate `Finance.pas` unit, whose currency-scaling wrappers are
//! unused by the dispatch table). Unlike the advanced/special functions
//! category, none of these depend on numeric integration - they're all
//! closed-form (or, for `irate`, a simple secant-method root find).
//!
//! `ptype` (payment timing) is `0` for payments at the end of a period, `1`
//! for payments at the start, matching the original.

use crate::error::{CalcError, CalcResult};

/// Straight-line depreciation per interval.
pub fn sln(cost: f64, salvage: f64, life: f64) -> CalcResult<f64> {
    if life == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok((cost - salvage) / life)
}

/// Sum-of-the-years-digits depreciation amount for a given `period`.
pub fn syd(cost: f64, salvage: f64, life: f64, period: f64) -> CalcResult<f64> {
    let denom = life * (life + 1.0) / 2.0;
    if denom == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok((cost - salvage) * ((life + 1.0 - period) / denom))
}

/// Number of compounding periods for `present_value` to grow into
/// `future_value` at `rate`, with no periodic payments.
pub fn cterm(rate: f64, future_value: f64, present_value: f64) -> CalcResult<f64> {
    if present_value == 0.0 || rate == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    let ratio = future_value / present_value;
    if ratio <= 0.0 || 1.0 + rate <= 0.0 {
        return Err(CalcError::DomainError("cterm".to_string()));
    }
    Ok(ratio.ln() / (1.0 + rate).ln())
}

/// Number of periodic deposits of `payment` at `rate` needed to accumulate
/// `future_value`.
pub fn term(payment: f64, rate: f64, future_value: f64) -> CalcResult<f64> {
    if payment == 0.0 || rate == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    let inner = 1.0 + future_value * (rate / payment);
    if inner <= 0.0 || 1.0 + rate <= 0.0 {
        return Err(CalcError::DomainError("term".to_string()));
    }
    Ok(inner.ln() / (1.0 + rate).ln())
}

/// Payment amount per interval on a loan/annuity of initial value
/// `principal`, spread over `term` intervals at `rate` per interval.
pub fn pmt(principal: f64, rate: f64, term: f64) -> CalcResult<f64> {
    if rate == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    let denom = 1.0 - (1.0 + rate).powf(-term);
    if denom == 0.0 {
        return Err(CalcError::Overflow);
    }
    finite_result("pmt", principal * (rate / denom))
}

/// Interest rate per interval such that `present_value` compounded over
/// `term` intervals accumulates into `future_value`.
pub fn rate(future_value: f64, present_value: f64, term: f64) -> CalcResult<f64> {
    if present_value == 0.0 || term == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    let ratio = future_value / present_value;
    if ratio < 0.0 {
        return Err(CalcError::DomainError("rate".to_string()));
    }
    Ok(ratio.powf(1.0 / term) - 1.0)
}

/// Present value of a loan/annuity paid off with `term` payments of
/// `payment` at `rate` per interval.
pub fn pv(payment: f64, rate: f64, term: f64) -> CalcResult<f64> {
    if rate == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    finite_result("pv", payment * (1.0 - (1.0 + rate).powf(-term)) / rate)
}

/// Net present value of a sequence of cash flows discounted at `rate`.
/// `args[0]` is the rate, `args[1..]` are the cash flows (at least one).
pub fn npv(args: &[f64]) -> CalcResult<f64> {
    if args.len() < 2 {
        return Err(CalcError::WrongArgCount {
            name: "npv".to_string(),
            expected: "at least 2".to_string(),
            got: args.len(),
        });
    }
    let rate = args[0];
    let cashflows = &args[1..];
    if 1.0 + rate == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    let mut total = 0.0;
    for (i, cashflow) in cashflows.iter().enumerate() {
        total += cashflow / (1.0 + rate).powi(i as i32 + 1);
    }
    finite_result("npv", total)
}

/// Accumulated amount from making `term` payments of `payment`, with
/// interest accruing on the accumulated amount at `rate` per interval.
pub fn fv(payment: f64, rate: f64, term: f64) -> CalcResult<f64> {
    if rate == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    finite_result("fv", payment * ((1.0 + rate).powf(term) - 1.0) / rate)
}

/// Double-declining-balance depreciation of an asset for `period`, with
/// initial `cost` and final `salvage` value at the end of `life` intervals.
pub fn ddb(cost: f64, salvage: f64, life: f64, period: f64) -> CalcResult<f64> {
    if life == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    if !period.is_finite() || period < 0.0 {
        return Err(CalcError::DomainError("ddb".to_string()));
    }
    // The original loops `while period > n`, incrementing `n` by 1 each
    // time; this bound is defensive so a pathologically large `period`
    // can't hang the evaluator.
    const MAX_ITERATIONS: u64 = 100_000;
    let mut cost = cost;
    let mut x = 0.0_f64;
    let mut n = 0u64;
    while period > n as f64 {
        if n >= MAX_ITERATIONS {
            return Err(CalcError::Overflow);
        }
        x = 2.0 * cost / life;
        if cost - x < salvage {
            x = cost - salvage;
        }
        if x < 0.0 {
            x = 0.0;
        }
        cost -= x;
        n += 1;
    }
    Ok(x)
}

/// Fixed-declining-balance depreciation of an asset for `period` (the
/// original calls this `db`), with initial `cost`, final `salvage` value at
/// the end of `life` intervals, and `month` months of depreciation in the
/// first year (defaults to `12`, i.e. no partial first year, when omitted).
pub fn db(cost: f64, salvage: f64, life: f64, period: f64, month: f64) -> CalcResult<f64> {
    if life.trunc() != life || life <= 0.0 {
        return Err(CalcError::DomainError("db".to_string()));
    }
    if period.trunc() != period || period <= 0.0 || period > life + 1.0 {
        return Err(CalcError::DomainError("db".to_string()));
    }
    if month.trunc() != month || !(1.0..=12.0).contains(&month) {
        return Err(CalcError::DomainError("db".to_string()));
    }
    if cost == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    let ratio = salvage / cost;
    if ratio < 0.0 {
        return Err(CalcError::DomainError("db".to_string()));
    }
    let rate = 1.0 - ratio.powf(1.0 / life);

    let period = period as i64;
    let life = life as i64;
    let mut cumulative = 0.0;
    let mut depreciation = 0.0;
    for p in 1..=period {
        depreciation = if p == 1 {
            cost * rate * month / 12.0
        } else if p == life + 1 {
            (cost - cumulative) * rate * (12.0 - month) / 12.0
        } else {
            (cost - cumulative) * rate
        };
        cumulative += depreciation;
    }
    finite_result("db", depreciation)
}

/// Interest rate per period solved numerically (secant method) for a loan
/// or annuity with `nper` periods, periodic payment `pmt`, present value
/// `pv`, and future value `fv`.
pub fn irate(nper: f64, pmt: f64, pv: f64, fv: f64, ptype: f64) -> CalcResult<f64> {
    if nper == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    let y = |rate: f64| -> f64 {
        if rate.abs() < 1e-6 {
            pv * (1.0 + nper * rate) + pmt * (1.0 + rate * ptype) * nper + fv
        } else {
            let f = (1.0 + rate).powf(nper);
            pv * f + pmt * (1.0 / rate + ptype) * (f - 1.0) + fv
        }
    };

    let mut x0 = 0.0_f64;
    let mut y0 = y(x0);
    let mut x1 = (1.0 / nper).exp() - 1.0;
    let mut y1 = y(x1);

    const MAX_ITERATIONS: u32 = 200;
    let mut iterations = 0;
    while (y0 - y1).abs() > 1e-6 {
        // Defensive: the secant method updates `y1` to a fresh, generally
        // distinct value each iteration; `y1 == y0` (which would otherwise
        // divide by zero below) was not observed for any input tried
        // (including deliberately pathological/non-convergent ones, see
        // `irate_non_convergent_case_is_overflow`), so this is believed
        // unreachable in practice but kept as a guard.
        if y1 == y0 {
            return Err(CalcError::Overflow);
        }
        let next_rate = (y1 * x0 - y0 * x1) / (y1 - y0);
        // Defensive: same reasoning - no input was found that drives the
        // secant iterate itself to a non-finite value without first hitting
        // the `y1 == y0` guard above or the `MAX_ITERATIONS` cutoff below.
        if !next_rate.is_finite() {
            return Err(CalcError::DomainError("irate".to_string()));
        }
        x0 = x1;
        x1 = next_rate;
        y0 = y1;
        y1 = y(x1);
        iterations += 1;
        if iterations >= MAX_ITERATIONS {
            return Err(CalcError::Overflow);
        }
    }
    Ok(x1)
}

/// Extended version of `cterm`/`term`: number of periods for a loan or
/// annuity with periodic payment `pmt`, rate `rate`, present value `pv`,
/// and future value `fv`.
pub fn nper(rate: f64, pmt: f64, pv: f64, fv: f64, ptype: f64) -> CalcResult<f64> {
    let f = pmt * (1.0 + rate * ptype);
    let denom = pv * rate + f;
    if denom == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    if rate.abs() > 1e-6 {
        let ratio = (f - rate * fv) / denom;
        if ratio <= 0.0 || 1.0 + rate <= 0.0 {
            return Err(CalcError::DomainError("nper".to_string()));
        }
        Ok(ratio.ln() / (1.0 + rate).ln())
    } else {
        Ok(-(fv + pv) / denom)
    }
}

/// Extended version of `pmt`: payment amount for a loan or annuity with
/// `nper` periods, rate `rate`, present value `pv`, and future value `fv`.
pub fn paymt(rate: f64, nper: f64, pv: f64, fv: f64, ptype: f64) -> CalcResult<f64> {
    let f = (1.0 + rate).powf(nper);
    let denom = (1.0 + rate * ptype) * (1.0 - f);
    if denom == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    finite_result("paymt", (fv + pv * f) * rate / denom)
}

/// Extended version of `fv`: future value for a loan or annuity with `nper`
/// periods, rate `rate`, periodic payment `pmt`, and present value `pv`.
pub fn fval(rate: f64, nper: f64, pmt: f64, pv: f64, ptype: f64) -> CalcResult<f64> {
    let f = (1.0 + rate).powf(nper);
    let result = if rate.abs() < 1e-6 {
        -pmt * nper * (1.0 + (nper - 1.0) * rate / 2.0) * (1.0 + rate * ptype) - pv * f
    } else {
        pmt * (1.0 - f) * (1.0 / rate + ptype) - pv * f
    };
    finite_result("fval", result)
}

/// Extended version of `pv`: present value for a loan or annuity with
/// `nper` periods, rate `rate`, periodic payment `pmt`, and future value
/// `fv`.
pub fn pval(rate: f64, nper: f64, pmt: f64, fv: f64, ptype: f64) -> CalcResult<f64> {
    if rate.abs() > 1e-6 {
        let f = (1.0 + rate).powf(nper);
        if f == 0.0 {
            return Err(CalcError::DivisionByZero);
        }
        finite_result("pval", (pmt * (1.0 / rate + ptype) * (1.0 - f) - fv) / f)
    } else {
        let denom = 1.0 + nper * rate;
        if denom == 0.0 {
            return Err(CalcError::DivisionByZero);
        }
        finite_result("pval", -(pmt * (1.0 + rate * ptype) * nper + fv) / denom)
    }
}

/// Interest portion of payment number `per` on a loan or annuity with
/// `nper` periods, rate `rate`, present value `pv`, and future value `fv`.
pub fn ipaymt(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, ptype: f64) -> CalcResult<f64> {
    let payment = paymt(rate, nper, pv, fv, ptype)?;
    let f = fval(rate, per - ptype - 1.0, payment, pv, ptype)?;
    finite_result("ipaymt", rate * f)
}

/// Principal portion of payment number `per` on a loan or annuity with
/// `nper` periods, rate `rate`, present value `pv`, and future value `fv`.
pub fn ppaymt(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, ptype: f64) -> CalcResult<f64> {
    let payment = paymt(rate, nper, pv, fv, ptype)?;
    let f = fval(rate, per - ptype - 1.0, payment, pv, ptype)?;
    finite_result("ppaymt", payment - rate * f)
}

/// Maps a NaN result (invalid domain, e.g. a fractional power of a
/// negative base) to `DomainError`, and an infinite result (genuine
/// magnitude overflow) to `Overflow`.
fn finite_result(name: &str, value: f64) -> CalcResult<f64> {
    if value.is_nan() {
        Err(CalcError::DomainError(name.to_string()))
    } else if value.is_infinite() {
        Err(CalcError::Overflow)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "expected {b}, got {a}");
    }

    #[test]
    fn sln_matches_known_value() {
        assert_close(sln(10000.0, 1000.0, 5.0).unwrap(), 1800.0, 1e-9);
    }

    #[test]
    fn sln_zero_life_is_division_by_zero() {
        assert_eq!(sln(10000.0, 1000.0, 0.0), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn syd_matches_known_value() {
        assert_close(syd(10000.0, 1000.0, 5.0, 1.0).unwrap(), 3000.0, 1e-9);
    }

    #[test]
    fn syd_zero_life_is_division_by_zero() {
        assert_eq!(
            syd(10000.0, 1000.0, 0.0, 1.0),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn cterm_matches_known_value() {
        assert_close(cterm(0.1, 2000.0, 1000.0).unwrap(), 7.272540897341713, 1e-9);
    }

    #[test]
    fn cterm_zero_rate_is_division_by_zero() {
        assert_eq!(cterm(0.0, 2000.0, 1000.0), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn cterm_non_positive_ratio_is_domain_error() {
        assert_eq!(
            cterm(0.1, -2000.0, 1000.0),
            Err(CalcError::DomainError("cterm".to_string()))
        );
    }

    #[test]
    fn term_matches_known_value() {
        assert_close(term(100.0, 0.01, 5000.0).unwrap(), 40.74890715609402, 1e-9);
    }

    #[test]
    fn term_zero_payment_is_division_by_zero() {
        assert_eq!(term(0.0, 0.01, 5000.0), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn term_non_positive_inner_is_domain_error() {
        assert_eq!(
            term(1.0, 1.0, -2.0),
            Err(CalcError::DomainError("term".to_string()))
        );
    }

    #[test]
    fn pmt_matches_known_value() {
        assert_close(pmt(10000.0, 0.08, 5.0).unwrap(), 2504.564545668364, 1e-6);
    }

    #[test]
    fn pmt_zero_rate_is_division_by_zero() {
        assert_eq!(pmt(10000.0, 0.0, 5.0), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn pmt_zero_term_is_overflow() {
        // denom = 1 - (1+rate)^0 = 0 for any rate when term is 0.
        assert_eq!(pmt(1000.0, 0.05, 0.0), Err(CalcError::Overflow));
    }

    #[test]
    fn rate_matches_known_value() {
        assert_close(
            rate(2000.0, 1000.0, 10.0).unwrap(),
            0.07177346253629313,
            1e-9,
        );
    }

    #[test]
    fn rate_zero_term_is_division_by_zero() {
        assert_eq!(rate(2000.0, 1000.0, 0.0), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn rate_negative_ratio_is_domain_error() {
        assert_eq!(
            rate(-100.0, 100.0, 5.0),
            Err(CalcError::DomainError("rate".to_string()))
        );
    }

    #[test]
    fn pv_matches_known_value() {
        assert_close(pv(1000.0, 0.08, 5.0).unwrap(), 3992.7100370780886, 1e-6);
    }

    #[test]
    fn pv_zero_rate_is_division_by_zero() {
        assert_eq!(pv(1000.0, 0.0, 5.0), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn pv_negative_base_fractional_term_is_domain_error() {
        // (1+rate)^-term is NaN for a negative base and non-integer term.
        assert_eq!(
            pv(100.0, -2.0, 0.5),
            Err(CalcError::DomainError("pv".to_string()))
        );
    }

    #[test]
    fn fv_matches_known_value() {
        assert_close(fv(1000.0, 0.08, 5.0).unwrap(), 5866.600960000006, 1e-6);
    }

    #[test]
    fn fv_zero_rate_is_division_by_zero() {
        assert_eq!(fv(1000.0, 0.0, 5.0), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn fv_huge_growth_is_overflow() {
        assert_eq!(fv(1.0, 1e10, 1e10), Err(CalcError::Overflow));
    }

    #[test]
    fn npv_matches_known_value() {
        assert_close(
            npv(&[0.1, 100.0, 200.0, 300.0]).unwrap(),
            481.59278737791124,
            1e-9,
        );
    }

    #[test]
    fn npv_requires_at_least_one_cashflow() {
        assert_eq!(
            npv(&[0.1]),
            Err(CalcError::WrongArgCount {
                name: "npv".to_string(),
                expected: "at least 2".to_string(),
                got: 1,
            })
        );
    }

    #[test]
    fn npv_rate_of_negative_one_is_division_by_zero() {
        assert_eq!(npv(&[-1.0, 100.0]), Err(CalcError::DivisionByZero));
    }

    #[test]
    fn ddb_matches_known_value() {
        assert_close(ddb(10000.0, 1000.0, 5.0, 1.0).unwrap(), 4000.0, 1e-9);
    }

    #[test]
    fn ddb_zero_life_is_division_by_zero() {
        assert_eq!(
            ddb(10000.0, 1000.0, 0.0, 1.0),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn ddb_negative_period_is_domain_error() {
        assert_eq!(
            ddb(10000.0, 1000.0, 5.0, -1.0),
            Err(CalcError::DomainError("ddb".to_string()))
        );
    }

    #[test]
    fn ddb_clamps_to_zero_when_salvage_exceeds_cost() {
        // Not a realistic input (salvage > cost), but the original has no
        // domain check preventing it: the first period's depreciation would
        // be negative, which the original clamps to 0.
        assert_close(ddb(1000.0, 2000.0, 5.0, 1.0).unwrap(), 0.0, 1e-9);
    }

    #[test]
    fn ddb_exceeding_max_iterations_is_overflow() {
        assert_eq!(
            ddb(10000.0, 1000.0, 5.0, 100_001.0),
            Err(CalcError::Overflow)
        );
    }

    #[test]
    fn db_matches_known_value() {
        assert_close(
            db(50000.0, 10000.0, 5.0, 1.0, 3.0).unwrap(),
            3440.254204028806,
            1e-6,
        );
    }

    #[test]
    fn db_rejects_invalid_period() {
        assert_eq!(
            db(50000.0, 10000.0, 5.0, 7.0, 12.0),
            Err(CalcError::DomainError("db".to_string()))
        );
    }

    #[test]
    fn db_rejects_invalid_month() {
        assert_eq!(
            db(50000.0, 10000.0, 5.0, 1.0, 13.0),
            Err(CalcError::DomainError("db".to_string()))
        );
    }

    #[test]
    fn db_rejects_invalid_life() {
        assert_eq!(
            db(50000.0, 10000.0, 0.0, 1.0, 12.0),
            Err(CalcError::DomainError("db".to_string()))
        );
    }

    #[test]
    fn db_zero_cost_is_division_by_zero() {
        assert_eq!(
            db(0.0, 10000.0, 5.0, 1.0, 12.0),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn db_negative_salvage_over_cost_ratio_is_domain_error() {
        assert_eq!(
            db(50000.0, -1000.0, 5.0, 1.0, 12.0),
            Err(CalcError::DomainError("db".to_string()))
        );
    }

    #[test]
    fn db_covers_middle_and_final_year_branches() {
        // period = life + 1 exercises the partial-final-year branch, and
        // the periods in between exercise the "else" (middle years) branch.
        assert_close(
            db(50000.0, 10000.0, 5.0, 6.0, 3.0).unwrap(),
            2652.017476323653,
            1e-6,
        );
    }

    #[test]
    fn extended_variants_round_trip() {
        let payment = paymt(0.08, 5.0, -10000.0, 0.0, 0.0).unwrap();
        assert_close(payment, 2504.564545668364, 1e-6);
        assert_close(fval(0.08, 5.0, payment, -10000.0, 0.0).unwrap(), 0.0, 1e-6);
        assert_close(nper(0.08, payment, -10000.0, 0.0, 0.0).unwrap(), 5.0, 1e-6);
        assert_close(pval(0.08, 5.0, payment, 0.0, 0.0).unwrap(), -10000.0, 1e-6);
        assert_close(irate(5.0, payment, -10000.0, 0.0, 0.0).unwrap(), 0.08, 1e-6);
    }

    #[test]
    fn irate_zero_nper_is_division_by_zero() {
        assert_eq!(
            irate(0.0, 100.0, -1000.0, 0.0, 0.0),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn irate_non_convergent_case_is_overflow() {
        assert_eq!(
            irate(
                10.5,
                -15.04691491034157,
                790.010300430693,
                89.59135283589649,
                0.0
            ),
            Err(CalcError::Overflow)
        );
    }

    #[test]
    fn nper_zero_denominator_is_division_by_zero() {
        assert_eq!(
            nper(0.05, 0.0, 0.0, 100.0, 0.0),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn nper_domain_error_when_ratio_non_positive() {
        assert_eq!(
            nper(0.1, 100.0, 100.0, 2000.0, 0.0),
            Err(CalcError::DomainError("nper".to_string()))
        );
    }

    #[test]
    fn nper_near_zero_rate_uses_linear_formula() {
        assert_close(nper(0.0, 100.0, 100.0, 1000.0, 0.0).unwrap(), -11.0, 1e-9);
    }

    #[test]
    fn paymt_zero_denominator_is_division_by_zero() {
        // (1 + rate*ptype) == 0 makes the denominator 0 regardless of `f`.
        assert_eq!(
            paymt(0.1, 5.0, 100.0, 100.0, -10.0),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn fval_near_zero_rate_uses_linear_formula() {
        assert!(fval(0.0, 5.0, 100.0, -1000.0, 0.0).unwrap().is_finite());
    }

    #[test]
    fn pval_zero_denominator_is_division_by_zero() {
        assert_eq!(
            pval(-1.0, 5.0, 100.0, 100.0, 0.0),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn pval_near_zero_rate_uses_linear_formula() {
        assert_close(pval(0.0, 5.0, 100.0, -1000.0, 0.0).unwrap(), 500.0, 1e-9);
    }

    #[test]
    fn pval_near_zero_rate_zero_denominator_is_division_by_zero() {
        assert_eq!(
            pval(1e-7, -1e7, 100.0, 0.0, 0.0),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn ipaymt_and_ppaymt_sum_to_paymt() {
        let payment = paymt(0.08, 5.0, -10000.0, 0.0, 0.0).unwrap();
        let interest = ipaymt(0.08, 1.0, 5.0, -10000.0, 0.0, 0.0).unwrap();
        let principal = ppaymt(0.08, 1.0, 5.0, -10000.0, 0.0, 0.0).unwrap();
        assert_close(interest + principal, payment, 1e-6);
    }
}
