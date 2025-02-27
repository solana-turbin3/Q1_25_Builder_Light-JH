use std::str::FromStr;

pub fn decimal_to_u64(value: &str, decimals: u8) -> Result<u64, &'static str> {
    // Split the price into integer and fractional parts
    let parts: Vec<&str> = value.split('.').collect();

    let integer_part = parts[0]; // Whole number part
    let fractional_part = if parts.len() > 1 { parts[1] } else { "" }; // Fractional part (optional)

    let fractional_len = fractional_part.len() as u8;

    // Check if the number has more decimals than allowed
    if fractional_len > decimals {
        return Err("Too many decimal places");
    }

    // Convert integer part to u64
    let mut value = u64::from_str(integer_part).map_err(|_| "Invalid integer part")?;

    // Handle the fractional part
    if fractional_len > 0 {
        let fractional_value =
            u64::from_str(fractional_part).map_err(|_| "Invalid fractional part")?;

        // Scale fractional part to match required decimals
        let missing_zeros = decimals - fractional_len;
        value = value * 10_u64.pow(decimals as u32)
            + fractional_value * 10_u64.pow(missing_zeros as u32);
    } else {
        // Just scale the integer part
        value *= 10_u64.pow(decimals as u32);
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_to_u64() {
        assert_eq!(decimal_to_u64("12.34", 2), Ok(1234));
        assert_eq!(decimal_to_u64("0.5", 3), Ok(500));
        assert_eq!(decimal_to_u64("100", 5), Ok(10000000));
        assert_eq!(decimal_to_u64("45.678", 3), Ok(45678));
        assert!(decimal_to_u64("3.1415", 2).is_err());
    }
}
