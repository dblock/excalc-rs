//! Unit conversion functions: temperature, distance, and mass. Each is a
//! simple linear (affine) conversion; degree/radian conversion is already
//! covered by `trig::deg`/`trig::rad` and isn't duplicated here.

use crate::error::CalcResult;

/// Exact conversion factor: `1 mile = 1.609344 km`.
const KM_PER_MILE: f64 = 1.609344;

/// Exact conversion factor: `1 lb = 0.45359237 kg`.
const KG_PER_LB: f64 = 0.453_592_37;

/// Exact conversion factor: `1 foot = 0.3048 m`.
const M_PER_FOOT: f64 = 0.3048;

/// Converts a temperature in Celsius to Fahrenheit.
pub fn c2f(c: f64) -> CalcResult<f64> {
    Ok(c * 9.0 / 5.0 + 32.0)
}

/// Converts a temperature in Fahrenheit to Celsius.
pub fn f2c(f: f64) -> CalcResult<f64> {
    Ok((f - 32.0) * 5.0 / 9.0)
}

/// Converts a distance in kilometers to miles.
pub fn km2mi(km: f64) -> CalcResult<f64> {
    Ok(km / KM_PER_MILE)
}

/// Converts a distance in miles to kilometers.
pub fn mi2km(mi: f64) -> CalcResult<f64> {
    Ok(mi * KM_PER_MILE)
}

/// Converts a mass in kilograms to pounds.
pub fn kg2lb(kg: f64) -> CalcResult<f64> {
    Ok(kg / KG_PER_LB)
}

/// Converts a mass in pounds to kilograms.
pub fn lb2kg(lb: f64) -> CalcResult<f64> {
    Ok(lb * KG_PER_LB)
}

/// Converts a distance in meters to feet.
pub fn m2ft(m: f64) -> CalcResult<f64> {
    Ok(m / M_PER_FOOT)
}

/// Converts a distance in feet to meters.
pub fn ft2m(ft: f64) -> CalcResult<f64> {
    Ok(ft * M_PER_FOOT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c2f_matches_known_values() {
        assert_eq!(c2f(0.0).unwrap(), 32.0);
        assert_eq!(c2f(100.0).unwrap(), 212.0);
        assert_eq!(c2f(-40.0).unwrap(), -40.0);
    }

    #[test]
    fn f2c_matches_known_values() {
        assert_eq!(f2c(32.0).unwrap(), 0.0);
        assert_eq!(f2c(212.0).unwrap(), 100.0);
        assert_eq!(f2c(-40.0).unwrap(), -40.0);
    }

    #[test]
    fn c2f_and_f2c_are_inverses() {
        let c = 37.0;
        assert!((f2c(c2f(c).unwrap()).unwrap() - c).abs() < 1e-9);
    }

    #[test]
    fn km2mi_matches_known_value() {
        let result = km2mi(1.0).unwrap();
        assert!((result - 0.621_371_192).abs() < 1e-9);
    }

    #[test]
    fn mi2km_matches_known_value() {
        assert_eq!(mi2km(1.0).unwrap(), 1.609344);
    }

    #[test]
    fn km_mi_are_inverses() {
        let km = 42.195; // marathon distance
        assert!((mi2km(km2mi(km).unwrap()).unwrap() - km).abs() < 1e-9);
    }

    #[test]
    fn kg2lb_matches_known_value() {
        let result = kg2lb(1.0).unwrap();
        assert!((result - 2.204_622_622).abs() < 1e-8);
    }

    #[test]
    fn lb2kg_matches_known_value() {
        assert_eq!(lb2kg(1.0).unwrap(), 0.45359237);
    }

    #[test]
    fn kg_lb_are_inverses() {
        let kg = 70.0;
        assert!((lb2kg(kg2lb(kg).unwrap()).unwrap() - kg).abs() < 1e-9);
    }

    #[test]
    fn m2ft_matches_known_value() {
        let result = m2ft(1.0).unwrap();
        assert!((result - 3.280_839_895).abs() < 1e-8);
    }

    #[test]
    fn ft2m_matches_known_value() {
        assert_eq!(ft2m(1.0).unwrap(), 0.3048);
    }

    #[test]
    fn m_ft_are_inverses() {
        let m = 1_000.0;
        assert!((ft2m(m2ft(m).unwrap()).unwrap() - m).abs() < 1e-6);
    }
}
