//! QSL source parser. Expensive work (ranges, constants, flags) happens at build time.
//! The public `QslSchema`/`QslField` data model is not modified.

pub(crate) use crate::lexer::Lexer as Cursor;
pub use crate::lexer::ParseError;
use crate::{
    QslField, QslSchema, QslType, FLAG_ALIGNMENT, FLAG_COMPACT, FLAG_FIELD_BORROW, FLAG_INLINE,
    FLAG_INLINE_ALWAYS, FLAG_INLINE_NEVER, FLAG_LIFETIME, FLAG_OPTIONAL_BITMAP, FLAG_OPT_TABLE,
    FLAG_PREDICT_SEQ, FLAG_SAFETY, FLAG_STATE_BITS, STATE_BITS_QUANTUM_SHIFT,
};
use std::collections::HashMap;

impl<'a> Cursor<'a> {
    pub(crate) fn atom(&mut self, defs: &HashMap<String, i128>) -> Result<i128, ParseError> {
        self.skip_trivia()?;
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
    let schemas = crate::schema_macro::expand_macros(&raw).map_err(|e| ParseError {
        offset: source.len(),
        message: e.to_string(),
    })?;
    crate::regions::lower_regions(&schemas).map_err(|e| ParseError {
        offset: source.len(),
        message: e.to_string(),
    })
}
#[doc(hidden)]
pub fn parse_qsl_raw_with_constants(
    source: &str,
    globals: &HashMap<String, i128>,
) -> Result<Vec<QslSchema>, ParseError> {
    parse_qsl_raw_with_primitives(source, globals, &Default::default())
}
#[doc(hidden)]
pub fn parse_qsl_raw_with_primitives(
    source: &str,
    globals: &HashMap<String, i128>,
    primitives: &std::collections::BTreeMap<String, crate::PrimitiveRepr>,
) -> Result<Vec<QslSchema>, ParseError> {
    let mut p = Cursor::new(source);
    let mut defs = globals.clone();
    let mut result = Vec::new();
    while !p.eof()? {
        p.expect("&")?;
        let name = p.ident()?.to_owned();
        let mut flags = 0;
        let mut options = crate::modernization::SchemaOptions::default();
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
                    "aggressive" => {options.aggressive=true;0},
                    "transmutate" => {options.transmutate=true;0},
                    "loyal_cache" => {options.loyal_cache=true;0},
                    "slots" => {p.expect("=")?;let n=p.expr(&defs)?;if !(1..=64).contains(&n){return Err(p.error("slots must be 1..64"));}options.slots=Some(n as usize);0},
                    "cache" => {p.expect("=")?;let n=p.expr(&defs)?;if n<0 || n>isize::MAX as i128{return Err(p.error("cache must be 0..isize::MAX bytes"));}options.cache_limit=Some(n as usize);0},
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
        let mut regions = Vec::new();
        while !p.eat("}")? {
            if p.eof()? {
                return Err(p.error("unclosed schema body"));
            }
            if p.eat("#")? {
                let key = p.ident()?.to_owned();
                if key == "region" {
                    p.expect("(")?;
                    if !matches!(
                        p.ident()?,
                        "aggressive_transmutate" | "aggresive_transmutate"
                    ) {
                        return Err(p.error("region expects aggressive_transmutate"));
                    }
                    p.expect("=")?;
                    p.expect("\"")?;
                    let ty = p.ident()?.to_owned();
                    p.expect("\"")?;
                    p.expect(")")?;
                    p.expect(";")?;
                    regions.push((ty, fields.len()));
                    continue;
                }
                if key == "end_region" {
                    let (ty, start) = regions
                        .pop()
                        .ok_or_else(|| p.error("end_region has no matching region"))?;
                    if start == fields.len() {
                        return Err(p.error("native region must not be empty"));
                    }
                    options.regions.push(crate::modernization::NativeRegion {
                        alignment: None,
                        external: false,
                        field_prefix: String::new(),
                        rust_type: ty,
                        start,
                        end: fields.len(),
                    });
                    p.expect(";")?;
                    continue;
                }
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
                    .or_else(|| {
                        primitives.get(inner_name).map(|repr| QslType::Custom {
                            rust_type: inner_name.to_owned(),
                            fixed_wire_size: Some(repr.bits as usize / 8),
                        })
                    })
                    .ok_or_else(|| {
                        p.error(
                            "array elements must be built-in or explicitly registered primitives",
                        )
                    })?;
                p.expect(">")?;
                Some(inner)
            } else {
                None
            };
            let optional = p.eat("?")?;
            let range = if p.eat("(")? {
                let min = p.expr(&defs)?;
                let max = if p.eat("..")? {
                    p.skip_trivia()?;
                    if p.source[p.pos..].starts_with(')') {
                        let top = primitive(&ty_name)
                            .and_then(|t| t.fixed_wire_size().ok().flatten())
                            .map(|n| {
                                let bits = n * 8;
                                if ty_name.starts_with('i') {
                                    (1i128 << (bits - 1)) - 1
                                } else {
                                    (1i128 << bits) - 1
                                }
                            })
                            .unwrap_or(u32::MAX as i128);
                        Some(top)
                    } else {
                        Some(p.expr(&defs)?)
                    }
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
                        if !matches!(mode, "false" | "inline" | "view") {
                            return Err(
                                p.error("binary_opt must be false (String), inline (InlineString) or view (&str); Guard and byte-string modes are unsupported")
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
            if tag == 0
                || tag
                    > if flags & FLAG_OPTIONAL_BITMAP != 0 {
                        65535
                    } else {
                        64
                    }
            {
                return Err(p.error("QSL tag exceeds the selected codec capacity"));
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
                if matches!(mode, "view")
                    && decompress == Some(true)
                    && !matches!(data, QslType::FixedCompressedString(_))
                {
                    return Err(p.error(
                        "compressed bytes cannot borrow decoded storage; use binary_opt=inline",
                    ));
                }
                if matches!(mode, "view")
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
                    data = QslType::BorrowedString { min, max };
                    flags |= FLAG_FIELD_BORROW;
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
                storage: Default::default(),
                optional,
                bits,
                data,
                name: field_name,
                tag,
                default,
            });
        }
        if !regions.is_empty() {
            return Err(p.error("unclosed native region"));
        }
        let mut schema = QslSchema {
            options,
            flags,
            fields,
            name,
            max_tag,
        };
        if result.iter().any(|s: &QslSchema| s.name == schema.name) {
            return Err(p.error(format!("duplicate schema `{}`", schema.name)));
        }
        crate::apply_primitive_bindings(&mut schema, primitives)
            .map_err(|_| p.error("invalid registered primitive descriptor"))?;
        crate::modernization::validate_options(&schema).map_err(|message| p.error(message))?;
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
        "i8" => QslType::Integer {
            bits: 8,
            signed: true,
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
        "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
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
            "&A[predict_seq]{str?(0) x;}",
        ] {
            assert!(parse_qsl(source).is_err(), "{source}");
        }
        let a = parse_qsl("&A{str(10) x|binary_opt=view|;}").unwrap();
        assert_eq!(a[0].fields[0].data.rust_type(false).unwrap(), "[u8; 10]");
        assert!(!a[0].lifetime());
        let a = parse_qsl("&A{str x|binary_opt=view|;}").unwrap();
        assert!(a[0].lifetime());
        assert!(a[0].fields[0].data.rust_type(true).unwrap().contains("str"));
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
mod string_modes_tests {
    use super::*;
    #[test]
    fn retired_guard_and_byte_string_modes_are_rejected() {
        for mode in ["guard", "true", "bytes", "Utf8Guard"] {
            for spec in ["str", "str(0..100)", "str(10)"] {
                assert!(parse_qsl(&format!("&A{{{spec} x|binary_opt={mode}|;}}")).is_err());
            }
        }
    }
    #[test]
    fn explicit_view_generates_str_and_preserves_owned_siblings() {
        let schemas =
            parse_qsl("&A{str(0..100) text|binary_opt=view|;str(0..100) owned;}").unwrap();
        assert!(schemas[0].lifetime());
        assert_eq!(
            schemas[0].fields[0].data.rust_type(false).unwrap(),
            "&'a str"
        );
        assert_eq!(
            schemas[0].fields[1].data.rust_type(false).unwrap(),
            "String"
        );
    }
}
