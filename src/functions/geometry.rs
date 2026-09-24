//! Geometry: n-dimensional vector/point operations plus dimension-free
//! shape formulas (areas, volumes).
//!
//! `distance`, `manhattan`, and `dot` all take an even-length argument
//! list, split into two equal halves representing the coordinates of two
//! points/vectors `a` and `b` (e.g. `distance(1, 2, 4, 6)` is the 2D
//! distance between `(1, 2)` and `(4, 6)`; `distance(0, 0, 0, 1, 1, 1)` is
//! the 3D distance between `(0, 0, 0)` and `(1, 1, 1)`). This lets the same
//! function work in any dimension without a dedicated argument per axis.

use crate::error::{CalcError, CalcResult};

/// Splits `args` in half, erroring with `WrongArgCount(name)` unless the
/// length is even and at least `2` (i.e. at least one coordinate per
/// point/vector).
fn split_pair<'a>(name: &str, args: &'a [f64]) -> CalcResult<(&'a [f64], &'a [f64])> {
    if args.len() < 2 || !args.len().is_multiple_of(2) {
        return Err(CalcError::WrongArgCount {
            name: name.to_string(),
            expected: "an even number, at least 2".to_string(),
            got: args.len(),
        });
    }
    let half = args.len() / 2;
    Ok((&args[..half], &args[half..]))
}

/// Euclidean distance between two n-dimensional points, given as a single
/// argument list split in half (see module docs).
pub fn distance(args: &[f64]) -> CalcResult<f64> {
    let (a, b) = split_pair("distance", args)?;
    Ok(a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt())
}

/// Manhattan (taxicab, L1) distance between two n-dimensional points, given
/// as a single argument list split in half (see module docs).
pub fn manhattan(args: &[f64]) -> CalcResult<f64> {
    let (a, b) = split_pair("manhattan", args)?;
    Ok(a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum())
}

/// Dot product of two n-dimensional vectors, given as a single argument
/// list split in half (see module docs).
pub fn dot(args: &[f64]) -> CalcResult<f64> {
    let (a, b) = split_pair("dot", args)?;
    Ok(a.iter().zip(b).map(|(x, y)| x * y).sum())
}

/// Euclidean norm (magnitude) of an n-dimensional vector.
pub fn norm(args: &[f64]) -> CalcResult<f64> {
    if args.is_empty() {
        return Err(CalcError::WrongArgCount {
            name: "norm".to_string(),
            expected: "at least 1".to_string(),
            got: 0,
        });
    }
    Ok(args.iter().map(|x| x.powi(2)).sum::<f64>().sqrt())
}

/// Area of a triangle with side lengths `a`, `b`, `c`, via Heron's formula:
/// `s = (a+b+c)/2`, `area = sqrt(s(s-a)(s-b)(s-c))`. Errors if the sides
/// can't form a valid triangle (any side `<= 0`, or the triangle
/// inequality is violated).
pub fn triarea(a: f64, b: f64, c: f64) -> CalcResult<f64> {
    if a <= 0.0 || b <= 0.0 || c <= 0.0 || a + b <= c || a + c <= b || b + c <= a {
        return Err(CalcError::DomainError("triarea".to_string()));
    }
    let s = (a + b + c) / 2.0;
    Ok((s * (s - a) * (s - b) * (s - c)).sqrt())
}

/// Area of a circle of radius `r`: `pi * r^2`.
pub fn circlearea(r: f64) -> CalcResult<f64> {
    if r <= 0.0 {
        return Err(CalcError::DomainError("circlearea".to_string()));
    }
    Ok(std::f64::consts::PI * r * r)
}

/// Circumference of a circle of radius `r`: `2 * pi * r`.
pub fn circumference(r: f64) -> CalcResult<f64> {
    if r <= 0.0 {
        return Err(CalcError::DomainError("circumference".to_string()));
    }
    Ok(2.0 * std::f64::consts::PI * r)
}

