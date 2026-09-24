//! Trigonometric, hyperbolic, and their inverse/reciprocal functions.
//!
//! The circular trig functions below (`sin`/`cos`/`tan`/`asin`/`acos`/`atan`
//! and their reciprocals `sec`/`csc`/`cot`/`asec`/`acsc`/`acot`) operate in
//! **radians only**, matching most programmatic/AI callers' expectations.
//! Rather than reviving the original Pascal engine's global `CalcMode =
//! deg/rad` toggle (implicit state that would silently change how every
//! trig call behaves), degree support is stateless and explicit:
//! [`deg`]/[`rad`] convert between the two units, and `sind`/`cosd`/`tand`/
//! `asind`/`acosd`/`atand` are degree-native convenience wrappers around the
//! six basic circular functions. Hyperbolic functions operate on plain real
//! numbers, not angles, so they have no degree/radian variants.

use crate::error::{CalcError, CalcResult};

const DEG_PER_RAD: f64 = 180.0 / std::f64::consts::PI;
const RAD_PER_DEG: f64 = std::f64::consts::PI / 180.0;

/// Converts an angle in radians to degrees.
pub fn deg(x: f64) -> CalcResult<f64> {
    Ok(x * DEG_PER_RAD)
}

/// Converts an angle in degrees to radians.
pub fn rad(x: f64) -> CalcResult<f64> {
    Ok(x * RAD_PER_DEG)
}

pub fn sin(x: f64) -> CalcResult<f64> {
    Ok(x.sin())
}

pub fn cos(x: f64) -> CalcResult<f64> {
    Ok(x.cos())
}

