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

/// Validates that `x` is a non-negative whole number representable as a
/// `u64`, erroring with `DomainError(name)` otherwise.
fn to_u64(x: f64, name: &str) -> CalcResult<u64> {
    if x < 0.0 || x.fract() != 0.0 || x > u64::MAX as f64 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(x as u64)
}

/// Population count: the number of `1` bits in `n`'s binary representation.
pub fn popcount(n: f64) -> CalcResult<f64> {
    Ok(to_u64(n, "popcount")?.count_ones() as f64)
}

/// Bit length: the number of bits needed to represent `n` (`bitlen(0) == 0`,
/// `bitlen(1) == 1`, `bitlen(255) == 8`, `bitlen(256) == 9`).
pub fn bitlen(n: f64) -> CalcResult<f64> {
    let n = to_u64(n, "bitlen")?;
    Ok((64 - n.leading_zeros()) as f64)
}

/// Reverses the lowest `width` bits of `n` (bits beyond `width` are ignored
/// and left as `0` in the result). `width` must be in `[1, 64]`.
pub fn bitreverse(n: f64, width: f64) -> CalcResult<f64> {
    let n = to_u64(n, "bitreverse")?;
    let width = to_u64(width, "bitreverse")?;
    if width == 0 || width > 64 || (width < 64 && n >= (1u64 << width)) {
        return Err(CalcError::DomainError("bitreverse".to_string()));
    }
    let reversed = n.reverse_bits();
    Ok((reversed >> (64 - width)) as f64)
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

    #[test]
    fn popcount_counts_set_bits() {
        assert_eq!(popcount(0.0).unwrap(), 0.0);
        assert_eq!(popcount(255.0).unwrap(), 8.0);
        assert_eq!(popcount(1024.0).unwrap(), 1.0);
    }

    #[test]
    fn popcount_domain_error() {
        assert_eq!(
            popcount(-1.0),
            Err(CalcError::DomainError("popcount".to_string()))
        );
        assert_eq!(
            popcount(1.5),
            Err(CalcError::DomainError("popcount".to_string()))
        );
    }

    #[test]
    fn bitlen_matches_known_values() {
        assert_eq!(bitlen(0.0).unwrap(), 0.0);
        assert_eq!(bitlen(1.0).unwrap(), 1.0);
        assert_eq!(bitlen(255.0).unwrap(), 8.0);
        assert_eq!(bitlen(256.0).unwrap(), 9.0);
    }

    #[test]
    fn bitreverse_matches_known_values() {
        // 0b0001 reversed in 4 bits is 0b1000 = 8.
        assert_eq!(bitreverse(1.0, 4.0).unwrap(), 8.0);
        // 0b11000000 reversed in 8 bits is 0b00000011 = 3.
        assert_eq!(bitreverse(0xc0 as f64, 8.0).unwrap(), 3.0);
        assert_eq!(bitreverse(0.0, 8.0).unwrap(), 0.0);
    }

    #[test]
    fn bitreverse_domain_errors() {
        assert_eq!(
            bitreverse(1.0, 0.0),
            Err(CalcError::DomainError("bitreverse".to_string()))
        );
        assert_eq!(
            bitreverse(1.0, 65.0),
            Err(CalcError::DomainError("bitreverse".to_string()))
        );
        // n doesn't fit in `width` bits.
        assert_eq!(
            bitreverse(256.0, 8.0),
            Err(CalcError::DomainError("bitreverse".to_string()))
        );
    }
}
