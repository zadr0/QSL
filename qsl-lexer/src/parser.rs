//! QSL source parser. Expensive work (ranges, constants, flags) happens at build time.
//! The public `QslSchema`/`QslField` data model is not modified.

pub(crate) use crate::lexer::Lexer as Cursor;
pub use crate::lexer::ParseError;
use crate::{
    QslField, QslSchema, QslType, FLAG_ALIGNMENT, FLAG_COMPACT, FLAG_INLINE, FLAG_INLINE_ALWAYS,
    FLAG_INLINE_NEVER, FLAG_LIFETIME, FLAG_OPTIONAL_BITMAP, FLAG_OPT_TABLE, FLAG_PREDICT_SEQ,
    FLAG_SAFETY, FLAG_STATE_BITS, STATE_BITS_QUANTUM_SHIFT,
};
use std::collections::HashMap;

impl<'a> Cursor<'a> {
    pub(crate) fn expr(&mut self, defs: &HashMap<String, i128>) -> Result<i128, ParseError> {
        self.wide_expr(defs)?
            .signed()
            .ok_or_else(|| self.error("expression does not fit i128"))
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
/// Parse exact signed/unsigned constants without narrowing through i128.
pub fn parse_qsl_with_wide_constants(
    source: &str,
    globals: &HashMap<String, crate::WideNumber>,
) -> Result<Vec<QslSchema>, ParseError> {
    let raw = parse_qsl_raw_with_wide_primitives(source, globals, &Default::default())?;
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
    let wide = globals
        .iter()
        .map(|(k, v)| (k.clone(), crate::WideNumber::from_i128(*v)))
        .collect();
    parse_qsl_raw_with_wide_primitives(source, &wide, primitives)
}
#[doc(hidden)]
pub fn parse_qsl_raw_with_wide_primitives(
    source: &str,
    globals: &HashMap<String, crate::WideNumber>,
    primitives: &std::collections::BTreeMap<String, crate::PrimitiveRepr>,
) -> Result<Vec<QslSchema>, ParseError> {
    let mut p = Cursor::new(source);
    p.wide_constants = globals.clone();
    let mut defs = globals
        .iter()
        .filter_map(|(k, v)| v.signed().map(|n| (k.clone(), n)))
        .collect::<HashMap<_, _>>();
    let mut result = Vec::new();
    while !p.eof()? {
        p.expect("&")?;
        let name = p.ident()?.to_owned();
        let mut flags = 0;
        let mut options = crate::layout::SchemaOptions::default();
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
                    "aggressive" | "aggresive" => {
                        if attributes.contains(if attr=="aggressive"{"aggresive"}else{"aggressive"}) {return Err(p.error("aggressive and aggresive are aliases; specify only one"));}
                        options.aggressive=true;
                        if p.eat("[")? {
                            let mut chosen=std::collections::HashSet::new();
                            loop {
                                let key=p.ident()?;if !chosen.insert(key){return Err(p.error("duplicate aggressive option"));}
                                let enabled=if p.eat("=")? {match p.ident()? {"true"=>true,"false"=>false,_=>return Err(p.error("aggressive option must be true or false"))}}else{true};
                                match key {"shift"=>options.aggressive_shift=Some(enabled),"bitmap"=>options.aggressive_bitmap=enabled,"pointing"=>options.aggressive_pointing=enabled,"group"=>options.aggressive_group=enabled,_=>return Err(p.error("unknown aggressive option"))}
                                if p.eat("]")? {break;}p.expect(",")?;
                            }
                        }
                        0
                    },
                    "dyn" => {
                        use qsl_core::dynamic::{DynHeader,DynMode,DynWidth};
                        let mut h=DynHeader{mode:DynMode::DynamicIndex,width:DynWidth::W32,revision:1};
                        if p.eat("[")? {let mut chosen=std::collections::HashSet::new();loop{
                            let key=p.ident()?;if !chosen.insert(key){return Err(p.error("duplicate dyn option"));}p.expect("=")?;
                            match key {
                                "type"=>h.mode=match p.ident()?{"line"=>DynMode::Line,"index"=>{if p.eat("(")?{let m=match p.ident()?{"strict"=>DynMode::StrictIndex,"dynamic"=>DynMode::DynamicIndex,_=>return Err(p.error("dyn index expects strict or dynamic"))};p.expect(")")?;m}else{DynMode::DynamicIndex}},_=>return Err(p.error("dyn type must be line or index"))},
                                "width"=>h.width=match p.expr(&defs)?{16=>DynWidth::W16,32=>DynWidth::W32,64=>DynWidth::W64,_=>return Err(p.error("dyn width must be 16,32,64"))},
                                "revision"=>h.revision=u32::try_from(p.expr(&defs)?).map_err(|_|p.error("dyn revision must fit u32"))?,
                                _=>return Err(p.error("unknown dyn option")),
                            }
                            if p.eat("]")?{break;}p.expect(",")?;
                        }}
                        h.validate().map_err(|_|p.error("dyn index16 is undefined; use index32/64 or line16"))?;options.dynamic=Some(h);0
                    },
                    "auto" => {
                        options.auto_layout=true;
                        if p.eat("[")? {
                            if p.ident()? != "wire" { return Err(p.error("auto expects wire=1")); }
                            p.expect("=")?;
                            if p.expr(&defs)? != 1 { return Err(p.error("unsupported auto-wire version; expected 1")); }
                            options.auto_wire=Some(1);p.expect("]")?;
                        }
                        0
                    },
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
                    options.regions.push(crate::layout::NativeRegion {
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
                    let value = p.wide_expr(&defs)?;
                    if let Some(n) = value.signed() {
                        defs.insert(key.clone(), n);
                    } else {
                        defs.remove(&key);
                    }
                    p.wide_constants.insert(key, value);
                } else {
                    return Err(p.error("directive requires a value"));
                }
                p.expect(";")?;
                continue;
            }
            let ty_name = p.ident()?.to_owned();
            let element = if ty_name == "array" || ty_name == "vec" {
                p.expect("<")?;
                let inner = parse_element_type(&mut p, &defs, primitives, 0)?;
                p.expect(">")?;
                Some(inner)
            } else {
                None
            };
            let optional = p.eat("?")?;
            let wide = matches!(ty_name.as_str(), "u128" | "i128");
            let mut wide_range = None;
            let range = if wide {
                if p.eat("(")? {
                    let signed = ty_name == "i128";
                    let min = p
                        .wide_expr(&defs)?
                        .native(128, signed)
                        .ok_or_else(|| p.error("128-bit range outside type bounds"))?;
                    let max = if p.eat("..")? {
                        p.skip_trivia()?;
                        if p.source[p.pos..].starts_with(')') {
                            if signed {
                                i128::MAX as u128
                            } else {
                                u128::MAX
                            }
                        } else {
                            p.wide_expr(&defs)?
                                .native(128, signed)
                                .ok_or_else(|| p.error("128-bit range outside type bounds"))?
                        }
                    } else {
                        min
                    };
                    wide_range = Some((min, max));
                    p.expect(")")?;
                }
                None
            } else if p.eat("(")? {
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
            let mut tag = fields.len() + 1;
            let mut default = None;
            let mut decompress = None;
            let mut step = None;
            let mut values = None;
            while p.eat("|")? {
                let key = p.ident()?;
                match key {
                    "default" => {
                        if default.is_some() {
                            return Err(p.error("duplicate default modifier"));
                        }
                        p.expect("=")?;
                        default = Some(if wide{p.wide_expr(&defs)?.native(128,ty_name=="i128").ok_or_else(||p.error("default outside 128-bit type bounds"))?as i128}else{p.expr(&defs)?});
                    }
                    "step"=>{if step.is_some(){return Err(p.error("duplicate step modifier"));}p.expect("=")?;step=Some(p.wide_expr(&defs)?.unsigned().filter(|n|*n!=0).ok_or_else(||p.error("step must be a positive integer"))?);}
                    "values"=>{if values.is_some(){return Err(p.error("duplicate values modifier"));}p.expect("=")?;p.expect("[")?;let mut list=Vec::new();if !p.eat("]")?{loop{list.push(p.wide_expr(&defs)?);if p.eat("]")?{break;}p.expect(",")?;}}values=Some(list);}
                    "binary_opt" => return Err(p.error("binary_opt is retired; use [lifetime] for &str, [aggressive] for bounded StackString, or decompress=true for InlineString")),
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
            if p.eat("=")? {
                if options.dynamic.is_none() {
                    return Err(p.error("explicit field tags require dyn"));
                }
                tag = usize::try_from(p.expr(&defs)?)
                    .map_err(|_| p.error("dyn tag must be a nonzero native unsigned integer"))?;
            }
            p.expect(";")?;
            if tag == 0
                || tag
                    > if let Some(header) = options.dynamic {
                        header.tag_limit()
                    } else if flags & FLAG_OPTIONAL_BITMAP != 0 {
                        65535
                    } else {
                        64
                    }
            {
                return Err(p.error("QSL tag exceeds the selected codec capacity"));
            }
            max_tag = max_tag.max(tag);
            let (mut data, mut bits) = if wide {
                let signed = ty_name == "i128";
                let (min, max) = wide_range.unwrap_or(if signed {
                    (i128::MIN as u128, i128::MAX as u128)
                } else {
                    (0, u128::MAX)
                });
                let d = crate::domain::IntegerDomain::progression(128, signed, min, max, 1)
                    .map_err(|_| p.error("invalid 128-bit range"))?;
                let bits = d.encoded_bits() as usize;
                (QslType::DomainInteger(d.into()), bits)
            } else {
                build_type(&ty_name, element, range, None, &p)?
            };
            if step.is_some() || values.is_some() {
                use crate::domain::{low_mask, IntegerDomain, IntegerSet, NumericDomain};
                let base = match &data {
                    QslType::Byte => IntegerDomain::unsigned(8, 0, 255),
                    QslType::Integer { bits, signed, .. } => {
                        let mask = low_mask(*bits as u16);
                        if *signed {
                            IntegerDomain::signed(
                                *bits as u16,
                                -(1i128 << (*bits - 1)),
                                (1i128 << (*bits - 1)) - 1,
                            )
                        } else {
                            IntegerDomain::unsigned(*bits as u16, 0, mask)
                        }
                    }
                    QslType::RangedInteger {
                        bits,
                        signed,
                        min,
                        max,
                    } => {
                        if *signed {
                            IntegerDomain::signed(*bits as u16, *min, *max)
                        } else {
                            IntegerDomain::unsigned(*bits as u16, *min as u128, *max as u128)
                        }
                    }
                    QslType::DomainInteger(NumericDomain::Interval(d)) => Ok(*d),
                    _ => return Err(p.error("step/values require an integer field")),
                }
                .map_err(|_| p.error("invalid integer domain"))?;
                if !wide {
                    if let Some(value) = default {
                        default = Some(
                            crate::wide_number::WideNumber::from_i128(value)
                                .native(base.physical_bits(), base.signedness())
                                .ok_or_else(|| p.error("default outside integer type bounds"))?
                                as i128,
                        );
                    }
                }
                let progression = if let Some(step) = step {
                    IntegerDomain::progression(
                        base.physical_bits(),
                        base.signedness(),
                        base.minimum_bits(),
                        base.maximum_bits(),
                        step,
                    )
                    .map_err(|_| p.error("invalid progression domain"))?
                } else {
                    base
                };
                let domain = if let Some(values) = values {
                    let mut native = Vec::new();
                    for value in values {
                        let n = value
                            .native(base.physical_bits(), base.signedness())
                            .ok_or_else(|| p.error("set value outside integer type bounds"))?;
                        progression
                            .encode_bits(n)
                            .map_err(|_| p.error("set value outside declared range or step"))?;
                        native.push(n);
                    }
                    NumericDomain::Set(
                        IntegerSet::new(base.physical_bits(), base.signedness(), native)
                            .map_err(|_| p.error("values must be nonempty and unique"))?,
                    )
                } else {
                    NumericDomain::Interval(progression)
                };
                bits = domain.encoded_bits() as usize;
                data = QslType::DomainInteger(domain);
            }

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
            fields.push(QslField {
                shifted_length: false,
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
        crate::layout::validate_options(&schema).map_err(|message| p.error(message))?;
        result.push(schema);
    }
    Ok(result)
}

fn as_usize(v: i128, p: &Cursor<'_>) -> Result<usize, ParseError> {
    usize::try_from(v).map_err(|_| p.error("expected nonnegative usize value"))
}

fn primitive(name: &str) -> Option<QslType> {
    let ty = match name {
        "u128" => QslType::DomainInteger(
            crate::domain::IntegerDomain::unsigned(128, 0, u128::MAX)
                .unwrap()
                .into(),
        ),
        "i128" => QslType::DomainInteger(
            crate::domain::IntegerDomain::signed(128, i128::MIN, i128::MAX)
                .unwrap()
                .into(),
        ),
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

fn parse_element_type(
    p: &mut Cursor<'_>,
    defs: &HashMap<String, i128>,
    primitives: &std::collections::BTreeMap<String, crate::PrimitiveRepr>,
    depth: usize,
) -> Result<QslType, ParseError> {
    if depth >= 32 {
        return Err(p.error("array type nesting exceeds 32"));
    }
    let name = p.ident()?.to_owned();
    let element = if matches!(&*name, "array" | "vec") {
        p.expect("<")?;
        let e = parse_element_type(p, defs, primitives, depth + 1)?;
        p.expect(">")?;
        Some(e)
    } else {
        None
    };
    if p.eat("?")? {
        return Err(
            p.error("optional array elements are unsupported; make the array field optional")
        );
    }
    let integer = name
        .strip_prefix('u')
        .or_else(|| name.strip_prefix('i'))
        .and_then(|n| n.parse::<u16>().ok())
        .filter(|n| matches!(n, 8 | 16 | 32 | 64 | 128));
    if let Some(width) = integer {
        use crate::domain::{IntegerDomain, IntegerSet, NumericDomain};
        let signed = name.starts_with('i');
        let mask = crate::domain::low_mask(width);
        let (mut lo, mut hi) = if signed {
            (1u128 << (width - 1), (1u128 << (width - 1)) - 1)
        } else {
            (0, mask)
        };
        let ranged = p.eat("(")?;
        if ranged {
            lo = p
                .wide_expr(defs)?
                .native(width, signed)
                .ok_or_else(|| p.error("array element range outside type bounds"))?;
            hi = if p.eat("..")? {
                p.skip_trivia()?;
                if p.source[p.pos..].starts_with(')') {
                    hi
                } else {
                    p.wide_expr(defs)?
                        .native(width, signed)
                        .ok_or_else(|| p.error("array element range outside type bounds"))?
                }
            } else {
                lo
            };
            p.expect(")")?;
        }
        let mut step = 1;
        let mut values = None;
        let mut seen = std::collections::HashSet::new();
        while p.eat("|")? {
            let attr = p.ident()?.to_owned();
            if !seen.insert(attr.clone()) {
                return Err(p.error("duplicate array element modifier"));
            }
            p.expect("=")?;
            match &*attr {
                "step" => {
                    step = p
                        .wide_expr(defs)?
                        .unsigned()
                        .filter(|n| *n != 0)
                        .ok_or_else(|| p.error("step must be a positive integer"))?
                }
                "values" => {
                    p.expect("[")?;
                    let mut v = Vec::new();
                    if !p.eat("]")? {
                        loop {
                            v.push(p.wide_expr(defs)?.native(width, signed).ok_or_else(|| {
                                p.error("set value outside array element type bounds")
                            })?);
                            if p.eat("]")? {
                                break;
                            }
                            p.expect(",")?;
                        }
                    }
                    values = Some(v);
                }
                _ => return Err(p.error("array element supports only step and values")),
            };
            p.expect("|")?;
        }
        if !ranged && seen.is_empty() {
            return primitive(&name).ok_or_else(|| p.error("invalid integer element"));
        }
        let progression = IntegerDomain::progression(width, signed, lo, hi, step)
            .map_err(|_| p.error("invalid array element progression"))?;
        let d = if let Some(v) = values {
            for n in &v {
                progression
                    .encode_bits(*n)
                    .map_err(|_| p.error("set value outside array element range or step"))?;
            }
            NumericDomain::Set(
                IntegerSet::new(width, signed, v)
                    .map_err(|_| p.error("values must be nonempty and unique"))?,
            )
        } else {
            NumericDomain::Interval(progression)
        };
        return Ok(QslType::DomainInteger(d));
    }
    let range = if p.eat("(")? {
        let lo = p.expr(defs)?;
        let hi = if p.eat("..")? {
            Some(p.expr(defs)?)
        } else {
            None
        };
        p.expect(")")?;
        Some((lo, hi))
    } else {
        None
    };
    if let Some(repr) = primitives.get(&name) {
        if range.is_some() {
            return Err(p.error("registered custom elements define their own range"));
        }
        return Ok(QslType::Custom {
            rust_type: name,
            fixed_wire_size: Some(repr.bits as usize / 8),
        });
    }
    if !matches!(
        &*name,
        "array" | "vec" | "str" | "String" | "string" | "flags"
    ) {
        return Err(p.error("array elements must be built-in or explicitly registered primitives"));
    }
    Ok(build_type(&name, element, range, None, p)?.0)
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
        "padding_bits" => {
            let (n, rest) = range.ok_or_else(|| p.error("padding_bits requires (bit_count)"))?;
            if rest.is_some() {
                return Err(p.error("padding_bits requires fixed bit count"));
            }
            let n = as_usize(n, p)?;
            if n > u32::MAX as usize {
                return Err(p.error("padding_bits must fit u32"));
            }
            (QslType::BitPadding(n), n)
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
        let a = parse_qsl("&A{str(10) x;}").unwrap();
        assert_eq!(a[0].fields[0].data.rust_type(false).unwrap(), "[u8; 10]");
        assert!(!a[0].lifetime());
        let a = parse_qsl("&A[lifetime]{str x;}").unwrap();
        assert!(a[0].lifetime());
        assert!(a[0].fields[0].data.rust_type(true).unwrap().contains("str"));
    }
}

#[cfg(test)]
mod fixed_utf8_tests {
    use super::*;
    #[test]
    fn fixed_byte_representation_includes_equal_bounds_and_guard_modifiers() {
        for source in ["&A{str(10..10) x;}", "&A{str(10) x|decompress=true|;}"] {
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
    fn lifetime_replaces_retired_binary_opt() {
        assert!(parse_qsl("&A{str(0..100) text|binary_opt=view|;}").is_err());
        let schemas = parse_qsl("&A[lifetime]{str(0..100) text;}").unwrap();
        assert!(schemas[0].lifetime());
        assert_eq!(
            schemas[0].fields[0].data.rust_type(true).unwrap(),
            "&'a str"
        );
    }
}

#[cfg(test)]
mod exact_domain_parser_tests {
    use super::*;
    use crate::domain::{IntegerDomain, IntegerSet};
    #[test]
    fn text_and_builder_share_wide_defaults_steps_sets_and_hashes() {
        let source=format!("&Wide[safety,compact]{{u128(0..{}) x|default={}|;i128({}..{}) y|default={}|;i32(-9..9) step|step=3|;i128 set|values=[{},-1,0,{}]|;}}",u128::MAX,u128::MAX,i128::MIN,i128::MAX,i128::MIN,i128::MIN,i128::MAX);
        let text = crate::QslSchemaBuilder::from_source(&source).unwrap();
        let mut b = crate::QslBuilder::new();
        b.schem(|s| {
            s.name("Wide").safety().compact();
            s.field("x").u128(0, u128::MAX, u128::MAX, false);
            s.field("y").i128(i128::MIN, i128::MAX, i128::MIN, false);
            s.field("step").domain(
                IntegerDomain::progression(32, true, (-9i32) as u32 as u128, 9, 3).unwrap(),
                None,
                false,
            );
            s.field("set").domain(
                IntegerSet::signed(128, [i128::MIN, -1, 0, i128::MAX]).unwrap(),
                None,
                false,
            );
        });
        let b = b.build().unwrap();
        assert_eq!(
            text.schemas()[0].hashes(text.config().use_hash).unwrap(),
            b.schemas()[0].hashes(b.config().use_hash).unwrap()
        );
        assert_eq!(text.schemas()[0].fields[0].default, Some(-1));
        let reversed = source.replace(
            &format!("[{},-1,0,{}]", i128::MIN, i128::MAX),
            &format!("[0,{},-1,{}]", i128::MAX, i128::MIN),
        );
        let reversed = crate::QslSchemaBuilder::from_source(&reversed).unwrap();
        assert_eq!(
            text.schemas()[0].hashes(text.config().use_hash).unwrap(),
            reversed.schemas()[0]
                .hashes(text.config().use_hash)
                .unwrap()
        );
    }
    #[test]
    fn wide_literals_and_invalid_domains_have_diagnostics() {
        for source in [
            "&P{u128 x|default=-1|;}",
            "&P{i128(0..170141183460469231731687303715884105728) x;}",
            "&P{i128(-170141183460469231731687303715884105729..0) x;}",
            "&P{u128(0..340282366920938463463374607431768211456) x;}",
            "&P{u8 x|step=0|;}",
            "&P{u8(1..8) x|step=2|;}",
            "&P{u8 x|values=[]|;}",
            "&P{u8 x|values=[0,-0]|;}",
            "&P{i8 x|values=[-1,0]| |default=255|;}",
            "&P{u8(0..3) x|values=[5]|;}",
            "&P{str(0..8) x|step=2|;}",
            "&P{u8(0..8) x|step=2| |values=[1]|;}",
        ] {
            assert!(parse_qsl(source).is_err(), "accepted {source}");
        }
        let tokens = crate::Lexer::new("340282366920938463463374607431768211455")
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens[0].kind, crate::TokenKind::WideInteger(u128::MAX));
        let both = parse_qsl("&P{u8(0..8) x|step=2| |values=[0,2,8]|;}").unwrap();
        assert_eq!(both[0].fields[0].bits, 2);
        let aligned = parse_qsl("&P{u8(1..7) x|step=2|;}").unwrap();
        assert_eq!(aligned[0].fields[0].bits, 2);

        let p=parse_qsl("&P{u128(340282366920938463463374607431768211455-3..340282366920938463463374607431768211455) x;}").unwrap();
        assert_eq!(p[0].fields[0].bits, 2);
    }
}