pub fn tan(x: f64) -> CalcResult<f64> {
    let c = x.cos();
    // In practice `c == 0.0` is unreachable for any finite f64 input (no
    // exact IEEE-754 zero of cos exists), so this guard is defensive and
    // untestable; kept for symmetry/documentation of the mathematical domain.
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

/// `sin`, taking its argument in degrees instead of radians.
pub fn sind(x: f64) -> CalcResult<f64> {
    sin(x * RAD_PER_DEG)
}

/// `cos`, taking its argument in degrees instead of radians.
pub fn cosd(x: f64) -> CalcResult<f64> {
    cos(x * RAD_PER_DEG)
}

/// `tan`, taking its argument in degrees instead of radians.
pub fn tand(x: f64) -> CalcResult<f64> {
    tan(x * RAD_PER_DEG)
}

/// `asin`, returning its result in degrees instead of radians.
pub fn asind(x: f64) -> CalcResult<f64> {
    asin(x).map(|r| r * DEG_PER_RAD)
}

/// `acos`, returning its result in degrees instead of radians.
pub fn acosd(x: f64) -> CalcResult<f64> {
    acos(x).map(|r| r * DEG_PER_RAD)
}

/// `atan`, returning its result in degrees instead of radians.
pub fn atand(x: f64) -> CalcResult<f64> {
    atan(x).map(|r| r * DEG_PER_RAD)
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
    // See `tan`: unreachable in practice, kept for domain documentation.
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
    // cosh(x) >= 1 for every real x, so this is mathematically unreachable;
    // kept as a defensive guard rather than an `unwrap`/panic.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degree_radian_conversion() {
        assert!((deg(std::f64::consts::PI).unwrap() - 180.0).abs() < 1e-12);
        assert!((rad(180.0).unwrap() - std::f64::consts::PI).abs() < 1e-12);
        assert!((deg(rad(30.0).unwrap()).unwrap() - 30.0).abs() < 1e-12);
    }

    #[test]
    fn degree_native_trig_functions() {
        assert!((sind(30.0).unwrap() - 0.5).abs() < 1e-9);
        assert!((cosd(60.0).unwrap() - 0.5).abs() < 1e-9);
        assert!((tand(45.0).unwrap() - 1.0).abs() < 1e-9);
        assert!((asind(0.5).unwrap() - 30.0).abs() < 1e-9);
        assert!((acosd(0.5).unwrap() - 60.0).abs() < 1e-9);
        assert!((atand(1.0).unwrap() - 45.0).abs() < 1e-9);
    }

    #[test]
    fn plain_trig_functions() {
        assert!((sin(0.0).unwrap() - 0.0).abs() < 1e-12);
        assert!((cos(0.0).unwrap() - 1.0).abs() < 1e-12);
        assert!((tan(0.0).unwrap() - 0.0).abs() < 1e-12);
        assert!((atan(1.0).unwrap() - std::f64::consts::FRAC_PI_4).abs() < 1e-12);
    }

    #[test]
    fn inverse_trig_domain() {
        assert!(asin(0.5).is_ok());
        assert_eq!(asin(2.0), Err(CalcError::DomainError("asin".to_string())));
        assert_eq!(asin(-2.0), Err(CalcError::DomainError("asin".to_string())));
        assert!(acos(0.5).is_ok());
        assert_eq!(acos(2.0), Err(CalcError::DomainError("acos".to_string())));
        assert_eq!(acos(-2.0), Err(CalcError::DomainError("acos".to_string())));
    }

    #[test]
    fn hyperbolic_functions() {
        assert!((sinh(0.0).unwrap() - 0.0).abs() < 1e-12);
        assert!((cosh(0.0).unwrap() - 1.0).abs() < 1e-12);
        assert!((tanh(0.0).unwrap() - 0.0).abs() < 1e-12);
        assert!((asinh(0.0).unwrap() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn inverse_hyperbolic_domain() {
        assert!(acosh(2.0).is_ok());
        assert_eq!(acosh(0.5), Err(CalcError::DomainError("acosh".to_string())));
        assert!(atanh(0.5).is_ok());
        assert_eq!(atanh(1.5), Err(CalcError::DomainError("atanh".to_string())));
        assert_eq!(
            atanh(-1.5),
            Err(CalcError::DomainError("atanh".to_string()))
        );
    }

    #[test]
    fn reciprocal_trig_functions() {
        assert!((sec(0.0).unwrap() - 1.0).abs() < 1e-12);
        assert_eq!(csc(0.0), Err(CalcError::DivisionByZero));
        assert_eq!(cot(0.0), Err(CalcError::DivisionByZero));
        assert!((csc(std::f64::consts::FRAC_PI_2).unwrap() - 1.0).abs() < 1e-12);
        assert!((cot(std::f64::consts::FRAC_PI_2).unwrap() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn inverse_reciprocal_trig_functions() {
        assert_eq!(asec(0.0), Err(CalcError::DivisionByZero));
        assert!(asec(2.0).is_ok());
        assert_eq!(acsc(0.0), Err(CalcError::DivisionByZero));
        assert!(acsc(2.0).is_ok());
        assert_eq!(acot(0.0), Err(CalcError::DivisionByZero));
        assert!(acot(2.0).is_ok());
    }

    #[test]
    fn reciprocal_hyperbolic_functions() {
        assert!((sech(0.0).unwrap() - 1.0).abs() < 1e-12);
        assert_eq!(csch(0.0), Err(CalcError::DivisionByZero));
        assert_eq!(coth(0.0), Err(CalcError::DivisionByZero));
        assert!(csch(1.0).is_ok());
        assert!(coth(1.0).is_ok());
    }

    #[test]
    fn inverse_reciprocal_hyperbolic_functions() {
        assert!(asech(0.5).is_ok());
        assert_eq!(asech(0.0), Err(CalcError::DomainError("asech".to_string())));
        assert_eq!(asech(2.0), Err(CalcError::DomainError("asech".to_string())));
        assert_eq!(acsch(0.0), Err(CalcError::DivisionByZero));
        assert!(acsch(2.0).is_ok());
        assert!(acoth(2.0).is_ok());
        assert_eq!(acoth(0.5), Err(CalcError::DomainError("acoth".to_string())));
        assert_eq!(
            acoth(-0.5),
            Err(CalcError::DomainError("acoth".to_string()))
        );
    }
}
