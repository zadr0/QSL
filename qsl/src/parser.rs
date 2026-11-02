//! QSL source parser. Expensive work (ranges, constants, flags) happens at build time.
//! The public `QslSchema`/`QslField` data model is not modified.

use crate::{
    QslField, QslSchema, QslType, FLAG_ALIGNMENT, FLAG_COMPACT, FLAG_FIELD_BORROW, FLAG_INLINE,
    FLAG_INLINE_ALWAYS, FLAG_INLINE_NEVER, FLAG_LIFETIME, FLAG_OPTIONAL_BITMAP, FLAG_OPT_TABLE,
    FLAG_PREDICT_SEQ, FLAG_SAFETY, FLAG_STATE_BITS, STATE_BITS_QUANTUM_SHIFT,
};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QSL at byte {}: {}", self.offset, self.message)
    }
}
impl std::error::Error for ParseError {}

impl ParseError {
    /// Byte offsets are suitable for the existing `qsl_error` source highlighter.
    pub fn span(&self) -> (usize, usize) {
        (self.offset, self.offset.saturating_add(1))
    }
}

pub(crate) struct Cursor<'a> {
    pub(crate) source: &'a str,
    pub(crate) pos: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }
    fn bytes(&self) -> &[u8] {
        self.source.as_bytes()
    }
    pub(crate) fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            offset: self.pos,
            message: message.into(),
        }
    }
    pub(crate) fn skip(&mut self) -> Result<(), ParseError> {
        loop {
            while self.pos < self.bytes().len() && self.bytes()[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }
            let rest = &self.source[self.pos..];
            if rest.starts_with("//") {
                self.pos += rest.find('\n').unwrap_or(rest.len());
            } else if let Some(comment) = rest.strip_prefix("/*") {
                let end = comment
                    .find("*/")
                    .ok_or_else(|| self.error("unclosed comment"))?;
                self.pos += end + 4;
            } else {
                return Ok(());
            }
        }
    }
    pub(crate) fn eat(&mut self, token: &str) -> Result<bool, ParseError> {
        self.skip()?;
        if self.source[self.pos..].starts_with(token) {
            self.pos += token.len();
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub(crate) fn expect(&mut self, token: &str) -> Result<(), ParseError> {
        if self.eat(token)? {
            Ok(())
        } else {
            Err(self.error(format!("expected `{token}`")))
        }
    }
    pub(crate) fn ident(&mut self) -> Result<&'a str, ParseError> {
        self.skip()?;
        let start = self.pos;
        let b = self.source.as_bytes();
        if b.get(start)
            .is_none_or(|c| !c.is_ascii_alphabetic() && *c != b'_')
        {
            return Err(self.error("expected identifier"));
        }
        self.pos += 1;
        while self.pos < b.len() && (b[self.pos].is_ascii_alphanumeric() || b[self.pos] == b'_') {
            self.pos += 1;
        }
        Ok(&self.source[start..self.pos])
    }
    pub(crate) fn eof(&mut self) -> Result<bool, ParseError> {
        self.skip()?;
        Ok(self.pos == self.bytes().len())
    }
    fn number(&mut self) -> Result<i128, ParseError> {
        self.skip()?;
        let b = self.source.as_bytes();
        let start = self.pos;
        let radix = if b.get(start) == Some(&b'0')
            && b.get(start + 1).is_some_and(|x| *x == b'x' || *x == b'X')
        {
            self.pos += 2;
            16
        } else if b.get(start) == Some(&b'0')
            && b.get(start + 1).is_some_and(|x| *x == b'b' || *x == b'B')
        {
            self.pos += 2;
            2
        } else {
            10
        };
        let digit_start = self.pos;
        let mut result = 0i128;
        while let Some(&ch) = b.get(self.pos) {
            if ch == b'_' {
                self.pos += 1;
                continue;
            }
            let digit = match ch {
                b'0'..=b'9' => ch - b'0',
                b'a'..=b'f' => ch - b'a' + 10,
                b'A'..=b'F' => ch - b'A' + 10,
                _ => break,
            };
            if digit as u32 >= radix {
                break;
            }
            result = result
                .checked_mul(radix as i128)
                .and_then(|v| v.checked_add(digit as i128))
                .ok_or_else(|| self.error("integer overflow"))?;
            self.pos += 1;
        }
        if self.pos == digit_start
            || self.source[digit_start..self.pos]
                .bytes()
                .all(|b| b == b'_')
        {
            return Err(self.error("expected numeric literal"));
        }
        Ok(result)
    }
    pub(crate) fn atom(&mut self, defs: &HashMap<String, i128>) -> Result<i128, ParseError> {
        self.skip()?;
        if self.eat("-")? {
            return self
                .atom(defs)?
                .checked_neg()
                .ok_or_else(|| self.error("integer overflow"));
        }
        if self.eat("+")? {
            return self.atom(defs);
        }
        if self.eat("(")? {
            let v = self.expr(defs)?;
            self.expect(")")?;
            return Ok(v);
        }
        if self.eat("!")? {
            let name = self.ident()?;
            return defs
                .get(name)
                .copied()
                .ok_or_else(|| self.error(format!("unknown constant `{name}`")));
        }
        if self
            .bytes()
            .get(self.pos)
            .is_some_and(|v| v.is_ascii_digit())
        {
            return self.number();
        }
        let name = self.ident()?;
        if name == "sizeof" {
            self.expect("(")?;
            let ty = self.ident()?;
            let size = match ty {
                "u8" | "i8" => 1,
                "u16" | "i16" => 2,
                "u32" | "i32" => 4,
                "u64" | "i64" => 8,
                _ => return Err(self.error("unsupported sizeof type")),
            };
            self.expect(")")?;
            return Ok(size);
        }
        defs.get(name)
            .copied()
            .ok_or_else(|| self.error(format!("unknown constant `{name}`")))
    }
    fn term(&mut self, defs: &HashMap<String, i128>) -> Result<i128, ParseError> {
        let mut result = self.atom(defs)?;
        loop {
            let op = if self.eat("*")? {
                0
            } else if self.eat("/")? {
                1
            } else if self.eat("%")? {
                2
            } else {
                return Ok(result);
            };
            let next = self.atom(defs)?;
            result = match op {
                0 => result.checked_mul(next),
                1 => result.checked_div(next),
                _ => result.checked_rem(next),
            }
            .ok_or_else(|| self.error("expression overflow or division by zero"))?;
        }
    }
    pub(crate) fn expr(&mut self, defs: &HashMap<String, i128>) -> Result<i128, ParseError> {
        let mut result = self.term(defs)?;
        loop {
            let op = if self.eat("+")? {
                Some(true)
            } else if self.eat("-")? {
                Some(false)
            } else {
                None
            };
            let Some(add) = op else {
                return Ok(result);
            };
            let next = self.term(defs)?;
            result = if add {
                result.checked_add(next)
            } else {
                result.checked_sub(next)
            }
            .ok_or_else(|| self.error("expression overflow"))?;
        }
    }
}

