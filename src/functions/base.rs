//! Base-conversion functions: format a non-negative integer as a
//! hexadecimal/octal/binary string. Pairs with the lexer's `0x`/`0o`/`0b`
//! literal support for parsing hex/octal/binary numbers back in (see
//! `lexer.rs`).

use crate::error::{CalcError, CalcResult};

/// Parses `x` as a non-negative whole number representable in a `u64`,
/// erroring with `DomainError(name)` otherwise.
fn non_negative_u64(name: &str, x: f64) -> CalcResult<u64> {
    if x < 0.0 || x.fract() != 0.0 || x > u64::MAX as f64 {
        return Err(CalcError::DomainError(name.to_string()));
    }
    Ok(x as u64)
}

/// Formats `x` as a `0x`-prefixed lowercase hexadecimal string, e.g.
/// `hex(255) = "0xff"`.
pub fn hex(x: f64) -> CalcResult<String> {
    Ok(format!("0x{:x}", non_negative_u64("hex", x)?))
}

/// Formats `x` as a `0o`-prefixed octal string, e.g. `oct(8) = "0o10"`.
pub fn oct(x: f64) -> CalcResult<String> {
    Ok(format!("0o{:o}", non_negative_u64("oct", x)?))
}

/// Formats `x` as a `0b`-prefixed binary string, e.g. `bin(10) = "0b1010"`.
pub fn bin(x: f64) -> CalcResult<String> {
    Ok(format!("0b{:b}", non_negative_u64("bin", x)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_matches_known_values() {
        assert_eq!(hex(255.0).unwrap(), "0xff");
        assert_eq!(hex(0.0).unwrap(), "0x0");
        assert_eq!(hex(16.0).unwrap(), "0x10");
    }

    #[test]
    fn oct_matches_known_values() {
        assert_eq!(oct(8.0).unwrap(), "0o10");
        assert_eq!(oct(0.0).unwrap(), "0o0");
        assert_eq!(oct(511.0).unwrap(), "0o777");
    }

    #[test]
    fn bin_matches_known_values() {
        assert_eq!(bin(10.0).unwrap(), "0b1010");
        assert_eq!(bin(0.0).unwrap(), "0b0");
        assert_eq!(bin(255.0).unwrap(), "0b11111111");
    }

    #[test]
    fn domain_errors_for_negative_or_fractional_input() {
        assert_eq!(hex(-1.0), Err(CalcError::DomainError("hex".to_string())));
        assert_eq!(hex(1.5), Err(CalcError::DomainError("hex".to_string())));
        assert_eq!(oct(-1.0), Err(CalcError::DomainError("oct".to_string())));
        assert_eq!(bin(-1.0), Err(CalcError::DomainError("bin".to_string())));
    }
}
