use thiserror::Error;

type RgbArray = [u8; 3];

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("Incomplete input")]
    Incomplete,
    #[error("Invalid input")]
    Invalid,
}

/// Accepts RGB colours of the form ab7c01 (no # at the start)
pub fn parse_rgb(input: &str) -> Result<RgbArray, ParserError> {
    let (r, remaining) = parse_two_digit_hex(input)?;
    let (g, remaining) = parse_two_digit_hex(remaining)?;
    let (b, remaining) = parse_two_digit_hex(remaining)?;
    if remaining.len() > 0 {
        return Err(ParserError::Invalid);
    }
    Ok([r, g, b])
}

fn parse_two_digit_hex(input: &str) -> Result<(u8, &str), ParserError> {
    if input.len() < 2 {
        return Err(ParserError::Incomplete);
    }
    let x = u8::from_str_radix(&input[..2], 16).map_err(|_| ParserError::Invalid)?;
    Ok((x, &input[2..]))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn valid() {
        assert_eq!(parse_rgb("ab0cff").unwrap(), [160 + 11, 12, 255]);
    }
    #[test]
    fn incomplete() {
        assert_eq!(parse_rgb("00d").unwrap_err(), ParserError::Incomplete)
    }
    #[test]
    fn invalid() {
        assert_eq!(parse_rgb("ab0z").unwrap_err(), ParserError::Invalid)
    }
    #[test]
    fn invalid_too_long() {
        assert_eq!(parse_rgb("ab00aabc").unwrap_err(), ParserError::Invalid)
    }
}