pub fn parse_qsl(source: &str) -> Result<Vec<QslSchema>, ParseError> {
    parse_qsl_with_constants(source, &HashMap::new())
}

pub fn parse_qsl_with_constants(
    source: &str,
    globals: &HashMap<String, i128>,
) -> Result<Vec<QslSchema>, ParseError> {
    let raw = parse_qsl_raw_with_constants(source, globals)?;
    crate::schema_macro::expand_macros(&raw).map_err(|e| ParseError {
        offset: source.len(),
        message: e.to_string(),
    })
}
pub(crate) fn parse_qsl_raw_with_constants(
    source: &str,
    globals: &HashMap<String, i128>,
) -> Result<Vec<QslSchema>, ParseError> {
    let mut p = Cursor::new(source);
    let mut defs = globals.clone();
    let mut result = Vec::new();
    while !p.eof()? {
        p.expect("&")?;
        let name = p.ident()?.to_owned();
        let mut flags = 0;
        let mut attributes = std::collections::HashSet::new();
        if p.eat("[")? {
            while !p.eat("]")? {
                let attr = p.ident()?;
                if !attributes.insert(attr) {
                    return Err(p.error("duplicate schema attribute"));
                }
                flags |= match attr {
                    "macro" => return Err(p.error("mark the Rust struct with #[qsl::qsl_macro] or #[qsl::schem_macro]; [macro] is not a QSL attribute")),
                    "qps" | "quant" => {
                        if attributes.contains(if attr=="qps"{"quant"}else{"qps"}) { return Err(p.error("qps and quant are aliases; specify only one")); }
                        let explicit=p.eat("=")?;
                        if attr=="quant"&&!explicit { return Err(p.error("quant expects =Q2, =Q4, =Q8 or =Q16")); }
                        let q: usize = if !explicit {16} else {match p.ident()? {
                            "Q2" => 2,
                            "Q4" => 4,
                            "Q8" => 8,
                            "Q16" => 16,
                            _ => return Err(p.error("qps expects Q2, Q4, Q8 or Q16")),
                        }};
                        crate::FLAG_PAYLOAD_QUANT
                            | ((q.trailing_zeros() as usize - 1) << crate::PAYLOAD_QUANT_SHIFT)
                    }
                    "inline" => {
                        if p.eat("=")? {
                            match p.ident()? {
                                "always" => FLAG_INLINE_ALWAYS,
                                "never" => FLAG_INLINE_NEVER,
                                "hint" => FLAG_INLINE,
                                _ => return Err(p.error("inline must be hint, always or never")),
                            }
                        } else {
                            FLAG_INLINE
                        }
                    }
                    "state_bits" => {
                        let mut q: usize = 1;
                        if p.eat("=")? {
                            q = match p.ident()? {
                                "Q1" => 1,
                                "Q2" => 2,
                                "Q4" => 4,
                                "Q8" => 8,
                                "Q16" => 16,
                                _ => {
                                    return Err(p.error("state_bits quantum must be Q1, Q2, Q4, Q8 or Q16"))
                                }
                            };
                        }
                        FLAG_STATE_BITS | ((q.trailing_zeros() as usize) << STATE_BITS_QUANTUM_SHIFT)
                    }
                    "opt_table" => {
                        if p.eat("=")? {
                            match p.ident()? {
                                "legacy" => crate::FLAG_OPT_TABLE_LEGACY,
                                "static" => FLAG_OPT_TABLE,
                                _ => return Err(p.error("opt_table must be static or legacy")),
                            }
                        } else {
                            FLAG_OPT_TABLE
                        }
                    }
                    "predict_seq" => FLAG_PREDICT_SEQ,
                    "compact" => FLAG_COMPACT,
                    "alignment" => FLAG_ALIGNMENT,
                    "lifetime" => FLAG_LIFETIME,
                    "safety" => FLAG_SAFETY,
                    "optionals" => {
                        p.expect("=")?;
                        if p.ident()? != "bitmap" {
                            return Err(p.error("expected optionals=bitmap"));
                        }
                        FLAG_OPTIONAL_BITMAP
                    }
                    _ => return Err(p.error(format!("unknown schema attribute `{attr}`"))),
                };
                if !p.eat(",")? {
                    p.expect("]")?;
                    break;
                }
            }
        }
        p.expect("{")?;
        let mut fields = Vec::new();
        let mut max_tag = 0;
        while !p.eat("}")? {
            if p.eof()? {
                return Err(p.error("unclosed schema body"));
            }
            if p.eat("#")? {
                let key = p.ident()?.to_owned();
                if p.eat("=")? {
                    // Numeric directives are compile-time constants. Other directives
                    // are not silently ignored: unsupported semantics must be explicit.
                    let value = p.expr(&defs)?;
                    defs.insert(key, value);
                } else {
                    return Err(p.error("directive requires a value"));
                }
                p.expect(";")?;
                continue;
            }
            let ty_name = p.ident()?.to_owned();
            let element = if ty_name == "array" || ty_name == "vec" {
                p.expect("<")?;
                let inner_name = p.ident()?;
                let inner = primitive(inner_name)
                    .ok_or_else(|| p.error("unsupported array element type"))?;
                p.expect(">")?;
                Some(inner)
            } else {
                None
            };
            let optional = p.eat("?")?;
            let range = if p.eat("(")? {
                let min = p.expr(&defs)?;
                let max = if p.eat("..")? {
                    Some(p.expr(&defs)?)
                } else {
                    None
                };
                p.expect(")")?;
                Some((min, max))
            } else {
                None
            };
            let field_name = p.ident()?.to_owned();
            // Tags are compiler-owned indices, never syntax-level modifiers.
            let tag = fields.len() + 1;
            let mut default = None;
            let mut decompress = None;
            let mut binary_opt = None;
            while p.eat("|")? {
                let key = p.ident()?;
                match key {
                    "default" => {
                        if default.is_some() {
                            return Err(p.error("duplicate default modifier"));
                        }
                        p.expect("=")?;
                        default = Some(p.expr(&defs)?);
                    }
                    "binary_opt" => {
                        if binary_opt.is_some() {
                            return Err(p.error("duplicate binary_opt modifier"));
                        }
                        p.expect("=")?;
                        let mode = p.ident()?;
                        if !matches!(mode, "true" | "false" | "inline" | "guard" | "view") {
                            return Err(
                                p.error("binary_opt must be true, false, inline, guard or view")
                            );
                        }
                        binary_opt = Some(mode);
                    }
                    "decompress" => {
                        if decompress.is_some() {
                            return Err(p.error("duplicate decompress modifier"));
                        }
                        p.expect("=")?;
                        decompress = Some(match p.ident()? {
                            "true" => true,
                            "false" => false,
                            _ => return Err(p.error("decompress must be true or false")),
                        });
                    }
                    _ => return Err(p.error(format!("unsupported field modifier `{key}`"))),
                }
                p.expect("|")?;
            }
            p.expect(";")?;
            if !(1..=64).contains(&tag) {
                return Err(p.error("QSL tag must be in 1..=64"));
            }
            max_tag = max_tag.max(tag);
            let (mut data, mut bits) = build_type(&ty_name, element, range, None, &p)?;
            if let Some(enabled) = decompress {
                if ty_name != "string" && ty_name != "str" && ty_name != "String" {
                    return Err(p.error("decompress is only valid on string fields"));
                }
                if enabled {
                    let (min, max) = match data {
                        QslType::RangedString { min, max } => (min, max),
                        QslType::FixedString(n) => (n, n),
                        _ => {
                            return Err(
                                p.error("decompress=true requires a finite UTF-8 byte range")
                            )
                        }
                    };
                    if max > u16::MAX as usize {
                        return Err(p.error("inline strings support at most 65535 decoded bytes"));
                    }
                    data = if matches!(data, QslType::FixedString(_)) {
                        QslType::FixedCompressedString(max)
                    } else {
                        QslType::CompressedString { min, max }
                    };
                    bits = 0;
                }
            }
            if let Some(mode) = binary_opt {
                if ty_name != "string" && ty_name != "str" && ty_name != "String" {
                    return Err(p.error("binary_opt is only valid on string/str fields"));
                }
                if matches!(mode, "true" | "guard" | "view")
                    && decompress == Some(true)
                    && !matches!(data, QslType::FixedCompressedString(_))
                {
                    return Err(p.error(
                        "compressed bytes cannot borrow decoded storage; use binary_opt=inline",
                    ));
                }
                if matches!(mode, "true" | "guard" | "view")
                    && !matches!(
                        data,
                        QslType::FixedString(_) | QslType::FixedCompressedString(_)
                    )
                {
                    let (min, max) = match data {
                        QslType::String(min) => (min, None),
                        QslType::RangedString { min, max } => (min, Some(max)),
                        _ => return Err(p.error("invalid binary string")),
                    };
                    data = if mode == "guard" {
                        QslType::CopyGuardString { min, max }
                    } else if mode == "view" {
                        QslType::GuardString { min, max }
                    } else {
                        QslType::BinaryString { min, max }
                    };
                    if mode != "guard" {
                        flags |= FLAG_FIELD_BORROW;
                    }
                } else if mode == "inline" && decompress != Some(true) {
                    data = match data {
                        QslType::RangedString { min, max } if max <= 65535 => {
                            QslType::InlineRawString { min, max }
                        }
                        QslType::FixedString(n) => QslType::FixedString(n),
                        _ => {
                            return Err(
                                p.error("inline string requires a finite byte range up to 65535")
                            )
                        }
                    };
                }
            }
            fields.push(QslField {
                optional,
                bits,
                data,
                name: field_name,
                tag,
                default,
            });
        }
        let schema = QslSchema {
            flags,
            fields,
            name,
            max_tag,
        };
        if result.iter().any(|s: &QslSchema| s.name == schema.name) {
            return Err(p.error(format!("duplicate schema `{}`", schema.name)));
        }
        result.push(schema);
    }
    Ok(result)
}

