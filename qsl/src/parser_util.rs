//! Overflow-aware primitives for the original parser's sign-magnitude input.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseNum(pub usize, pub bool);

/// Inclusive range width: ceil(log2(MAX-MIN+1)), including singleton = 0.
pub fn rank_bits(min: ParseNum, max: ParseNum) -> Option<usize> {
    let min_negative = min.1 && min.0 != 0;
    let max_negative = max.1 && max.0 != 0;
    let span = match (min_negative, max_negative) {
        (false, false) => max.0.checked_sub(min.0)?,
        (true, true) => min.0.checked_sub(max.0)?,
        (true, false) => min.0.checked_add(max.0)?,
        (false, true) => return None,
    };
    Some((usize::BITS - span.leading_zeros()) as usize)
}

pub fn parse_bytes(bytes: &[u8]) -> Option<usize> {
    let mut result = 0usize;
    if bytes.is_empty() {
        return None;
    }
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as usize)?;
    }
    Some(result)
}

pub fn parse_hex_bytes(bytes: &[u8]) -> Result<usize, &'static str> {
    let mut result = 0usize;
    if bytes.is_empty() {
        return Err("Empty hex literal");
    }
    for &b in bytes {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as usize,
            b'a'..=b'f' => (b - b'a' + 10) as usize,
            b'A'..=b'F' => (b - b'A' + 10) as usize,
            _ => return Err("Invalid hex digit"),
        };
        result = result
            .checked_mul(16)
            .and_then(|v| v.checked_add(digit))
            .ok_or("Hex overflow")?;
    }
    Ok(result)
}

/// Linear scanner for `&Name{...}` blocks. Cursor and ranges are always
/// absolute indexes into the same source (unlike the original offset mixup).
pub fn scan_schema_blocks(source: &[u8]) -> Result<Vec<&[u8]>, &'static str> {
    let mut result = Vec::new();
    let mut cursor = 0usize;
    while cursor < source.len() {
        let Some(start) = source[cursor..].iter().position(|&ch| ch == b'&') else {
            break;
        };
        let begin = cursor + start + 1;
        let close = source[begin..]
            .iter()
            .position(|&ch| ch == b'}')
            .ok_or("Unclosed schema")?;
        let end = begin + close;
        result.push(&source[begin..end]);
        cursor = end + 1;
    }
    Ok(result)
}

pub fn parse_schema_flags(source: &[u8]) -> Result<usize, &'static str> {
    use crate::types::{
        FLAG_ALIGNMENT, FLAG_COMPACT, FLAG_LIFETIME, FLAG_OPTIONAL_BITMAP, FLAG_SAFETY,
    };
    let mut flags = 0;
    for raw in source.split(|&b| b == b',') {
        let word = raw.trim_ascii();
        if word.is_empty() {
            continue;
        }
        flags |= match word {
            b"compact" => FLAG_COMPACT,
            b"safety" => FLAG_SAFETY,
            b"alignment" => FLAG_ALIGNMENT,
            b"lifetime" => FLAG_LIFETIME,
            b"optionals=bitmap" => FLAG_OPTIONAL_BITMAP,
            _ => return Err("Unknown QSL schema flag"),
        };
    }
    Ok(flags)
}

pub fn parse_primitive_type(token: &[u8]) -> Option<crate::compat::ParserQslType> {
    use crate::compat::ParserQslType as T;
    Some(match token {
        b"u8" => T::Byte,
        b"u16" => T::Number16(false, 0),
        b"u32" => T::Number32(false, 0),
        b"u64" => T::Number64(false, 0),
        b"i16" => T::Number16(true, 0),
        b"i32" => T::Number32(true, 0),
        b"i64" => T::Number64(true, 0),
        b"string" | b"str" => T::String(0),
        _ => return None,
    })
}

#[inline]
pub fn encode_signed_minimum(value: ParseNum) -> usize {
    if value.1 {
        0usize.wrapping_sub(value.0)
    } else {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inclusive_range_bits() {
        assert_eq!(rank_bits(ParseNum(10, false), ParseNum(17, false)), Some(3));
        assert_eq!(rank_bits(ParseNum(5, true), ParseNum(10, false)), Some(4));
        assert_eq!(rank_bits(ParseNum(5, true), ParseNum(3, true)), Some(2));
        assert_eq!(rank_bits(ParseNum(0, false), ParseNum(0, false)), Some(0));
        assert_eq!(rank_bits(ParseNum(3, false), ParseNum(5, true)), None);
    }
    #[test]
    fn overflow_checks() {
        assert_eq!(parse_bytes(b"184467440737095516160000"), None);
        assert!(parse_hex_bytes(b"fffffffffffffffff").is_err());
    }
    #[test]
    fn block_offsets_are_absolute() {
        let blocks = scan_schema_blocks(b"header &A{u16 x;} other &B{u32 y;}").unwrap();
        assert_eq!(blocks, vec![&b"A{u16 x;"[..], &b"B{u32 y;"[..]]);
    }
    #[test]
    fn parser_no_longer_maps_u32_to_u64() {
        use crate::compat::ParserQslType;
        assert!(matches!(
            parse_primitive_type(b"u32"),
            Some(ParserQslType::Number32(false, 0))
        ));
        assert!(matches!(
            parse_primitive_type(b"u8"),
            Some(ParserQslType::Byte)
        ));
    }
}
