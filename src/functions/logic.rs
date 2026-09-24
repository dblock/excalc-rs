//! Fixed-arity logical/bitwise functions (`not`, `shl`, `shr`). The binary
//! bitwise operators (`and`, `or`, `xor`, ...) are implemented directly in
//! `eval.rs` alongside the other `BinaryOp` variants.

use crate::error::{CalcError, CalcResult};

/// Truncates towards zero and converts to `i64`, erroring with
/// `DomainError(name)` if the value is outside `i64`'s range.
pub(crate) fn to_i64(x: f64, name: &str) -> CalcResult<i64> {
    let t = x.trunc();
    if t < i64::MIN as f64 || t > i64::MAX as f64 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(t as i64)
}

/// Bitwise not (two's-complement, `!x` in Rust, i.e. `~x = -x - 1`).
pub fn not(x: f64) -> CalcResult<f64> {
    Ok(!to_i64(x, "not")? as f64)
}

/// Shift `x` left by `y` bits, i.e. `x * 2^y` (as a bitwise shift on `i64`,
/// wrapping if bits are shifted out — consistent with the other bitwise
/// operators, which also operate on fixed-width two's-complement integers).
pub fn shl(x: f64, y: f64) -> CalcResult<f64> {
    let x = to_i64(x, "shl")?;
    let y = to_i64(y, "shl")?;
    if !(0..64).contains(&y) {
        return Err(CalcError::DomainError("shl".to_string()));
    }
    Ok(x.wrapping_shl(y as u32) as f64)
}

/// Shift `x` right by `y` bits, i.e. `x / 2^y`.
pub fn shr(x: f64, y: f64) -> CalcResult<f64> {
    let x = to_i64(x, "shr")?;
    let y = to_i64(y, "shr")?;
    if !(0..64).contains(&y) {
        return Err(CalcError::DomainError("shr".to_string()));
    }
    Ok((x >> y) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_matches_manual_example() {
        assert_eq!(not(1.0).unwrap(), -2.0);
        assert_eq!(not(0.0).unwrap(), -1.0);
    }

    #[test]
    fn not_domain_error_out_of_range() {
        assert_eq!(not(1e30), Err(CalcError::DomainError("not".to_string())));
    }

    #[test]
    fn shl_matches_manual_example() {
        assert_eq!(shl(2.0, 1.0).unwrap(), 4.0);
        assert_eq!(shl(1.0, 10.0).unwrap(), 1024.0);
    }

    #[test]
    fn shl_domain_and_wraps() {
        assert_eq!(
            shl(1.0, -1.0),
            Err(CalcError::DomainError("shl".to_string()))
        );
        assert_eq!(
            shl(1.0, 64.0),
            Err(CalcError::DomainError("shl".to_string()))
        );
        // Shifting bits out the top wraps rather than erroring, consistent
        // with the other bitwise operators.
        assert_eq!(shl(i64::MAX as f64, 4.0).unwrap(), -16.0);
    }

    #[test]
    fn shr_matches_manual_example() {
        assert_eq!(shr(2.0, 1.0).unwrap(), 1.0);
        assert_eq!(shr(1024.0, 10.0).unwrap(), 1.0);
    }

    #[test]
    fn shr_domain_error() {
        assert_eq!(
            shr(1.0, -1.0),
            Err(CalcError::DomainError("shr".to_string()))
        );
        assert_eq!(
            shr(1.0, 64.0),
            Err(CalcError::DomainError("shr".to_string()))
        );
    }
}