fn as_usize(v: i128, p: &Cursor<'_>) -> Result<usize, ParseError> {
    usize::try_from(v).map_err(|_| p.error("expected nonnegative usize value"))
}

fn primitive(name: &str) -> Option<QslType> {
    let ty = match name {
        "u8" => QslType::Byte,
        "u16" => QslType::Integer {
            bits: 16,
            signed: false,
            min: 0,
        },
        "u32" => QslType::Integer {
            bits: 32,
            signed: false,
            min: 0,
        },
        "u64" => QslType::Integer {
            bits: 64,
            signed: false,
            min: 0,
        },
        "i16" => QslType::Integer {
            bits: 16,
            signed: true,
            min: 0,
        },
        "i32" => QslType::Integer {
            bits: 32,
            signed: true,
            min: 0,
        },
        "i64" => QslType::Integer {
            bits: 64,
            signed: true,
            min: 0,
        },
        _ => return None,
    };
    Some(ty)
}

fn build_type(
    name: &str,
    element: Option<QslType>,
    range: Option<(i128, Option<i128>)>,
    override_bits: Option<usize>,
    p: &Cursor<'_>,
) -> Result<(QslType, usize), ParseError> {
    let result = match name {
        "u8" | "u16" | "u32" | "u64" | "i16" | "i32" | "i64" => {
            let standard = primitive(name).unwrap();
            let width = standard
                .fixed_wire_size()
                .map_err(|_| p.error("invalid integer type"))?
                .unwrap()
                * 8;
            if let Some((min, max)) = range {
                let max = max.unwrap_or(min);
                let bits =
                    QslType::rank_bits(min, max).map_err(|_| p.error("invalid numeric range"))?;
                let signed = name.starts_with('i');
                let low = if signed { -(1i128 << (width - 1)) } else { 0 };
                let high = if signed {
                    (1i128 << (width - 1)) - 1
                } else {
                    (1i128 << width) - 1
                };
                if min < low || max > high {
                    return Err(p.error("numeric range outside type bounds"));
                }
                (
                    QslType::RangedInteger {
                        bits: width as u8,
                        signed,
                        min,
                        max,
                    },
                    bits,
                )
            } else {
                (standard, width)
            }
        }
        "string" | "str" | "String" => match range {
            None => (QslType::String(0), 32),
            Some((min, None)) => (QslType::FixedString(as_usize(min, p)?), 0),
            Some((min, Some(max))) => {
                let min = as_usize(min, p)?;
                let max = as_usize(max, p)?;
                let bits = QslType::rank_bits(min as i128, max as i128)
                    .map_err(|_| p.error("invalid string range"))?;
                if min == max {
                    (QslType::FixedString(min), 0)
                } else {
                    (QslType::RangedString { min, max }, bits)
                }
            }
        },
        "array" | "vec" => {
            let elem = Box::new(element.ok_or_else(|| p.error("array needs element type"))?);
            match range {
                None => (QslType::Array(elem, 0), 32),
                Some((min, None)) => (QslType::FixedArray(elem, as_usize(min, p)?), 0),
                Some((min, Some(max))) => {
                    let min = as_usize(min, p)?;
                    let max = as_usize(max, p)?;
                    let bits = QslType::rank_bits(min as i128, max as i128)
                        .map_err(|_| p.error("invalid array range"))?;
                    (
                        QslType::RangedArray {
                            element: elem,
                            min,
                            max,
                        },
                        bits,
                    )
                }
            }
        }
        "flags" => {
            let (size, rest) = range.ok_or_else(|| p.error("flags requires (bit_count)"))?;
            if rest.is_some() {
                return Err(p.error("flags requires fixed bit count"));
            }
            let size = as_usize(size, p)?;
            (QslType::Flags(size), size)
        }
        "padding" => {
            let (size, rest) = range.ok_or_else(|| p.error("padding requires (byte_count)"))?;
            if rest.is_some() {
                return Err(p.error("padding requires fixed byte count"));
            }
            (QslType::Padding(as_usize(size, p)?), 0)
        }
        _ => {
            if range.is_some() {
                return Err(p.error("custom type with ranges is not supported"));
            }
            (
                QslType::Custom {
                    rust_type: name.to_owned(),
                    fixed_wire_size: None,
                },
                0,
            )
        }
    };
    if let Some(bits) = override_bits {
        if bits != result.1 {
            return Err(p.error("bits modifier disagrees with computed range width"));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compressed_strings_require_finite_byte_bounds_and_boolean_attribute() {
        let schemas = parse_qsl("&User { string(1..100) name|decompress=true|; }").unwrap();
        assert!(matches!(
            schemas[0].fields[0].data,
            QslType::CompressedString { min: 1, max: 100 }
        ));
        let schemas = parse_qsl("&User { string(1..100) name|decompress=false|; }").unwrap();
        assert!(matches!(
            schemas[0].fields[0].data,
            QslType::RangedString { min: 1, max: 100 }
        ));
        for source in [
            "&U {string name|decompress=true|;}",
            "&U {string(0..65536) name|decompress=true|;}",
            "&U {string(1..100) name|decompress=1|;}",
            "&U {u8 name|decompress=true|;}",
            "&U {string(1..100) name|decompress=true||decompress=false|;}",
        ] {
            assert!(parse_qsl(source).is_err(), "{source}");
        }
    }

    #[test]
    fn qsl_extension_form_and_constant_folding() {
        let schemas = parse_qsl(
            "&User[compact,optionals=bitmap]{\n\
           #BASE=sizeof(u64)+2;\n\
           u32(!BASE..17) id |default=17|;\n\
           string? name;\n}",
        )
        .unwrap();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].fields[0].bits, 3);
        assert_eq!(schemas[0].fields[0].default, Some(17));
        assert!(schemas[0].fields[1].optional);
        assert_eq!(
            schemas[0].flags & FLAG_OPTIONAL_BITMAP,
            FLAG_OPTIONAL_BITMAP
        );
    }

    #[test]
    fn negative_range_and_multischema() {
        let schemas = parse_qsl("&A[compact]{i16(-5..10) delta;} &B{u64 id;}").unwrap();
        assert_eq!(schemas[0].fields[0].bits, 4);
        assert_eq!(schemas[1].fields[0].bits, 64);
    }

    #[test]
    fn rejects_invalid_tag_and_missing_bracket() {
        assert!(parse_qsl("&A{u8? x |tag=9|;}").is_err());
        assert!(parse_qsl("&A{u8? x |bits=3|;}").is_err());
        assert!(parse_qsl("&A{u8 x |default=999|;}").is_err());
        assert!(parse_qsl("&A{u8(3..5) x |default=2|;}").is_err());
        assert!(parse_qsl("&A{u8 x |default=1| |default=2|;}").is_err());
        assert!(parse_qsl("&A{string x |default=2|;}").is_err());
        assert!(parse_qsl("&A{u8 x;").is_err());
    }

    #[test]
    fn assigns_tags_in_field_order_and_preserves_defaults() {
        let schemas = parse_qsl("&A { u32 id; u8? first |default=7|; u16? second; }").unwrap();
        let fields = &schemas[0].fields;
        assert_eq!(
            fields.iter().map(|f| f.tag).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(schemas[0].max_tag, 3);
        assert_eq!(fields[1].default, Some(7));
    }
}

#[cfg(test)]
mod lowlevel_tests {
    use super::*;
    #[test]
    fn binary_and_sequence_modifiers_are_validated() {
        for source in [
            "&A{u32 x|binary_opt=true|;}",
            "&A{str x|binary_opt=inline|;}",
            "&A{str(0..100) x|binary_opt=view||decompress=true|;}",
            "&A[predict_seq]{u64 id;u16? x;u8 required;}",
            "&A[predict_seq]{u16? x;u16? y;}",
            "&A[opt_table,compact]{u16? x;u16? y;}",
            "&A[opt_table,optionals=bitmap]{u16? x;u16? y;}",
            "&A[predict_seq]{str?(0) x;}",
        ] {
            assert!(parse_qsl(source).is_err(), "{source}");
        }
        let a = parse_qsl("&A{str(10) x|binary_opt=view|;}").unwrap();
        assert_eq!(a[0].fields[0].data.rust_type(false).unwrap(), "[u8; 10]");
        assert!(!a[0].lifetime());
        let a = parse_qsl("&A{str x|binary_opt=view|;}").unwrap();
        assert!(a[0].lifetime());
        assert!(a[0].fields[0]
            .data
            .rust_type(true)
            .unwrap()
            .contains("Utf8Guard"));
    }
}

#[cfg(test)]
mod fixed_utf8_tests {
    use super::*;
    #[test]
    fn fixed_byte_representation_includes_equal_bounds_and_guard_modifiers() {
        for source in [
            "&A{str(10..10) x;}",
            "&A{str(10) x|binary_opt=view||decompress=true|;}",
        ] {
            let schema = parse_qsl(source).unwrap();
            assert_eq!(
                schema[0].fields[0].data.rust_type(false).unwrap(),
                "[u8; 10]"
            );
        }
        assert!(parse_qsl("&A[predict_seq]{str?(0) x|decompress=true|;}").is_err());
    }
}

#[cfg(test)]
mod copy_guard_tests {
    use super::*;
    #[test]
    fn guard_is_owned_and_view_is_explicitly_borrowed() {
        for spec in ["str", "str(0..4096)"] {
            let copy = parse_qsl(&format!("&A{{{spec} x|binary_opt=guard|;}}")).unwrap();
            assert!(!copy[0].lifetime());
            assert_eq!(
                copy[0].fields[0].data.rust_type(false).unwrap(),
                "::qsl::Utf8CopyGuard"
            );
            let view = parse_qsl(&format!("&A{{{spec} x|binary_opt=view|;}}")).unwrap();
            assert!(view[0].lifetime());
        }
        let fixed = parse_qsl("&A{str(10) x|binary_opt=guard|;}").unwrap();
        assert_eq!(
            fixed[0].fields[0].data.rust_type(false).unwrap(),
            "[u8; 10]"
        );
    }
}