/// Volume of a sphere of radius `r`: `(4/3) * pi * r^3`.
pub fn spherevol(r: f64) -> CalcResult<f64> {
    if r <= 0.0 {
        return Err(CalcError::DomainError("spherevol".to_string()));
    }
    Ok(4.0 / 3.0 * std::f64::consts::PI * r.powi(3))
}

/// Surface area of a sphere of radius `r`: `4 * pi * r^2`.
pub fn spherearea(r: f64) -> CalcResult<f64> {
    if r <= 0.0 {
        return Err(CalcError::DomainError("spherearea".to_string()));
    }
    Ok(4.0 * std::f64::consts::PI * r * r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_matches_2d_pythagorean_triple() {
        assert_eq!(distance(&[0.0, 0.0, 3.0, 4.0]).unwrap(), 5.0);
    }

    #[test]
    fn distance_works_in_3d() {
        let result = distance(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]).unwrap();
        assert!((result - 3f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn distance_rejects_odd_or_short_arg_counts() {
        assert!(matches!(
            distance(&[1.0, 2.0, 3.0]),
            Err(CalcError::WrongArgCount { .. })
        ));
        assert!(matches!(
            distance(&[1.0]),
            Err(CalcError::WrongArgCount { .. })
        ));
    }

    #[test]
    fn manhattan_matches_known_value() {
        assert_eq!(manhattan(&[0.0, 0.0, 3.0, 4.0]).unwrap(), 7.0);
    }

    #[test]
    fn manhattan_works_in_3d() {
        assert_eq!(
            manhattan(&[1.0, 2.0, 3.0, 4.0, 0.0, 6.0]).unwrap(),
            3.0 + 2.0 + 3.0
        );
    }

    #[test]
    fn dot_matches_known_value() {
        assert_eq!(dot(&[1.0, 2.0, 3.0, 4.0]).unwrap(), 11.0);
    }

    #[test]
    fn dot_works_in_3d() {
        assert_eq!(dot(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0]).unwrap(), 0.0);
    }

    #[test]
    fn norm_matches_pythagorean_triple() {
        assert_eq!(norm(&[3.0, 4.0]).unwrap(), 5.0);
    }

    #[test]
    fn norm_works_in_3d() {
        assert_eq!(norm(&[2.0, 3.0, 6.0]).unwrap(), 7.0);
    }

    #[test]
    fn norm_rejects_empty_args() {
        assert!(matches!(norm(&[]), Err(CalcError::WrongArgCount { .. })));
    }

    #[test]
    fn triarea_matches_known_3_4_5_triangle() {
        assert_eq!(triarea(3.0, 4.0, 5.0).unwrap(), 6.0);
    }

    #[test]
    fn triarea_rejects_invalid_triangles() {
        assert!(matches!(
            triarea(1.0, 2.0, 100.0),
            Err(CalcError::DomainError(_))
        ));
        assert!(matches!(
            triarea(0.0, 1.0, 1.0),
            Err(CalcError::DomainError(_))
        ));
    }

    #[test]
    fn circlearea_matches_known_value() {
        let result = circlearea(2.0).unwrap();
        assert!((result - 4.0 * std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn circumference_matches_known_value() {
        let result = circumference(2.0).unwrap();
        assert!((result - 4.0 * std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn spherevol_matches_known_value() {
        let result = spherevol(3.0).unwrap();
        assert!((result - 36.0 * std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn spherearea_matches_known_value() {
        let result = spherearea(3.0).unwrap();
        assert!((result - 36.0 * std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn radius_domain_errors_reject_non_positive() {
        assert!(matches!(circlearea(0.0), Err(CalcError::DomainError(_))));
        assert!(matches!(
            circumference(-1.0),
            Err(CalcError::DomainError(_))
        ));
        assert!(matches!(spherevol(0.0), Err(CalcError::DomainError(_))));
        assert!(matches!(spherearea(-1.0), Err(CalcError::DomainError(_))));
    }
}
