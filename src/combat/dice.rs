use rand::Rng;
use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiceParseError {
    InvalidFormat(String),
    ParseInt(ParseIntError),
}

impl From<ParseIntError> for DiceParseError {
    fn from(e: ParseIntError) -> Self {
        DiceParseError::ParseInt(e)
    }
}

/// Roll a single D6, returning a value from 1 to 6.
pub fn roll_d6() -> u8 {
    rand::thread_rng().gen_range(1..=6)
}

/// Roll multiple D6 dice, returning a Vec of results.
pub fn roll_d6_batch(count: usize) -> Vec<u8> {
    (0..count).map(|_| roll_d6()).collect()
}

/// Roll a single D3, returning a value from 1 to 3.
pub fn roll_d3() -> u8 {
    rand::thread_rng().gen_range(1..=3)
}

/// Parse and resolve a dice string.
/// Supported formats:
/// - Fixed number: "5", "10"
/// - D6: "D6"
/// - 2D6: "2D6"
/// - D3: "D3"
/// - 2D3: "2D3"
/// - Modified: "D6+2", "D3-1", "2D6+3"
pub fn parse_dice_string(input: &str) -> Result<u8, DiceParseError> {
    let input = input.trim().to_uppercase();

    // Try simple number first
    if let Ok(n) = input.parse::<u8>() {
        return Ok(n);
    }

    // Check for D6 or D3 pattern
    if !input.contains('D') {
        return Err(DiceParseError::InvalidFormat(input));
    }

    // Split by plus or minus to handle modifiers
    // Example: "2D6+3" -> dice_part = "2D6", modifier = "+3"
    let (dice_part, modifier) = if let Some(idx) = input.find('+') {
        (&input[..idx], Some(&input[idx..]))
    } else if let Some(idx) = input.find('-') {
        (&input[..idx], Some(&input[idx..]))
    } else {
        (input.as_str(), None)
    };

    // Parse dice part like "2D6" or "D6"
    let parts: Vec<&str> = dice_part.split('D').collect();
    if parts.len() != 2 {
        return Err(DiceParseError::InvalidFormat(input));
    }

    let count = if parts[0].is_empty() {
        1
    } else {
        parts[0].parse::<u8>()?
    };

    let sides = parts[1].parse::<u8>()?;

    if sides != 3 && sides != 6 {
        return Err(DiceParseError::InvalidFormat(format!(
            "Unsupported dice sides: {}",
            sides
        )));
    }

    // Use u16 for accumulation to prevent overflow, then clamp to u8
    let mut accum: u16 = 0;
    for _ in 0..count {
        accum += if sides == 6 { roll_d6() } else { roll_d3() } as u16;
    }
    let mut total = accum.min(u8::MAX as u16) as u8;

    // Apply modifier
    if let Some(mod_str) = modifier {
        if let Some(stripped) = mod_str.strip_prefix('+') {
            let mod_val = stripped.parse::<i16>()?;
            total = (total as i16 + mod_val).max(0) as u8;
        } else if let Some(stripped) = mod_str.strip_prefix('-') {
            let mod_val = stripped.parse::<i16>()?;
            total = (total as i16 - mod_val).max(0) as u8;
        }
    }

    Ok(total)
}

/// Resolve a dice string, returning a guaranteed non-zero value if possible.
#[allow(dead_code)]
pub fn resolve_dice_string(input: &str) -> u8 {
    parse_dice_string(input).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_d6_returns_values_between_1_and_6() {
        for _ in 0..1000 {
            let result = roll_d6();
            assert!(result >= 1 && result <= 6);
        }
    }

    #[test]
    fn roll_multiple_d6_returns_correct_count() {
        let results = roll_d6_batch(10);
        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|&r| r >= 1 && r <= 6));
    }

    #[test]
    fn parse_attacks_fixed() {
        assert_eq!(parse_dice_string("5"), Ok(5));
        assert_eq!(parse_dice_string("10"), Ok(10));
    }

    #[test]
    fn parse_attacks_d6() {
        let result = parse_dice_string("D6").unwrap();
        assert!(result >= 1 && result <= 6);
    }

    #[test]
    fn parse_attacks_2d6() {
        let result = parse_dice_string("2D6").unwrap();
        assert!(result >= 2 && result <= 12);
    }

    #[test]
    fn parse_damage_with_modifier() {
        // D3+2 should return 3-5
        let result = parse_dice_string("D3+2").unwrap();
        assert!(result >= 3 && result <= 5);
    }

    #[test]
    fn parse_invalid_string_returns_error() {
        assert!(parse_dice_string("XYZ").is_err());
        assert!(parse_dice_string("D8").is_err());
        assert!(parse_dice_string("").is_err());
    }
}
