//! Trigonometric, hyperbolic, and their inverse/reciprocal functions.
//!
//! v1 operates in radians only. The original Pascal engine supported a
//! degree/radian mode toggle (`CalcMode = deg/rad`) that flipped conversion
//! before/after these calls; that's deferred to a follow-up (angle-unit
//! support), since most programmatic/AI callers want radians by default.

use crate::error::{CalcError, CalcResult};

pub fn sin(x: f64) -> CalcResult<f64> {
    Ok(x.sin())
}

pub fn cos(x: f64) -> CalcResult<f64> {
    Ok(x.cos())
}

pub fn tan(x: f64) -> CalcResult<f64> {
    let c = x.cos();
    if c == 0.0 {
        return Err(CalcError::DomainError("tan".to_string()));
    }
    Ok(x.tan())
}

pub fn asin(x: f64) -> CalcResult<f64> {
    if !(-1.0..=1.0).contains(&x) {
        return Err(CalcError::DomainError("asin".to_string()));
    }
    Ok(x.asin())
}

pub fn acos(x: f64) -> CalcResult<f64> {
    if !(-1.0..=1.0).contains(&x) {
        return Err(CalcError::DomainError("acos".to_string()));
    }
    Ok(x.acos())
}

pub fn atan(x: f64) -> CalcResult<f64> {
    Ok(x.atan())
}

pub fn sinh(x: f64) -> CalcResult<f64> {
    Ok(x.sinh())
}

pub fn cosh(x: f64) -> CalcResult<f64> {
    Ok(x.cosh())
}

pub fn tanh(x: f64) -> CalcResult<f64> {
    Ok(x.tanh())
}

pub fn asinh(x: f64) -> CalcResult<f64> {
    Ok(x.asinh())
}

pub fn acosh(x: f64) -> CalcResult<f64> {
    if x < 1.0 {
        return Err(CalcError::DomainError("acosh".to_string()));
    }
    Ok(x.acosh())
}

pub fn atanh(x: f64) -> CalcResult<f64> {
    if x.abs() > 1.0 {
        return Err(CalcError::DomainError("atanh".to_string()));
    }
    Ok(x.atanh())
}

pub fn sec(x: f64) -> CalcResult<f64> {
    let c = x.cos();
    if c == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok(1.0 / c)
}

pub fn csc(x: f64) -> CalcResult<f64> {
    let s = x.sin();
    if s == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok(1.0 / s)
}

pub fn cot(x: f64) -> CalcResult<f64> {
    let s = x.sin();
    if s == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok(x.cos() / s)
}

pub fn asec(x: f64) -> CalcResult<f64> {
    if x == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    acos(1.0 / x)
}

pub fn acsc(x: f64) -> CalcResult<f64> {
    if x == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    asin(1.0 / x)
}

pub fn acot(x: f64) -> CalcResult<f64> {
    if x == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok((1.0 / x).atan())
}

pub fn sech(x: f64) -> CalcResult<f64> {
    let c = x.cosh();
    if c == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok(1.0 / c)
}

pub fn csch(x: f64) -> CalcResult<f64> {
    let s = x.sinh();
    if s == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok(1.0 / s)
}

pub fn coth(x: f64) -> CalcResult<f64> {
    let s = x.sinh();
    if s == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok(x.cosh() / s)
}

pub fn asech(x: f64) -> CalcResult<f64> {
    if x <= 0.0 || x > 1.0 {
        return Err(CalcError::DomainError("asech".to_string()));
    }
    acosh(1.0 / x)
}

pub fn acsch(x: f64) -> CalcResult<f64> {
    if x == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    asinh(1.0 / x)
}

pub fn acoth(x: f64) -> CalcResult<f64> {
    if x.abs() <= 1.0 {
        return Err(CalcError::DomainError("acoth".to_string()));
    }
    atanh(1.0 / x)
}
