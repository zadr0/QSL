use crate::io::{QslError, QslResult, QslSizer};
use crate::types::{QslField, QslSchema, QslType, FLAG_ALIGNMENT};
use crate::QslIoPlan;
use std::fmt::Write as _;
use std::fs;
#[path = "codegen_length_graph.rs"]
mod length_graph_codegen;
#[path = "codegen_payload_quant.rs"]
mod payload_quant_codegen;
#[path = "codegen_qps.rs"]
mod qps_codegen;
#[path = "codegen_static_optional.rs"]
mod static_optional_codegen;
#[path = "codegen_validation.rs"]
mod validation_codegen;

/// Finite upper bound used by DEQ RT allocation and editor diagnostics.
pub fn schema_maximum_size(schema: &QslSchema) -> QslResult<Option<usize>> {
    schema.validate()?;
    qps_codegen::maximum_size(schema)
}

fn ln(out: &mut String, source: impl AsRef<str>) {
    out.push_str(source.as_ref());
    out.push('\n');
}

/// Indent generated Rust only once, during the Cargo build. This formatting
/// never runs when a serialized object is written or read.
fn indent_generated(source: &str) -> String {
    let mut result = String::with_capacity(source.len() + source.len() / 4);
    let mut depth = 0usize;

    for line in source.lines() {
        let code = line.trim();
        if code.is_empty() {
            result.push('\n');
            continue;
        }

        let starts_with_closing_brace = usize::from(code.starts_with('}'));
        for _ in 0..depth.saturating_sub(starts_with_closing_brace) {
            result.push_str("    ");
        }
        result.push_str(code);
        result.push('\n');

        let mut quoted = false;
        let mut escaped = false;
        for byte in code.bytes() {
            if quoted {
                if byte == b'"' && !escaped {
                    quoted = false;
                }
                escaped = byte == b'\\' && !escaped;
                continue;
            }
            if byte == b'"' {
                quoted = true;
            } else if byte == b'{' {
                depth += 1;
            } else if byte == b'}' {
                depth = depth.saturating_sub(1);
            }
        }
    }
    result
}

fn string_layout(ty: &QslType) -> Option<(usize, Option<usize>, usize, usize)> {
    match *ty {
        QslType::String(min) => Some((min, None, 4, 0)),
        QslType::RangedString { min, max } => Some((min, Some(max), 4, 0)),
        QslType::BinaryString { min, max }
        | QslType::GuardString { min, max }
        | QslType::CopyGuardString { min, max } => Some((min, max, 4, 0)),
        QslType::FixedString(n) => Some((n, Some(n), 0, 0)),
        QslType::FixedCompressedString(n) => Some((
            n,
            Some(n),
            1 + if n <= 255 { 1 } else { 2 },
            if n <= 255 { 1 } else { 2 },
        )),
        QslType::CompressedString { min, max } => Some((
            min,
            Some(max),
            if max <= 127 {
                1
            } else if max <= 32767 {
                2
            } else {
                4
            },
            if max <= 255 { 1 } else { 2 },
        )),
        QslType::InlineRawString { min, max } => Some((
            min,
            Some(max),
            if max <= 127 {
                1
            } else if max <= 32767 {
                2
            } else {
                4
            },
            0,
        )),
        _ => None,
    }
}

/// Generic cursor writers already know the actual initialized length. Return it
/// directly instead of walking variable fields again after writing.
fn needs_written_length(schema: &QslSchema) -> bool {
    if schema.opt_table() {
        return true;
    }
    if schema.state_bits() || schema.predict_table() {
        return true;
    }
    !(fixed_flat_layout(schema) || compact_flat_layout(schema) || optional_flat_layout(schema))
}

fn bytes_of(ty: &QslType) -> QslResult<usize> {
    ty.fixed_wire_size()?.ok_or(QslError::InvalidSchema)
}

/// The code generator does all stable arithmetic before emitting Rust source.
/// The same type-specific routines are used by the schema writer and reader.
pub trait TypeInterpreter {
    fn type_name(&self, out: &mut Vec<u8>, borrowed: bool) -> QslResult<()>;
    fn size(&self, out: &mut Vec<u8>, expr: &str, width: usize, compact: bool) -> QslResult<()>;
    fn write(&self, out: &mut Vec<u8>, expr: &str) -> QslResult<()>;
    fn read(&self, out: &mut Vec<u8>, name: &str, borrowed: bool) -> QslResult<()>;
    fn comp_write(&self, out: &mut Vec<u8>, expr: &str, bits: usize) -> QslResult<()>;
    fn comp_read(
        &self,
        out: &mut Vec<u8>,
        name: &str,
        bits: usize,
        borrowed: bool,
    ) -> QslResult<()>;
}

impl TypeInterpreter for QslType {
    fn type_name(&self, out: &mut Vec<u8>, borrowed: bool) -> QslResult<()> {
        out.extend_from_slice(self.rust_type(borrowed)?.as_bytes());
        Ok(())
    }
    fn size(&self, out: &mut Vec<u8>, expr: &str, width: usize, compact: bool) -> QslResult<()> {
        let mut src = String::new();
        emit_size_field(self, &mut src, expr, width, compact)?;
        out.extend_from_slice(src.as_bytes());
        Ok(())
    }
    fn write(&self, out: &mut Vec<u8>, expr: &str) -> QslResult<()> {
        let mut src = String::new();
        fast_type_write(self, &mut src, expr, 0, false)?;
        out.extend_from_slice(src.as_bytes());
        Ok(())
    }
    fn read(&self, out: &mut Vec<u8>, name: &str, borrowed: bool) -> QslResult<()> {
        let mut src = String::new();
        emit_unchecked_value(self, &mut src, name, 0, false, borrowed)?;
        out.extend_from_slice(src.as_bytes());
        Ok(())
    }
    fn comp_write(&self, out: &mut Vec<u8>, expr: &str, bits: usize) -> QslResult<()> {
        let mut src = String::new();
        fast_type_write(self, &mut src, expr, self.rank_width(bits)?, true)?;
        out.extend_from_slice(src.as_bytes());
        Ok(())
    }
    fn comp_read(
        &self,
        out: &mut Vec<u8>,
        name: &str,
        bits: usize,
        borrowed: bool,
    ) -> QslResult<()> {
        let mut src = String::new();
        emit_unchecked_value(self, &mut src, name, self.rank_width(bits)?, true, borrowed)?;
        out.extend_from_slice(src.as_bytes());
        Ok(())
    }
}

fn is_bytes(ty: &QslType) -> bool {
    matches!(ty, QslType::Byte)
}

/// Scalar optional I/O evaluates both address candidates, then selects a valid
/// pointer. It never reads an absent field or writes beyond exact wire capacity.
fn optional_scalar_read(
    ty: &QslType,
    out: &mut String,
    name: &str,
    present: &str,
) -> QslResult<bool> {
    if !matches!(
        ty,
        QslType::Byte | QslType::Integer { .. } | QslType::RangedInteger { .. } | QslType::Flags(_)
    ) {
        return Ok(false);
    }
    let rust = ty.primitive()?.ok_or(QslError::InvalidSchema)?;
    let size = bytes_of(ty)?;
    ln(out,format!("let {name}={{let present={present};let zero=0{rust};let source=std::hint::select_unpredictable(present,reader.as_ptr().cast::<{rust}>(),&zero as *const {rust});let source=::qsl::payload_quant::selected_address(source);let value=::qsl::payload_quant::scalar_operand(<{rust}>::from_le(std::ptr::read_unaligned(source)) as u64) as {rust};reader.read_exact({size}*(present as usize));present.then_some(value)}};"));
    Ok(true)
}
fn optional_scalar_write(field: &QslField, out: &mut String) -> QslResult<bool> {
    if !matches!(
        field.data,
        QslType::Byte | QslType::Integer { .. } | QslType::RangedInteger { .. } | QslType::Flags(_)
    ) {
        return Ok(false);
    }
    let rust = field.data.primitive()?.ok_or(QslError::InvalidSchema)?;
    let size = bytes_of(&field.data)?;
    ln(out,format!("{{let present=self.{}.is_some();let zero=0{rust};let source=std::hint::select_unpredictable(present,self.{}.as_slice().as_ptr(),&zero as *const {rust});let source=::qsl::payload_quant::selected_address(source);let value=std::ptr::read(source).to_le();let mut sink=std::mem::MaybeUninit::<{rust}>::uninit();let target=std::hint::select_unpredictable(present,writer.as_mut_ptr().cast::<{rust}>(),sink.as_mut_ptr());let target=::qsl::payload_quant::selected_address(target.cast_const()).cast_mut();std::ptr::write_unaligned(target,value);writer.claim_bytes({size}*(present as usize));}}",field.name,field.name));
    Ok(true)
}

fn min_literal(min: i128) -> String {
    if min == i128::MIN {
        "i128::MIN".into()
    } else {
        format!("{min}i128")
    }
}

fn minimum_field(s: &mut QslSizer, ty: &QslType, compact: bool, width: usize) -> QslResult<()> {
    match ty {
        QslType::BinaryString { min, .. }
        | QslType::GuardString { min, .. }
        | QslType::CopyGuardString { min, .. } => {
            return minimum_field(s, &QslType::String(*min), compact, width)
        }
        QslType::InlineRawString { min, max } => s.bytes(
            crate::InlineString::<0>::HEADER_BYTES
                + *min
                + if *max > 32767 {
                    3
                } else if *max > 127 {
                    1
                } else {
                    0
                },
        )?,
        QslType::FixedCompressedString(n) => {
            let width = if *n <= 255 { 1 } else { 2 };
            let min = if *n >= crate::STRING_COMPRESSION_MIN_BYTES {
                (*n).min(1 + width + 2 + (*n - 1).div_ceil(131) * (1 + width))
            } else {
                *n
            };
            s.bytes(min)?;
        }
        QslType::None => {}
        QslType::Padding(n) => s.bytes(*n)?,
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::Flags(_) => {
            if compact {
                s.bits(width)?;
            } else {
                s.bytes(bytes_of(ty)?)?;
            }
        }
        QslType::CompressedString { min, max } => {
            let header = if *max <= 127 {
                1
            } else if *max <= 32767 {
                2
            } else {
                4
            };
            let width = if *max <= 255 { 1 } else { 2 };
            let payload = if *max >= crate::STRING_COMPRESSION_MIN_BYTES {
                (*min).min(
                    2 + ((*min).max(crate::STRING_COMPRESSION_MIN_BYTES) - 1).div_ceil(131)
                        * (1 + width),
                )
            } else {
                *min
            };
            s.bytes(header + payload)?;
        }
        QslType::FixedString(n) => s.bytes(*n)?,
        QslType::String(min) => {
            if compact {
                s.bits(width)?;
            } else {
                s.bytes(4)?;
            }
            // Each Unicode scalar value requires at least one UTF-8 byte.
            s.bytes(*min)?;
        }
        QslType::RangedString { min, .. } => {
            if compact {
                s.bits(width)?;
            } else {
                s.bytes(4)?;
            }
            s.bytes(*min)?;
        }
        QslType::Array(elem, min)
        | QslType::RangedArray {
            element: elem, min, ..
        } => {
            if compact {
                s.bits(width)?;
            } else {
                s.bytes(4)?;
            }
            if is_bytes(elem) {
                s.bytes(*min)?;
            } else {
                for _ in 0..*min {
                    minimum_field(s, elem, compact, elem.default_compact_bits()?)?;
                }
                if compact {
                    s.align()?;
                }
            }
        }
        QslType::FixedArray(elem, n) => {
            if is_bytes(elem) {
                s.bytes(*n)?;
            } else {
                for _ in 0..*n {
                    minimum_field(s, elem, compact, elem.default_compact_bits()?)?;
                }
                if compact {
                    s.align()?;
                }
            }
        }
        QslType::Custom {
            fixed_wire_size, ..
        } => {
            if compact {
                s.align()?;
            } else if let Some(n) = fixed_wire_size {
                s.bytes(*n)?;
            } else {
                s.align()?;
            }
        }
    }
    Ok(())
}

fn minimum_size(schema: &QslSchema) -> QslResult<usize> {
    let mut s = QslSizer::default();
    if schema.opt_table() && !schema.state_bits() {
        s.bytes(crate::static_optional::StaticOptionalPlan::analyze(schema)?.header_bytes())?;
    } else if schema.legacy_opt_table() {
        s.bytes(1 + schema.optional_count() * 4)?;
    } else if schema.predict_seq() {
    } else if schema.bitmap() {
        s.bytes(schema.bitmap_words() * 4)?;
    } else if schema.optional_count() == 1 {
        s.bits(1)?;
        if !schema.compact() {
            s.align()?;
        }
    } else if schema.optional_count() > 0 {
        s.bits(schema.count_bits())?;
        if !schema.compact() {
            s.align()?;
        }
    }
    for f in &schema.fields {
        if !f.optional {
            minimum_field(
                &mut s,
                &f.data,
                schema.compact(),
                f.data.rank_width(f.bits)?,
            )?;
        }
        if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
            s.align()?;
        }
    }
    s.finish()
}

/// Resolve the minimum encoded size without guessing a custom codec's width.
/// Built-in field offsets are folded now; each custom size remains a Rust
/// associated constant, which the downstream compiler can fold at monomorphization.
fn minimum_size_expr(schema: &QslSchema) -> QslResult<String> {
    if schema.state_bits() || schema.predict_table() {
        let mut size = QslSizer::default();
        size.bytes(if schema.state_bits() { 4 } else { 0 })?;
        for f in schema.fields.iter().filter(|f| !f.optional) {
            minimum_field(&mut size, &f.data, false, f.bits)?;
        }
        return Ok(size.finish()?.to_string());
    }
    let mut size = minimum_size(schema)?.to_string();
    for field in schema.fields.iter().filter(|field| !field.optional) {
        if let QslType::Custom { rust_type, .. } = &field.data {
            let constant = if schema.compact() {
                "COMP_SIZE"
            } else {
                "SIZE"
            };
            size.push_str(&format!(" + <{rust_type} as ::qsl::QslCustom>::{constant}"));
        }
    }
    Ok(size)
}

fn emit_presence(schema: &QslSchema, ordinal: usize) -> String {
    if schema.bitmap() {
        let word = ordinal / 32;
        let shift = 31 - ordinal % 32;
        format!("(__qsl_mask_{word} & (1u32 << {shift})) != 0")
    } else {
        format!("__qsl_present[{ordinal}]")
    }
}

/// Collapsible contribution of an ordinary mandatory fixed-width field.
/// `true` means bit count, `false` means byte count.
fn constant_size(ty: &QslType, compact: bool, width: usize) -> Option<(bool, usize)> {
    match ty {
        QslType::None => Some((compact, 0)),
        QslType::Padding(n) => Some((false, *n)),
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::Flags(_) => {
            if compact {
                Some((true, width))
            } else {
                ty.fixed_wire_size().ok().flatten().map(|n| (false, n))
            }
        }
        QslType::FixedString(n) => Some((false, *n)),
        QslType::FixedArray(elem, n) if is_bytes(elem) => Some((false, *n)),
        _ => None,
    }
}

/// Emits arithmetic only. Schema constraints are verified by the parser; the
/// runtime sizing path has no Result, overflow checks or error branches.
fn emit_size_field(
    ty: &QslType,
    out: &mut String,
    value: &str,
    width: usize,
    compact: bool,
) -> QslResult<()> {
    match ty {
        QslType::BinaryString { min, .. }
        | QslType::GuardString { min, .. }
        | QslType::CopyGuardString { min, .. } => {
            return emit_size_field(&QslType::String(*min), out, value, width, compact)
        }
        QslType::InlineRawString { .. } => {
            add_size_bytes(out, &format!("({value}).wire_capacity()"))
        }
        QslType::FixedCompressedString(n) => add_size_bytes(out, &n.to_string()),
        QslType::None => {}
        QslType::Padding(n) => add_size_bytes(out, &n.to_string()),
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::Flags(_) => {
            if compact {
                ln(out, format!("__qsl_bits += {width};"));
            } else {
                add_size_bytes(out, &bytes_of(ty)?.to_string());
            }
        }
        QslType::CompressedString { .. } => {
            add_size_bytes(out, &format!("({value}).wire_capacity()"))
        }
        QslType::FixedString(n) => add_size_bytes(out, &n.to_string()),
        QslType::String(_) | QslType::RangedString { .. } => {
            if compact {
                ln(out, format!("__qsl_bits += {width};"));
            } else {
                add_size_bytes(out, "4");
            }
            add_size_bytes(out, &format!("({value}).len()"));
        }
        QslType::Array(elem, _) | QslType::RangedArray { element: elem, .. } => {
            if compact {
                ln(out, format!("__qsl_bits += {width};"));
            } else {
                add_size_bytes(out, "4");
            }
            if is_bytes(elem) {
                add_size_bytes(out, &format!("({value}).len()"));
            } else if let Some((is_bits, per_item)) =
                constant_size(elem, compact, elem.default_compact_bits()?)
            {
                let total = format!("({value}).len() * {per_item}");
                if is_bits {
                    ln(out, format!("__qsl_bits += {total};"));
                } else {
                    add_size_bytes(out, &total);
                }
                if compact {
                    ln(out, "__qsl_bits = (__qsl_bits + 7) & !7usize;");
                }
            } else {
                ln(out, format!("for __qsl_item in ({value}).iter() {{"));
                emit_size_field(
                    elem,
                    out,
                    "__qsl_item",
                    elem.default_compact_bits()?,
                    compact,
                )?;
                ln(out, "}");
                if compact {
                    ln(out, "__qsl_bits = (__qsl_bits + 7) & !7usize;");
                }
            }
        }
        QslType::FixedArray(elem, n) => {
            if is_bytes(elem) {
                add_size_bytes(out, &n.to_string());
            } else if let Some((is_bits, per_item)) =
                constant_size(elem, compact, elem.default_compact_bits()?)
            {
                let total = n.checked_mul(per_item).ok_or(QslError::LengthOverflow)?;
                if is_bits {
                    ln(out, format!("__qsl_bits += {total};"));
                } else {
                    add_size_bytes(out, &total.to_string());
                }
                if compact {
                    ln(out, "__qsl_bits = (__qsl_bits + 7) & !7usize;");
                }
            } else {
                ln(out, format!("for __qsl_item in ({value}).iter() {{"));
                emit_size_field(
                    elem,
                    out,
                    "__qsl_item",
                    elem.default_compact_bits()?,
                    compact,
                )?;
                ln(out, "}");
                if compact {
                    ln(out, "__qsl_bits = (__qsl_bits + 7) & !7usize;");
                }
            }
        }
        QslType::Custom { .. } => {
            let size = if compact { "qsl_comp_size" } else { "qsl_size" };
            add_size_bytes(out, &format!("({value}).{size}()"));
        }
    }
    Ok(())
}

#[inline]
fn add_size_bytes(out: &mut String, bytes: &str) {
    ln(
        out,
        format!("__qsl_bits = ((__qsl_bits + 7) & !7usize) + ({bytes}) * 8;"),
    );
}

fn emit_size(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    if schema.opt_table() && !schema.state_bits() {
        if let crate::static_optional::StaticPresence::Graph(p) =
            crate::static_optional::StaticOptionalPlan::analyze(schema)?.presence
        {
            return length_graph_codegen::size(schema, &p, out);
        }
    }
    if schema.state_bits() || schema.predict_table() {
        return qps_codegen::size(schema, out);
    }
    ln(out, "#[inline(always)] fn __qsl_size(&self) -> usize {");
    if schema.sequential() {
        ln(
            out,
            format!(
                "let mut __qsl_bits: usize = {};",
                if schema.opt_table() && !schema.state_bits() {
                    crate::static_optional::StaticOptionalPlan::analyze(schema)?.header_bytes() * 8
                } else if schema.legacy_opt_table() {
                    (1 + schema.optional_count() * 4) * 8
                } else {
                    0
                }
            ),
        );
    } else if schema.bitmap() {
        ln(
            out,
            format!(
                "let mut __qsl_bits: usize = {};",
                schema.bitmap_words() * 32
            ),
        );
    } else if schema.optional_count() == 1 {
        ln(out, "let mut __qsl_bits: usize = 1;");
    } else if schema.optional_count() > 1 {
        let sum = schema
            .fields
            .iter()
            .filter(|f| f.optional)
            .map(|f| format!("(self.{}.is_some() as usize)", f.name))
            .collect::<Vec<_>>()
            .join(" + ");
        ln(
            out,
            format!(
                "let mut __qsl_bits: usize = {} + ({sum}) * {};",
                schema.count_bits(),
                schema.tag_bits()
            ),
        );
    } else {
        ln(out, "let mut __qsl_bits: usize = 0;");
    }
    if !schema.compact() {
        ln(out, "__qsl_bits = (__qsl_bits + 7) & !7usize;");
    }
    for field in &schema.fields {
        let width = field.data.rank_width(field.bits)?;
        if field.optional {
            ln(
                out,
                format!("if let Some(__qsl_value) = self.{}.as_ref() {{", field.name),
            );
            emit_size_field(&field.data, out, "__qsl_value", width, schema.compact())?;
            ln(out, "}");
        } else {
            emit_size_field(
                &field.data,
                out,
                &format!("&self.{}", field.name),
                width,
                schema.compact(),
            )?;
        }
        if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "__qsl_bits = (__qsl_bits + 7) & !7usize;");
        }
    }
    ln(out, "(__qsl_bits + 7) >> 3");
    ln(out, "}");
    Ok(())
}

fn fast_type_write(
    ty: &QslType,
    out: &mut String,
    expr: &str,
    width: usize,
    compact: bool,
) -> QslResult<()> {
    match ty {
        QslType::BinaryString { .. } | QslType::GuardString { .. }
        | QslType::CopyGuardString { .. } => {
            ln(out, format!("let __qsl_bytes: &[u8] = ({expr}).as_ref(); let __qsl_len = __qsl_bytes.len();"));
            if compact {
                let min = ty.rank_base();
                ln(out, format!("writer.write_bits_unchecked((__qsl_len - {min}) as u64, {width});"));
            } else { ln(out, "writer.write_bytes_unchecked(&(__qsl_len as u32).to_le_bytes());"); }
            ln(out, "writer.write_bytes_unchecked(__qsl_bytes);");
        }
        QslType::InlineRawString { .. } => ln(out, format!("writer.flush_unchecked(); let __qsl_used = ({expr}).encode_raw_to(writer.as_mut_ptr()); writer.claim_bytes(__qsl_used);")),
        QslType::FixedCompressedString(n) => ln(out, format!("writer.flush_unchecked(); let __qsl_used = ::qsl::encode_fixed_utf8::<{n}>(({expr}).as_ptr(), writer.as_mut_ptr()); writer.claim_bytes(__qsl_used);")),
        QslType::None => {}
        QslType::Padding(n) => ln(out, format!("writer.write_zeros_unchecked({n});")),
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::Flags(_) => {
            if compact {
                ln(out, format!(
                    "writer.write_bits_unchecked(((*{expr} as i128).wrapping_sub({})) as u64, {width});",
                    min_literal(ty.rank_base())
                ));
            } else {
                ln(
                    out,
                    format!("writer.write_bytes_unchecked(&(*{expr}).to_le_bytes());"),
                );
            }
        }
        QslType::CompressedString { max, .. } => {
            ln(out, format!("writer.write_inline_string_with::<{max},{}>({expr});", string_header_word(*max)))
        }
        QslType::FixedString(_) => {
            ln(
                out,
                format!("writer.write_bytes_unchecked({expr});"),
            );
        }
        QslType::String(_) | QslType::RangedString { .. } => {
            let base = ty.rank_base();
            ln(out, format!("let __qsl_len = ({expr}).len();"));
            if compact {
                ln(
                    out,
                    format!(
                        "writer.write_bits_unchecked((__qsl_len - {base}) as u64, {width});"
                    ),
                );
            } else {
                ln(
                    out,
                    "writer.write_bytes_unchecked(&(__qsl_len as u32).to_le_bytes());",
                );
            }
            ln(
                out,
                format!("writer.write_bytes_unchecked(({expr}).as_bytes());"),
            );
        }
        QslType::Array(elem, min)
        | QslType::RangedArray {
            element: elem, min, ..
        } => {
            if compact {
                ln(
                    out,
                    format!(
                        "writer.write_bits_unchecked((({expr}).len() - {min}) as u64, {width});"
                    ),
                );
            } else {
                ln(
                    out,
                    format!(
                        "writer.write_bytes_unchecked(&(({expr}).len() as u32).to_le_bytes());"
                    ),
                );
            }
            if is_bytes(elem) {
                ln(out, format!("writer.write_bytes_unchecked({expr});"));
            } else {
                ln(out, format!("for __qsl_item in ({expr}).iter() {{"));
                fast_type_write(
                    elem,
                    out,
                    "__qsl_item",
                    elem.default_compact_bits()?,
                    compact,
                )?;
                ln(out, "}");
                if compact {
                    ln(out, "writer.flush_unchecked();");
                }
            }
        }
        QslType::FixedArray(elem, _) => {
            if is_bytes(elem) {
                ln(out, format!("writer.write_bytes_unchecked({expr});"));
            } else {
                ln(out, format!("for __qsl_item in ({expr}).iter() {{"));
                fast_type_write(
                    elem,
                    out,
                    "__qsl_item",
                    elem.default_compact_bits()?,
                    compact,
                )?;
                ln(out, "}");
                if compact {
                    ln(out, "writer.flush_unchecked();");
                }
            }
        }
        QslType::Custom { .. } => {
            let size_fn = if compact { "qsl_comp_size" } else { "qsl_size" };
            let write_fn = if compact {
                "qsl_comp_write_unchecked"
            } else {
                "qsl_write_unchecked"
            };
            ln(
                out,
                format!("let __qsl_dst = writer.claim_bytes(({expr}).{size_fn}());"),
            );
            ln(out, format!("({expr}).{write_fn}(__qsl_dst);"));
        }
    }
    Ok(())
}

fn fixed_flat_layout(schema: &QslSchema) -> bool {
    !schema.compact()
        && schema.fields.iter().all(|f| {
            !f.optional
                && matches!(
                    &f.data,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::Flags(_)
                        | QslType::Padding(_)
                        | QslType::None
                )
        })
}

fn compact_flat_layout(schema: &QslSchema) -> bool {
    schema.compact()
        && schema.flags & FLAG_ALIGNMENT == 0
        && schema.fields.iter().all(|f| {
            !f.optional
                && matches!(
                    &f.data,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::Flags(_)
                        | QslType::None
                )
        })
}

/// A variable-size record that can still be written by unrolled pointer
/// copies, with one data-dependent 0-or-N-byte copy and no control branches.
fn optional_flat_layout(schema: &QslSchema) -> bool {
    !schema.compact()
        && !schema.sequential()
        && schema.optional_count() == 1
        && schema.fields.iter().all(|f| {
            matches!(
                &f.data,
                QslType::Byte
                    | QslType::Integer { .. }
                    | QslType::RangedInteger { .. }
                    | QslType::Flags(_)
            ) || (!f.optional && matches!(&f.data, QslType::Padding(_) | QslType::None))
        })
}

fn emit_optional_flat_write(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let optional = schema
        .fields
        .iter()
        .find(|f| f.optional)
        .ok_or(QslError::InvalidSchema)?;
    ln(
        out,
        format!(
            "let __qsl_presence = self.{}.is_some() as usize;",
            optional.name
        ),
    );
    let mut offset = if schema.bitmap() {
        schema.bitmap_words() * 4
    } else {
        1
    };
    if schema.bitmap() {
        for (index, fields) in schema.fields.chunks(32).enumerate() {
            let mut required_mask = 0u32;
            let mut optional_mask = 0u32;
            for (slot, field) in fields.iter().enumerate() {
                if field.optional {
                    optional_mask |= 1u32 << (31 - slot);
                } else {
                    required_mask |= 1u32 << (31 - slot);
                }
            }
            ln(out, format!("std::ptr::write_unaligned(to.add({}) as *mut u32, ({}u32 | (__qsl_presence as u32 * {}u32)).to_le());", index*4, required_mask, optional_mask));
        }
    } else {
        ln(out, "std::ptr::write(to, __qsl_presence as u8);");
    }
    let mut dynamic = 0usize;
    for field in &schema.fields {
        let place = if dynamic == 0 {
            format!("{offset}")
        } else {
            format!("{offset} + __qsl_presence * {dynamic}")
        };
        if field.optional {
            let bytes = bytes_of(&field.data)?;
            ln(
                out,
                format!(
                    "let __qsl_opt = self.{}.unwrap_or(0).to_le_bytes();",
                    field.name
                ),
            );
            ln(out, format!("std::ptr::copy_nonoverlapping(__qsl_opt.as_ptr(), to.add({place}), {bytes} * __qsl_presence);"));
            dynamic += bytes;
        } else {
            match &field.data {
                QslType::Padding(n) => {
                    ln(
                        out,
                        format!("std::ptr::write_bytes(to.add({place}), 0, {n});"),
                    );
                    offset += n;
                }
                QslType::None => {}
                ty => {
                    let rust_ty = ty.primitive()?.ok_or(QslError::InvalidSchema)?;
                    ln(out, format!("std::ptr::write_unaligned(to.add({place}) as *mut {rust_ty}, self.{}.to_le());", field.name));
                    offset += bytes_of(ty)?;
                }
            }
        }
    }
    Ok(())
}

fn emit_compact_flat_write(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let mut offset = 0usize;
    if schema.bitmap() {
        for fields in schema.fields.chunks(32) {
            let mask = if fields.len() == 32 {
                u32::MAX
            } else {
                u32::MAX << (32 - fields.len())
            };
            ln(
                out,
                format!(
                    "std::ptr::write_unaligned(to.add({offset}) as *mut u32, {mask}u32.to_le());"
                ),
            );
            offset += 4;
        }
    }

    let mut words: Vec<Vec<String>> = Vec::new();
    let mut cursor = 0usize;
    for f in &schema.fields {
        if matches!(&f.data, QslType::None) {
            continue;
        }
        let width = f.data.rank_width(f.bits)?;
        if width == 0 {
            continue;
        }
        let raw = format!(
            "((self.{} as i128).wrapping_sub({}) as u64)",
            f.name,
            min_literal(f.data.rank_base())
        );
        let word = cursor / 64;
        let shift = cursor % 64;
        while words.len() <= word {
            words.push(Vec::new());
        }
        let first = width.min(64 - shift);
        let part = if first == 64 {
            raw.clone()
        } else {
            format!("({raw} & {}u64)", (1u64 << first) - 1)
        };
        words[word].push(format!("({part} << {shift})"));
        if first < width {
            if words.len() <= word + 1 {
                words.push(Vec::new());
            }
            words[word + 1].push(format!("({raw} >> {first})"));
        }
        cursor = cursor.checked_add(width).ok_or(QslError::LengthOverflow)?;
    }

    for (word_index, fragments) in words.iter().enumerate() {
        let word_offset = offset
            .checked_add(word_index * 8)
            .ok_or(QslError::LengthOverflow)?;
        let remaining = cursor.saturating_sub(word_index * 64);
        let actual_bytes = remaining.div_ceil(8).min(8);
        let expr = fragments.join(" | ");
        if actual_bytes == 8 {
            ln(
                out,
                format!(
                "std::ptr::write_unaligned(to.add({word_offset}) as *mut u64, ({expr}).to_le());"
            ),
            );
        } else if actual_bytes > 0 {
            ln(
                out,
                format!(
                    "let __qsl_word_{word_index} = (({expr}) as {}).to_le_bytes();",
                    native_word(actual_bytes)
                ),
            );
            ln(out, format!(
                "std::ptr::copy_nonoverlapping(__qsl_word_{word_index}.as_ptr(), to.add({word_offset}), {actual_bytes});"
            ));
        }
    }
    Ok(())
}

fn emit_flat_write(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let mut offset = 0usize;
    if schema.bitmap() {
        for fields in schema.fields.chunks(32) {
            let mask = if fields.len() == 32 {
                u32::MAX
            } else {
                u32::MAX << (32 - fields.len())
            };
            ln(
                out,
                format!(
                    "std::ptr::write_unaligned(to.add({offset}) as *mut u32, {}u32.to_le());",
                    mask
                ),
            );
            offset += 4;
        }
    }
    for f in &schema.fields {
        match &f.data {
            QslType::None => {}
            QslType::Padding(n) => {
                if *n != 0 {
                    ln(
                        out,
                        format!("std::ptr::write_bytes(to.add({offset}), 0, {n});"),
                    );
                }
                offset += n;
            }
            other => {
                let rust_ty = other.primitive()?.ok_or(QslError::InvalidSchema)?;
                ln(out, format!(
                    "std::ptr::write_unaligned(to.add({offset}) as *mut {rust_ty}, self.{}.to_le());",
                    f.name
                ));
                offset += bytes_of(other)?;
            }
        }
    }
    Ok(())
}

/// Emit straight-line reads from constant byte offsets. All offsets are folded
/// at generation time; the generated method has no error or length branches.
fn emit_flat_read(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let mut offset = if schema.bitmap() {
        schema.bitmap_words() * 4
    } else {
        0
    };
    for (i, field) in schema.fields.iter().enumerate() {
        match &field.data {
            QslType::None => ln(out, format!("let __qsl_field_{i} = ();")),
            QslType::Padding(len) => {
                ln(out, format!("let __qsl_field_{i} = ();"));
                offset += len;
            }
            ty => {
                let primitive = ty.primitive()?.ok_or(QslError::InvalidSchema)?;
                ln(out, format!(
                    "let __qsl_field_{i} = {primitive}::from_le(std::ptr::read_unaligned(bytes.add({offset}) as *const {primitive}));"
                ));
                offset += bytes_of(ty)?;
            }
        }
    }
    ln(out, "Self {");
    for (i, field) in schema.fields.iter().enumerate() {
        ln(out, format!("{}: __qsl_field_{i},", field.name));
    }
    ln(out, "}");
    Ok(())
}

/// Emit a fixed compact-layout decoder with compile-time shifts, masks and
/// exact-width loads. Partial trailing words never read past the allocation.
fn emit_compact_flat_read(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let offset = if schema.bitmap() {
        schema.bitmap_words() * 4
    } else {
        0
    };
    let total_bits = schema.fields.iter().try_fold(0usize, |sum, field| {
        sum.checked_add(field.data.rank_width(field.bits)?)
            .ok_or(QslError::LengthOverflow)
    })?;
    for word in 0..total_bits.div_ceil(64) {
        let shift = word * 64;
        let bytes = (total_bits - shift).div_ceil(8).min(8);
        let start = offset + word * 8;
        let word_type = native_word(bytes);
        let mut parts = Vec::new();
        let mut position = 0;
        while position < bytes {
            let n = if bytes - position >= 8 {
                8
            } else if bytes - position >= 4 {
                4
            } else if bytes - position >= 2 {
                2
            } else {
                1
            };
            let ty = native_word(n);
            parts.push(format!("(({ty}::from_le(std::ptr::read_unaligned(bytes.add({}).cast::<{ty}>())) as {word_type}) << {})",start+position,position*8));
            position += n;
        }
        ln(
            out,
            format!(
                "let __qsl_word_{word}:{word_type}={};",
                if parts.len() == 1 {
                    parts[0]
                        .strip_prefix('(')
                        .and_then(|p| p.strip_suffix(')'))
                        .unwrap_or(&parts[0])
                        .to_owned()
                } else {
                    parts.join(" | ")
                }
            ),
        );
    }
    let mut cursor = 0usize;
    for (i, field) in schema.fields.iter().enumerate() {
        let width = field.data.rank_width(field.bits)?;
        if matches!(field.data, QslType::None) {
            ln(out, format!("let __qsl_field_{i} = ();"));
            continue;
        }
        let primitive = field.data.primitive()?.ok_or(QslError::InvalidSchema)?;
        let expr = if width == 0 {
            "0u64".to_owned()
        } else {
            let shift = cursor % 64;
            let word = cursor / 64;
            let raw = if shift + width <= 64 {
                format!("((__qsl_word_{word} as u64) >> {shift})")
            } else {
                format!(
                    "(((__qsl_word_{word} as u64) >> {shift}) | ((__qsl_word_{} as u64) << {}))",
                    word + 1,
                    64 - shift
                )
            };
            if width == 64 {
                raw
            } else {
                format!("({raw} & {}u64)", (1u64 << width) - 1)
            }
        };
        ln(
            out,
            format!(
                "let __qsl_field_{i} = (({expr} as i128).wrapping_add({})) as {primitive};",
                min_literal(field.data.rank_base())
            ),
        );
        cursor += width;
    }
    ln(out, "Self {");
    for (i, field) in schema.fields.iter().enumerate() {
        ln(out, format!("{}: __qsl_field_{i},", field.name));
    }
    ln(out, "}");
    Ok(())
}

fn unchecked_read_supported(schema: &QslSchema) -> bool {
    schema.fields.iter().all(|field| match &field.data {
        QslType::None
        | QslType::Padding(_)
        | QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::Flags(_)
        | QslType::GuardString { .. }
        | QslType::CopyGuardString { .. }
        | QslType::BinaryString { .. }
        | QslType::InlineRawString { .. }
        | QslType::FixedCompressedString(_)
        | QslType::CompressedString { .. }
        | QslType::FixedString(_)
        | QslType::String(_)
        | QslType::RangedString { .. } => true,
        QslType::Array(elem, _)
        | QslType::FixedArray(elem, _)
        | QslType::RangedArray { element: elem, .. } => is_bytes(elem),
        QslType::Custom { .. } => true,
    })
}

fn emit_unchecked_value(
    ty: &QslType,
    out: &mut String,
    name: &str,
    width: usize,
    compact: bool,
    borrowed: bool,
) -> QslResult<()> {
    match ty {
        QslType::BinaryString { min, .. }
        | QslType::GuardString { min, .. }
        | QslType::CopyGuardString { min, .. } => {
            if compact {
                ln(
                    out,
                    format!("let __qsl_len = reader.read_bits({width}) as usize + {min};"),
                );
            } else {
                ln(out, "let __qsl_len = reader.read_u32() as usize;");
            }
            if matches!(ty, QslType::CopyGuardString { .. }) {
                ln(out, format!("let {name} = ::qsl::Utf8CopyGuard::from_bytes_unchecked(std::slice::from_raw_parts(reader.read_exact(__qsl_len), __qsl_len));"));
            } else if matches!(ty, QslType::GuardString { .. }) {
                ln(out, format!("let {name} = ::qsl::Utf8Guard::from_bytes_unchecked(std::slice::from_raw_parts(reader.read_exact(__qsl_len), __qsl_len));"));
            } else {
                ln(out, format!("let {name} = std::slice::from_raw_parts(reader.read_exact(__qsl_len), __qsl_len);"));
            }
        }
        QslType::InlineRawString { max, .. } => ln(
            out,
            format!(
                "let {name} = reader.read_raw_inline_string_with::<{max},{}>();",
                string_header_word(*max)
            ),
        ),
        QslType::FixedCompressedString(n) => {
            ln(
                out,
                format!("let mut __qsl_array = std::mem::MaybeUninit::<[u8; {n}]>::uninit();"),
            );
            ln(out, format!("reader.align(); let __qsl_used = ::qsl::decode_fixed_utf8::<{n}>(reader.as_ptr(), __qsl_array.as_mut_ptr().cast()); reader.read_exact(__qsl_used); let {name} = __qsl_array.assume_init();"));
        }
        QslType::None => ln(out, format!("let {name} = ();")),
        QslType::Padding(size) => {
            ln(out, format!("let _ = reader.read_exact({size});"));
            ln(out, format!("let {name} = ();"));
        }
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::Flags(_) => {
            let ty_name = ty.primitive()?.ok_or(QslError::InvalidSchema)?;
            if compact {
                ln(out, format!(
                    "let {name} = ((reader.read_bits({width}) as i128).wrapping_add({})) as {ty_name};",
                    min_literal(ty.rank_base())
                ));
            } else {
                ln(out, format!("let {name} = reader.read_{ty_name}();"));
            }
        }
        QslType::CompressedString { max, .. } => ln(
            out,
            format!(
                "let {name}=reader.read_inline_string_with::<{max},{}>();",
                string_header_word(*max)
            ),
        ),
        QslType::FixedString(size) => {
            ln(out, format!("let {name} = std::ptr::read_unaligned(reader.read_exact({size}).cast::<[u8; {size}]>());"));
        }
        QslType::String(_) | QslType::RangedString { .. } => {
            if compact {
                ln(
                    out,
                    format!(
                        "let __qsl_len = reader.read_bits({width}) as usize + {};",
                        ty.rank_base()
                    ),
                );
            } else {
                ln(out, "let __qsl_len = reader.read_u32() as usize;");
            }
            ln(out, "let __qsl_ptr = reader.read_exact(__qsl_len);");
            if borrowed {
                ln(out, format!("let {name} = std::str::from_utf8_unchecked(std::slice::from_raw_parts(__qsl_ptr, __qsl_len));"));
            } else {
                ln(out, format!("let {name} = String::from_utf8_unchecked(std::slice::from_raw_parts(__qsl_ptr, __qsl_len).to_vec());"));
            }
        }
        QslType::Array(_, min) | QslType::RangedArray { min, .. } => {
            if compact {
                ln(
                    out,
                    format!("let __qsl_len = reader.read_bits({width}) as usize + {min};"),
                );
            } else {
                ln(out, "let __qsl_len = reader.read_u32() as usize;");
            }
            ln(out, "let __qsl_ptr = reader.read_exact(__qsl_len);");
            if borrowed {
                ln(
                    out,
                    format!("let {name} = std::slice::from_raw_parts(__qsl_ptr, __qsl_len);"),
                );
            } else {
                ln(
                    out,
                    format!(
                        "let {name} = std::slice::from_raw_parts(__qsl_ptr, __qsl_len).to_vec();"
                    ),
                );
            }
        }
        QslType::FixedArray(_, exact) => {
            ln(out, format!("let __qsl_ptr = reader.read_exact({exact});"));
            if borrowed {
                ln(
                    out,
                    format!("let {name} = std::slice::from_raw_parts(__qsl_ptr, {exact});"),
                );
            } else {
                ln(
                    out,
                    format!(
                        "let {name} = std::slice::from_raw_parts(__qsl_ptr, {exact}).to_vec();"
                    ),
                );
            }
        }
        QslType::Custom { rust_type, .. } => {
            let (size, decoder) = if compact {
                ("COMP_SIZE", "qsl_comp_read_unchecked")
            } else {
                ("SIZE", "qsl_read_unchecked")
            };
            ln(out, format!(
                "let {name} = {rust_type}::{decoder}(reader.read_exact(<{rust_type} as ::qsl::QslCustom>::{size}));"
            ));
        }
    }
    Ok(())
}

fn string_header_word(max: usize) -> &'static str {
    if max <= 127 {
        "u8"
    } else if max <= 32767 {
        "u16"
    } else {
        "u32"
    }
}

fn native_word(bytes: usize) -> &'static str {
    match bytes {
        0..=1 => "u8",
        2 => "u16",
        3..=4 => "u32",
        _ => "u64",
    }
}
fn emit_aligned_rank_read(out: &mut String, name: &str, width: usize) {
    let bytes = width / 8;
    let word = native_word(bytes);
    ln(
        out,
        format!("let __qsl_rank_ptr=reader.read_exact({bytes});"),
    );
    let mut pieces = Vec::new();
    let mut offset = 0;
    while offset < bytes {
        let chunk = if bytes - offset >= 8 {
            8
        } else if bytes - offset >= 4 {
            4
        } else if bytes - offset >= 2 {
            2
        } else {
            1
        };
        let ty = native_word(chunk);
        pieces.push(format!("(({ty}::from_le(std::ptr::read_unaligned(__qsl_rank_ptr.add({offset}).cast::<{ty}>())) as {word}) << {})",offset*8));
        offset += chunk;
    }
    ln(
        out,
        format!(
            "let {name}:{word}={};",
            if pieces.len() == 1 {
                pieces[0]
                    .strip_prefix('(')
                    .and_then(|p| p.strip_suffix(')'))
                    .unwrap_or(&pieces[0])
                    .to_owned()
            } else {
                pieces.join(" | ")
            }
        ),
    );
}
fn emit_planned_write(
    ty: &QslType,
    out: &mut String,
    value: &str,
    width: usize,
    compact: bool,
    aligned: bool,
) -> QslResult<()> {
    if !compact || !aligned {
        return fast_type_write(ty, out, value, width, compact);
    }
    let word = native_word(width.div_ceil(8));
    let bytes = width / 8;
    match ty {
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::Flags(_) => {
            let base = min_literal(ty.rank_base());
            let cast = if ty.primitive()? == Some(word) {
                format!("*({value})")
            } else {
                format!("*({value}) as {word}")
            };
            let rank = if ty.rank_base() == 0 {
                format!("({cast})")
            } else {
                format!("({cast}).wrapping_sub({base} as {word})")
            };
            ln(out, format!("let __qsl_rank=({rank}).to_le_bytes();"));
            ln(
                out,
                format!("writer.write_ptr_unchecked(__qsl_rank.as_ptr(),{bytes});"),
            );
        }
        QslType::String(_)
        | QslType::RangedString { .. }
        | QslType::BinaryString { .. }
        | QslType::GuardString { .. }
        | QslType::CopyGuardString { .. }
        | QslType::Array(_, _)
        | QslType::RangedArray { .. } => {
            if matches!(ty, QslType::String(_) | QslType::RangedString { .. }) {
                ln(out, format!("let __qsl_data=({value}).as_bytes();"));
            } else {
                ln(out, format!("let __qsl_data:&[u8]=({value}).as_ref();"));
            }
            let min = ty.rank_base();
            ln(
                out,
                format!("let __qsl_rank=((__qsl_data.len()-{min}) as {word}).to_le_bytes();"),
            );
            ln(out,format!("writer.write_ptr_unchecked(__qsl_rank.as_ptr(),{bytes}); writer.write_bytes_unchecked(__qsl_data);"));
        }
        _ => return fast_type_write(ty, out, value, width, compact),
    }
    Ok(())
}
fn emit_planned_value(
    ty: &QslType,
    out: &mut String,
    name: &str,
    width: usize,
    compact: bool,
    borrowed: bool,
    aligned: bool,
) -> QslResult<()> {
    if !compact || !aligned {
        return emit_unchecked_value(ty, out, name, width, compact, borrowed);
    }
    match ty {
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::Flags(_) => {
            emit_aligned_rank_read(out, "__qsl_rank", width);
            let primitive = ty.primitive()?.ok_or(QslError::InvalidSchema)?;
            let base = min_literal(ty.rank_base());
            ln(
                out,
                format!(
                    "let {name}=(__qsl_rank as {primitive}).wrapping_add({base} as {primitive});"
                ),
            );
        }
        QslType::String(_)
        | QslType::RangedString { .. }
        | QslType::BinaryString { .. }
        | QslType::GuardString { .. }
        | QslType::CopyGuardString { .. }
        | QslType::Array(_, _)
        | QslType::RangedArray { .. } => {
            emit_aligned_rank_read(out, "__qsl_rank", width);
            let min = ty.rank_base();
            ln(out,format!("let __qsl_len=__qsl_rank as usize+{min}; let __qsl_ptr=reader.read_exact(__qsl_len);"));
            let bytes = "std::slice::from_raw_parts(__qsl_ptr,__qsl_len)";
            let value = if matches!(ty, QslType::Array(_, _) | QslType::RangedArray { .. }) {
                if borrowed {
                    bytes.to_owned()
                } else {
                    format!("{bytes}.to_vec()")
                }
            } else if matches!(ty, QslType::BinaryString { .. }) {
                bytes.to_owned()
            } else if matches!(ty, QslType::CopyGuardString { .. }) {
                format!("::qsl::Utf8CopyGuard::from_bytes_unchecked({bytes})")
            } else if matches!(ty, QslType::GuardString { .. }) {
                format!("::qsl::Utf8Guard::from_bytes_unchecked({bytes})")
            } else if borrowed {
                format!("std::str::from_utf8_unchecked({bytes})")
            } else {
                format!("String::from_utf8_unchecked({bytes}.to_vec())")
            };
            ln(out, format!("let {name}={value};"));
        }
        _ => return emit_unchecked_value(ty, out, name, width, compact, borrowed),
    }
    Ok(())
}

fn emit_unchecked_dynamic_read(
    schema: &QslSchema,
    out: &mut String,
    plan: &QslIoPlan,
) -> QslResult<()> {
    ln(
        out,
        format!(
            "let mut reader = {}::new(bytes);",
            plan.blocks[0].reader.reader()
        ),
    );
    let optional: Vec<_> = schema.fields.iter().filter(|f| f.optional).collect();
    if schema.bitmap() {
        for index in 0..schema.bitmap_words() {
            ln(out, format!("let __qsl_mask_{index} = reader.read_u32();"));
        }
    } else if optional.len() == 1 {
        ln(out, "let __qsl_present = [reader.read_bits(1) != 0];");
        if !schema.compact() {
            ln(out, "reader.align();");
        }
    } else if optional.len() > 1 {
        ln(
            out,
            format!(
                "let __qsl_count = reader.read_bits({}) as usize;",
                schema.count_bits()
            ),
        );
        let width = schema.tag_bits();
        let mask = (1u64 << width) - 1;
        ln(
            out,
            format!("let __qsl_tags = reader.read_bits((__qsl_count * {width}) as u32);"),
        );
        if optional.len() >= 4 && schema.max_tag <= 64 {
            let word = native_word(schema.max_tag.div_ceil(8));
            if optional.iter().all(|field| field.tag != 1) {
                // Zero extension supplies tag zero in unused slots. This tag is
                // never queried here, so no runtime count checks are needed.
                let terms = (0..optional.len())
                    .map(|slot| {
                        format!(
                            "(1{word} << (((__qsl_tags >> {}) & {mask}) as u32))",
                            slot * width
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                ln(out, format!("let __qsl_presence:{word}={terms};"));
            } else {
                ln(out, format!("let mut __qsl_presence:{word}=0; let mut __qsl_tags_left=__qsl_tags; let mut __qsl_left=__qsl_count;"));
                ln(out, format!("while __qsl_left != 0 {{ __qsl_presence |= 1{word} << ((__qsl_tags_left & {mask}) as u32); __qsl_tags_left >>= {width}; __qsl_left -= 1; }}"));
            }
            ln(out, "let __qsl_present = [");
            for field in &optional {
                ln(
                    out,
                    format!("(__qsl_presence & (1{word} << {})) != 0,", field.tag - 1),
                );
            }
            ln(out, "]; ");
        } else {
            ln(out, "let __qsl_present = [");
            for field in &optional {
                let expr = (0..optional.len())
                    .map(|slot| {
                        format!(
                        "((__qsl_count > {slot}) & (((__qsl_tags >> {}) & {mask}u64) == {}u64))",
                        slot * width,
                        field.tag - 1
                    )
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                ln(out, format!("{expr},"));
            }
            ln(out, "]; ");
        }
        if !schema.compact() {
            ln(out, "reader.align();");
        }
    }
    let mut optional_ordinal = 0usize;
    let mut __qsl_block = 0;
    for (i, field) in schema.fields.iter().enumerate() {
        let block = plan.field_blocks[i];
        if block != __qsl_block {
            ln(
                out,
                format!(
                    "let mut reader={}::new(reader.as_ptr());",
                    plan.blocks[block].reader.reader()
                ),
            );
            __qsl_block = block;
        }
        let name = format!("__qsl_field_{i}");
        if field.optional {
            let present = emit_presence(schema, if schema.bitmap() { i } else { optional_ordinal });
            optional_ordinal += 1;
            ln(out, format!("let {name} = if {present} {{"));
            emit_planned_value(
                &field.data,
                out,
                "__qsl_value",
                field.data.rank_width(field.bits)?,
                schema.compact(),
                schema.borrow_strings(),
                plan.aligned_ranks[i],
            )?;
            ln(out, "Some(__qsl_value)");
            ln(out, "} else { None };");
        } else {
            emit_planned_value(
                &field.data,
                out,
                &name,
                field.data.rank_width(field.bits)?,
                schema.compact(),
                schema.borrow_strings(),
                plan.aligned_ranks[i],
            )?;
        }
        if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "reader.align();");
        }
    }
    ln(out, "Self {");
    for (i, field) in schema.fields.iter().enumerate() {
        ln(out, format!("{}: __qsl_field_{i},", field.name));
    }
    ln(out, "}");
    Ok(())
}

/// Emit an append-only optional directory or an EOF-inferred single tail.
fn emit_sequential_write(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    ln(out, "let mut writer = ::qsl::QslByteWriter::from_raw(to);");
    for f in schema.fields.iter().filter(|f| !f.optional) {
        fast_type_write(&f.data, out, &format!("&self.{}", f.name), f.bits, false)?;
    }
    if schema.legacy_opt_table() {
        let count = schema.optional_count();
        ln(out, format!("writer.write_bytes_unchecked(&[{count}u8]);"));
        ln(
            out,
            format!("let __qsl_table = writer.claim_bytes({count} * 4);"),
        );
        ln(out, "let __qsl_payload = writer.as_mut_ptr();");
    }
    for (i, f) in schema.fields.iter().filter(|f| f.optional).enumerate() {
        ln(
            out,
            format!("if let Some(__qsl_value) = self.{}.as_ref() {{", f.name),
        );
        if schema.legacy_opt_table() {
            ln(out, format!("let __qsl_offset = 1 + writer.as_mut_ptr().offset_from(__qsl_payload) as u32; std::ptr::write_unaligned(__qsl_table.add({i} * 4).cast::<u32>(), __qsl_offset.to_le());"));
        }
        fast_type_write(&f.data, out, "__qsl_value", f.bits, false)?;
        if schema.legacy_opt_table() {
            ln(out, format!("}} else {{ std::ptr::write_unaligned(__qsl_table.add({i} * 4).cast::<u32>(), 0); }}"));
        } else {
            ln(out, "}");
        }
    }
    if needs_written_length(schema) {
        ln(out, "writer.as_mut_ptr().offset_from(to) as usize");
    }
    Ok(())
}

fn emit_sequential_read(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    if schema.predict_seq() {
        ln(
            out,
            "#[inline(always)] unsafe fn __qsl_read_sized(bytes:*const u8,len:usize)->Self {",
        );
    } else {
        ln(
            out,
            "#[inline(always)] unsafe fn __qsl_read_unchecked(bytes:*const u8)->Self {",
        );
    }
    ln(out, "let mut reader = ::qsl::QslByteReader::new(bytes);");
    for (i, f) in schema
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| !f.optional)
    {
        emit_unchecked_value(
            &f.data,
            out,
            &format!("__qsl_field_{i}"),
            f.bits,
            false,
            schema.borrow_strings(),
        )?;
    }
    if schema.legacy_opt_table() {
        ln(out, "let __qsl_count = reader.read_u8() as usize;");
        ln(out, "let __qsl_table = reader.read_exact(__qsl_count * 4); let __qsl_payload = reader.as_ptr();");
    }
    for (ordinal, (i, f)) in schema
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| f.optional)
        .enumerate()
    {
        if schema.predict_seq() {
            ln(out, format!("let __qsl_field_{i} = if (reader.as_ptr().offset_from(bytes) as usize) < len {{"));
        } else {
            ln(out, format!("let __qsl_offset = if __qsl_count > {ordinal} {{ u32::from_le(std::ptr::read_unaligned(__qsl_table.add({ordinal} * 4).cast::<u32>())) }} else {{0}};"));
            ln(
                out,
                format!("let __qsl_field_{i} = if __qsl_offset != 0 {{"),
            );
            ln(out, "let mut reader = ::qsl::QslByteReader::new(__qsl_payload.add(__qsl_offset as usize - 1));");
        }
        emit_unchecked_value(
            &f.data,
            out,
            "__qsl_value",
            f.bits,
            false,
            schema.borrow_strings(),
        )?;
        ln(out, "Some(__qsl_value) } else {None};");
    }
    ln(out, "Self {");
    for (i, f) in schema.fields.iter().enumerate() {
        ln(out, format!("{}:__qsl_field_{i},", f.name));
    }
    ln(out, "} }");
    Ok(())
}

fn emit_fast_read(schema: &QslSchema, out: &mut String, plan: &QslIoPlan) -> QslResult<()> {
    if schema.opt_table() && !schema.state_bits() {
        return static_optional_codegen::read(schema, out);
    }
    if schema.state_bits() || schema.predict_table() {
        return qps_codegen::read(schema, out);
    }
    if !unchecked_read_supported(schema) {
        return Ok(());
    }
    if schema.sequential() {
        return emit_sequential_read(schema, out);
    }
    ln(out, "#[inline(always)]");
    ln(
        out,
        "unsafe fn __qsl_read_unchecked(bytes: *const u8) -> Self {",
    );
    if fixed_flat_layout(schema) {
        emit_flat_read(schema, out)?;
    } else if compact_flat_layout(schema) {
        emit_compact_flat_read(schema, out)?;
    } else {
        emit_unchecked_dynamic_read(schema, out, plan)?;
    }
    ln(out, "}");
    Ok(())
}

fn emit_fast_header(schema: &QslSchema, out: &mut String) {
    if schema.bitmap() {
        for fields in schema.fields.chunks(32) {
            let mut mask_expr = String::from("0u32");
            for (index, f) in fields.iter().enumerate() {
                let shift = 31 - index;
                if f.optional {
                    write!(
                        mask_expr,
                        " | ((self.{}.is_some() as u32) << {shift})",
                        f.name
                    )
                    .unwrap();
                } else {
                    write!(mask_expr, " | (1u32 << {shift})").unwrap();
                }
            }
            ln(
                out,
                format!("writer.write_bytes_unchecked(&({mask_expr}).to_le_bytes());"),
            );
        }
    } else if schema.optional_count() == 1 {
        let field = schema.fields.iter().find(|f| f.optional).unwrap();
        ln(
            out,
            format!(
                "writer.write_bits_unchecked(self.{}.is_some() as u64, 1);",
                field.name
            ),
        );
        if !schema.compact() {
            ln(out, "writer.flush_unchecked();");
        }
    } else if schema.optional_count() > 1 {
        let mut count = String::from("0u64");
        for f in schema.fields.iter().filter(|f| f.optional) {
            write!(count, " + (self.{}.is_some() as u64)", f.name).unwrap();
        }
        ln(
            out,
            format!(
                "writer.write_bits_unchecked({count}, {});",
                schema.count_bits()
            ),
        );
        for f in schema.fields.iter().filter(|f| f.optional) {
            ln(
                out,
                format!(
                    "writer.write_bits_unchecked({}, {} * (self.{}.is_some() as u32));",
                    f.tag - 1,
                    schema.tag_bits(),
                    f.name
                ),
            );
        }
        if !schema.compact() {
            ln(out, "writer.flush_unchecked();");
        }
    }
}

fn emit_fast_write(schema: &QslSchema, out: &mut String, plan: &QslIoPlan) -> QslResult<()> {
    ln(out, "#[inline(always)]");
    ln(
        out,
        if needs_written_length(schema) {
            "unsafe fn __qsl_write_unchecked(&self,to:*mut u8)->usize {"
        } else {
            "unsafe fn __qsl_write_unchecked(&self,to:*mut u8) {"
        },
    );
    if schema.state_bits() || schema.predict_table() {
        qps_codegen::write(schema, out)?;
    } else if schema.opt_table() {
        static_optional_codegen::write(schema, out)?;
    } else if schema.sequential() {
        emit_sequential_write(schema, out)?;
    } else if fixed_flat_layout(schema) {
        emit_flat_write(schema, out)?;
    } else if compact_flat_layout(schema) {
        emit_compact_flat_write(schema, out)?;
    } else if optional_flat_layout(schema) {
        emit_optional_flat_write(schema, out)?;
    } else {
        ln(
            out,
            format!(
                "let mut writer = {}::from_raw(to);",
                plan.blocks[0].writer.writer()
            ),
        );
        emit_fast_header(schema, out);
        let mut __qsl_block = 0;
        for (i, f) in schema.fields.iter().enumerate() {
            let block = plan.field_blocks[i];
            if block != __qsl_block {
                ln(
                    out,
                    format!(
                        "let mut writer={}::from_raw(writer.as_mut_ptr());",
                        plan.blocks[block].writer.writer()
                    ),
                );
                __qsl_block = block;
            }
            let width = f.data.rank_width(f.bits)?;
            if f.optional {
                if matches!(
                    &f.data,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::Flags(_)
                ) {
                    let primitive = f.data.primitive()?.ok_or(QslError::InvalidSchema)?;
                    ln(
                        out,
                        format!(
                            "let __qsl_opt_{i}: {primitive} = self.{}.unwrap_or(0);",
                            f.name
                        ),
                    );
                    if schema.compact() && plan.aligned_ranks[i] {
                        let word = native_word(width.div_ceil(8));
                        let base = min_literal(f.data.rank_base());
                        let bytes = width / 8;
                        ln(out,format!("let __qsl_le_{i}=((__qsl_opt_{i} as {word}).wrapping_sub({base} as {word})).to_le_bytes();"));
                        ln(out,format!("writer.write_ptr_unchecked(__qsl_le_{i}.as_ptr(), {bytes} * (self.{}.is_some() as usize));",f.name));
                    } else if schema.compact() {
                        ln(out, format!(
                            "writer.write_bits_unchecked(((__qsl_opt_{i} as i128).wrapping_sub({})) as u64, {width} * (self.{}.is_some() as u32));",
                            min_literal(f.data.rank_base()), f.name
                        ));
                    } else {
                        ln(
                            out,
                            format!("let __qsl_le_{i} = __qsl_opt_{i}.to_le_bytes();"),
                        );
                        ln(out, format!(
                            "writer.write_ptr_unchecked(__qsl_le_{i}.as_ptr(), {} * (self.{}.is_some() as usize));",
                            bytes_of(&f.data)?, f.name
                        ));
                    }
                } else {
                    ln(
                        out,
                        format!("if let Some(__qsl_value) = self.{}.as_ref() {{", f.name),
                    );
                    emit_planned_write(
                        &f.data,
                        out,
                        "__qsl_value",
                        width,
                        schema.compact(),
                        plan.aligned_ranks[i],
                    )?;
                    ln(out, "}");
                }
            } else {
                emit_planned_write(
                    &f.data,
                    out,
                    &format!("&self.{}", f.name),
                    width,
                    schema.compact(),
                    plan.aligned_ranks[i],
                )?;
            }
            if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
                ln(out, "writer.flush_unchecked();");
            }
        }
        ln(out, "writer.flush_unchecked();");
        if needs_written_length(schema) {
            ln(out, "writer.as_mut_ptr().offset_from(to) as usize");
        }
    }
    ln(out, "}");
    Ok(())
}

fn direct_read_layout(schema: &QslSchema) -> bool {
    !schema.compact()
        && schema.optional_count() == 0
        && schema.fields.iter().any(|f| {
            matches!(
                f.data,
                QslType::CompressedString { .. }
                    | QslType::InlineRawString { .. }
                    | QslType::FixedString(_)
                    | QslType::FixedCompressedString(_)
            )
        })
}
fn emit_direct_read(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    ln(
        out,
        "#[inline(always)] unsafe fn __qsl_read_into(bytes:*const u8, out:*mut Self) {",
    );
    ln(out, "let mut reader=::qsl::QslByteReader::new(bytes);");
    if schema.bitmap() {
        ln(
            out,
            format!("reader.read_exact({});", schema.bitmap_words() * 4),
        );
    }
    for (i, f) in schema.fields.iter().enumerate() {
        let dest = format!("std::ptr::addr_of_mut!((*out).{})", f.name);
        match f.data {
            QslType::CompressedString { max, .. } => {
                ln(
                    out,
                    format!("reader.read_inline_string_into_with::<{max},{}>({dest});", string_header_word(max)),
                );
            }
            QslType::InlineRawString { max, .. } => ln(
                out,
                format!("reader.read_raw_inline_string_into_with::<{max},{}>({dest});", string_header_word(max)),
            ),
            QslType::FixedString(n) => ln(out,format!("std::ptr::copy_nonoverlapping(reader.read_exact({n}),{dest}.cast::<u8>(),{n});")),
            QslType::FixedCompressedString(n) => {
                ln(out,format!("reader.align(); let used=::qsl::decode_fixed_utf8::<{n}>(reader.as_ptr(),{dest}.cast()); reader.read_exact(used);"));
            }
            _ => {
                emit_unchecked_value(
                    &f.data,
                    out,
                    &format!("field_{i}"),
                    f.bits,
                    false,
                    schema.borrow_strings(),
                )?;
                ln(out, format!("{dest}.write(field_{i});"));
            }
        }
    }
    ln(out, "}");
    Ok(())
}

fn prepared_read_layout(schema: &QslSchema) -> bool {
    !schema.lifetime()
        && schema.optional_count() == 0
        && schema.fields.iter().any(|f| {
            matches!(
                f.data,
                QslType::CopyGuardString { .. }
                    | QslType::String(_)
                    | QslType::RangedString { .. }
                    | QslType::InlineRawString { .. }
                    | QslType::CompressedString { .. }
                    | QslType::FixedString(_)
                    | QslType::FixedCompressedString(_)
            )
        })
}
fn emit_prepared_read(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let plan = QslIoPlan::analyze(schema)?;
    ln(out, "const REUSES_STORAGE:bool=true;");
    ln(
        out,
        "#[inline(always)] unsafe fn deserialize_reuse(input:&[u8],output:&mut Self) {",
    );
    ln(
        out,
        format!(
            "let mut reader={}::new(input.as_ptr());",
            plan.blocks[0].reader.reader()
        ),
    );
    if schema.bitmap() {
        ln(
            out,
            format!("reader.read_exact({});", schema.bitmap_words() * 4),
        );
    }
    let mut block_index = 0;
    for (i, field) in schema.fields.iter().enumerate() {
        let block = plan.field_blocks[i];
        if block != block_index {
            ln(
                out,
                format!(
                    "let mut reader={}::new(reader.as_ptr());",
                    plan.blocks[block].reader.reader()
                ),
            );
            block_index = block;
        }
        if matches!(
            field.data,
            QslType::CopyGuardString { .. } | QslType::String(_) | QslType::RangedString { .. }
        ) {
            let min = field.data.rank_base();
            let width = field.data.rank_width(field.bits)?;
            if schema.compact() {
                if plan.aligned_ranks[i] {
                    emit_aligned_rank_read(out, "__qsl_rank", width);
                    ln(out, format!("let __qsl_len=__qsl_rank as usize+{min};"));
                } else {
                    ln(
                        out,
                        format!("let __qsl_len=reader.read_bits({width}) as usize+{min};"),
                    );
                }
            } else {
                ln(out, "let __qsl_len=reader.read_u32() as usize;");
            }
            if matches!(field.data, QslType::CopyGuardString { .. }) {
                ln(out,format!("output.{}.copy_from_raw_parts_unchecked(reader.read_exact(__qsl_len),__qsl_len);",field.name));
            } else {
                ln(out,format!("let target=output.{}.as_mut_vec(); debug_assert!(__qsl_len<=target.capacity()); std::ptr::copy_nonoverlapping(reader.read_exact(__qsl_len),target.as_mut_ptr(),__qsl_len); target.set_len(__qsl_len);",field.name));
            }
        } else if let QslType::InlineRawString { max, .. } = field.data {
            ln(out,format!("reader.read_raw_inline_string_into_with::<{max},{}>(std::ptr::addr_of_mut!(output.{}));",string_header_word(max),field.name));
        } else if let QslType::CompressedString { max, .. } = field.data {
            ln(out,format!("reader.read_inline_string_into_with::<{max},{}>(std::ptr::addr_of_mut!(output.{}));",string_header_word(max),field.name));
        } else if let QslType::FixedString(n) = field.data {
            ln(out,format!("std::ptr::copy_nonoverlapping(reader.read_exact({n}),std::ptr::addr_of_mut!(output.{}).cast::<u8>(),{n});",field.name));
        } else if let QslType::FixedCompressedString(n) = field.data {
            ln(out,format!("reader.align(); let used=::qsl::decode_fixed_utf8::<{n}>(reader.as_ptr(),std::ptr::addr_of_mut!(output.{}).cast::<u8>()); reader.read_exact(used);",field.name));
        } else {
            emit_planned_value(
                &field.data,
                out,
                "__qsl_value",
                field.data.rank_width(field.bits)?,
                schema.compact(),
                false,
                plan.aligned_ranks[i],
            )?;
            ln(out, format!("output.{}=__qsl_value;", field.name));
        }
        if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "reader.align();");
        }
    }
    ln(out, "}");
    Ok(())
}

fn emit_trait(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let borrowing = schema.lifetime();
    let size = minimum_size_expr(schema)?;
    let trait_name = if borrowing {
        "::qsl::BorrowSerializable<'a>"
    } else {
        "::qsl::Serializable"
    };
    let ty_name = if borrowing {
        format!("{}<'a>", schema.name)
    } else {
        schema.name.clone()
    };
    ln(out, "#[allow(clippy::identity_op)]");
    if borrowing {
        ln(out, format!("impl<'a> {trait_name} for {ty_name} {{"));
    } else {
        ln(out, format!("impl {trait_name} for {ty_name} {{"));
    }
    ln(
        out,
        format!(
            "const OPTIONAL_FIELDS_COUNT: usize = {};",
            schema.optional_count()
        ),
    );
    ln(
        out,
        format!(
            "const SIZE_TAG: usize = {};",
            if schema.sequential() || schema.bitmap() || schema.optional_count() <= 1 {
                0
            } else {
                schema.tag_bits()
            }
        ),
    );
    ln(
        out,
        format!("#[inline] fn get_size() -> usize {{ {size} }}"),
    );
    if fixed_flat_layout(schema) || compact_flat_layout(schema) {
        ln(
            out,
            format!("#[inline] fn serialized_size(&self) -> usize {{ {size} }}"),
        );
    } else if optional_flat_layout(schema) {
        let optional = schema.fields.iter().find(|f| f.optional).unwrap();
        ln(out, format!("#[inline] fn serialized_size(&self) -> usize {{ {size} + (self.{}.is_some() as usize) * {} }}", optional.name, bytes_of(&optional.data)?));
    } else {
        ln(
            out,
            "#[inline] fn serialized_size(&self) -> usize { self.__qsl_size() }",
        );
    }
    ln(
        out,
        "#[inline] unsafe fn serialize_unsafe(&self, to: *mut u8) { self.__qsl_write_unchecked(to); }",
    );
    if needs_written_length(schema) {
        ln(out,"#[inline(always)] unsafe fn serialize_into_unsafe(&self,to:*mut u8)->usize {self.__qsl_write_unchecked(to)}");
    }
    if direct_read_layout(schema) {
        let method = if borrowing {
            "deserialize_borrowed_into"
        } else {
            "deserialize_into"
        };
        ln(out,format!("#[inline(always)] unsafe fn {method}(bytes:*const u8,_len:usize,out:*mut Self) {{ Self::__qsl_read_into(bytes,out); }}"));
    }
    if prepared_read_layout(schema) {
        emit_prepared_read(schema, out)?;
    }
    if borrowing {
        ln(
            out,
            "#[inline] unsafe fn deserialize_borrowed_unsafe(bytes: *const u8) -> Self {",
        );
    } else {
        ln(
            out,
            "#[inline] unsafe fn deserialize_unsafe(bytes: *const u8) -> Self {",
        );
    }
    if schema.predict_seq() || schema.opt_table() && !schema.state_bits() {
        ln(out, "let _ = bytes; panic!(\"predict_seq needs an input length; use deserialize(slice) or deserialize_from_raw_parts\")");
    } else {
        ln(out, "Self::__qsl_read_unchecked(bytes)");
    }
    ln(out, "}");
    if schema.predict_seq() || schema.opt_table() && !schema.state_bits() {
        let method = if borrowing {
            "deserialize_borrowed_from_raw_parts"
        } else {
            "deserialize_from_raw_parts"
        };
        ln(out, format!("#[inline(always)] unsafe fn {method}(bytes:*const u8,len:usize)->Self {{ Self::__qsl_read_sized(bytes,len) }}"));
    }
    ln(out, "}");
    Ok(())
}

#[derive(Debug, Default)]
pub struct QslInterpreter {
    pub output: Vec<u8>,
    pub config: crate::deq::GenerationConfig,
}

fn contains_custom(ty: &QslType) -> bool {
    match ty {
        QslType::Custom { .. } => true,
        QslType::Array(inner, _)
        | QslType::FixedArray(inner, _)
        | QslType::RangedArray { element: inner, .. } => contains_custom(inner),
        _ => false,
    }
}

fn import_header(schemas: &[QslSchema]) -> &'static str {
    if schemas
        .iter()
        .flat_map(|s| &s.fields)
        .any(|f| contains_custom(&f.data))
    {
        "use ::qsl::QslCustom;\n"
    } else {
        ""
    }
}

impl QslInterpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate(&mut self, schemas: &[QslSchema]) -> QslResult<()> {
        let expanded =
            crate::schema_macro::expand_macros(schemas).map_err(|_| QslError::InvalidSchema)?;
        let schemas = expanded.as_slice();
        self.output.clear();
        let mut src = String::from("// @generated by QSL; do not edit.\n");
        src.push_str(import_header(schemas));
        for schema in schemas {
            schema.validate()?;
            if schema.lifetime() {
                self.generate_lifetime_to(schema, &mut src)?;
            } else {
                self.generate_default_to(schema, &mut src)?;
            }
            emit_qps_bytes(schema, &mut src)?;
        }
        self.output = indent_generated(&src).into_bytes();
        Ok(())
    }

    /// Parser-compatible front end. QSL parser AST stays untouched; only the
    /// internal wire-generation descriptors are normalized here.
    pub fn generate_parsed(&mut self, schemas: &[crate::compat::ParserQslSchema]) -> QslResult<()> {
        let normalized = schemas
            .iter()
            .map(QslSchema::try_from)
            .collect::<QslResult<Vec<_>>>()?;
        self.generate(&normalized)
    }

    pub fn generate_field(&mut self, field: &QslField, borrowed: bool) -> QslResult<()> {
        let mut src = format!("pub {}: ", field.name);
        let ty = field.data.rust_type(borrowed)?;
        if field.optional {
            write!(src, "Option<{ty}>").unwrap();
        } else {
            src.push_str(&ty);
        }
        self.output.extend_from_slice(src.as_bytes());
        Ok(())
    }

    fn generate_struct(schema: &QslSchema, out: &mut String) -> QslResult<()> {
        let suffix = if schema.lifetime() { "<'a>" } else { "" };
        ln(out, format!("pub struct {}{} {{", schema.name, suffix));
        for f in &schema.fields {
            let ty = f.data.rust_type(schema.borrow_strings())?;
            if f.optional {
                ln(out, format!("pub {}: Option<{ty}>,", f.name));
            } else {
                ln(out, format!("pub {}: {ty},", f.name));
            }
        }
        ln(out, "}");
        Ok(())
    }

    fn emit_default_impl(schema: &QslSchema, out: &mut String) -> QslResult<()> {
        if !schema.fields.iter().any(|f| f.default.is_some()) {
            return Ok(());
        }
        // A custom codec cannot be assumed to implement Default. Requiring
        // all custom defaults explicitly would be a separate language feature.
        if schema
            .fields
            .iter()
            .any(|f| matches!(&f.data, QslType::Custom { .. }))
        {
            return Err(QslError::InvalidSchema);
        }
        let ty = if schema.lifetime() {
            format!("{}<'a>", schema.name)
        } else {
            schema.name.clone()
        };
        let lifetime = if schema.lifetime() { "<'a>" } else { "" };
        ln(out, format!("impl{lifetime} Default for {ty} {{"));
        ln(out, "#[inline] fn default() -> Self { Self {");
        for field in &schema.fields {
            let expr = if let Some(default) = field.default {
                let rust_ty = field.data.primitive()?.ok_or(QslError::InvalidSchema)?;
                let val = format!("{default}i128 as {rust_ty}");
                if field.optional {
                    format!("Some({val})")
                } else {
                    val
                }
            } else if field.optional {
                "None".into()
            } else {
                match &field.data {
                    QslType::RangedInteger { min, .. } => {
                        let rust_ty = field.data.primitive()?.ok_or(QslError::InvalidSchema)?;
                        format!("({min}i128 as {rust_ty})")
                    }
                    QslType::String(min) if schema.borrow_strings() && *min != 0 => {
                        return Err(QslError::InvalidSchema)
                    }
                    QslType::String(min) => {
                        format!("\" \".repeat({min})")
                    }
                    QslType::RangedString { min, .. } if schema.borrow_strings() && *min != 0 => {
                        return Err(QslError::InvalidSchema)
                    }
                    QslType::RangedString { min, .. } => format!("\" \".repeat({min})"),
                    QslType::FixedString(n) | QslType::FixedCompressedString(n) => {
                        format!("[b' '; {n}]")
                    }
                    QslType::CopyGuardString { min, .. } => {
                        format!("::qsl::Utf8CopyGuard::copy_of(\"{}\")", " ".repeat(*min))
                    }
                    QslType::GuardString { min, .. } => {
                        format!("::qsl::Utf8Guard::from_str(\"{}\")", " ".repeat(*min))
                    }
                    QslType::BinaryString { min, .. } => {
                        format!("b\"{}\" as &[u8]", " ".repeat(*min))
                    }
                    QslType::InlineRawString { min, max }
                    | QslType::CompressedString { min, max } => {
                        format!("::qsl::InlineString::<{max}>::spaces::<{min}>()")
                    }
                    _ => "Default::default()".into(),
                }
            };
            ln(out, format!("{}: {expr},", field.name));
        }
        ln(out, "} }");
        ln(out, "}");
        Ok(())
    }

    fn generate_impl(&self, schema: &QslSchema, out: &mut String) -> QslResult<()> {
        self.emit_schema_hashes(schema, out)?;
        let generated_start = out.len();
        let quant = crate::payload_quant::PayloadQuantPlan::analyze(schema)?;
        if quant.active {
            payload_quant_codegen::generate(self, schema, &quant, out)?;
            let hint = if schema.flags & crate::FLAG_INLINE_ALWAYS != 0 {
                "#[inline(always)]"
            } else if schema.flags & crate::FLAG_INLINE_NEVER != 0 {
                "#[inline(never)]"
            } else if schema.flags & crate::FLAG_INLINE != 0 {
                "#[inline]"
            } else {
                ""
            };
            let generated = out[generated_start..]
                .replace("#[inline(always)]", hint)
                .replace("#[inline]", hint);
            out.truncate(generated_start);
            out.push_str(&generated);
            return Ok(());
        }
        let plan = QslIoPlan::analyze(schema)?;
        let mut constant_names = std::collections::HashSet::new();
        for f in &schema.fields {
            if string_layout(&f.data).is_some()
                && !constant_names.insert(f.name.to_ascii_uppercase())
            {
                return Err(QslError::InvalidSchema);
            }
        }
        if !unchecked_read_supported(schema) {
            // Arbitrary variable-sized custom arrays do not have a static
            // pointer-advance rule. Reject them during build, not at runtime.
            return Err(QslError::InvalidSchema);
        }
        Self::generate_struct(schema, out)?;
        if schema.opt_table() && !schema.state_bits() {
            static_optional_codegen::tables(schema, out)?;
        } else {
            qps_codegen::tables(schema, out)?;
        }
        if self.config.qsl_rt {
            qps_codegen::region_static(schema, out, self.config.qsl_rt_default)?;
        }
        Self::emit_default_impl(schema, out)?;
        if schema.predict_seq() {
            for f in schema.fields.iter().filter(|f| f.optional) {
                if let QslType::Custom { rust_type, .. } = &f.data {
                    ln(out,format!("const _: () = assert!(<{rust_type} as ::qsl::QslCustom>::SIZE > 0, \"predict_seq cannot represent a zero-byte optional codec\");"));
                }
            }
        }

        // Codegen accepts reference expressions and emits explicit zero bases.
        // Keep stylistic lints local to the generated implementation.
        ln(out, "#[allow(clippy::identity_op, clippy::erasing_op, clippy::deref_addrof, clippy::needless_borrow, clippy::useless_asref, clippy::double_parens, clippy::unnecessary_cast)]");
        if schema.lifetime() {
            ln(out, format!("impl<'a> {}<'a> {{", schema.name));
        } else {
            ln(out, format!("impl {} {{", schema.name));
        }
        if schema.quant_requested() {
            ln(
                out,
                format!(
                    "pub const QUANT_ACTIVE:bool=false;pub const QUANT_FALLBACK:&'static str={:?};",
                    quant.reason.unwrap_or("raw fallback")
                ),
            );
        }
        let registers = |reading: bool| -> QslResult<Vec<usize>> {
            if fixed_flat_layout(schema) || (!reading && optional_flat_layout(schema)) {
                return Ok(vec![0]);
            }
            if compact_flat_layout(schema) {
                let total = schema.fields.iter().try_fold(0usize, |sum, f| {
                    Ok::<_, QslError>(sum + f.data.rank_width(f.bits)?)
                })?;
                return (0..total.div_ceil(64))
                    .map(|word| {
                        Ok(crate::BitRegister::for_bits((total - word * 64).min(64))?.bits())
                    })
                    .collect();
            }
            Ok(plan
                .blocks
                .iter()
                .map(|block| {
                    if reading {
                        block.reader.bits()
                    } else {
                        block.writer.bits()
                    }
                })
                .collect())
        };
        ln(
            out,
            format!(
                "#[allow(dead_code)] pub const BIT_READER_REGISTERS:&'static [usize]=&{:?};",
                registers(true)?
            ),
        );
        ln(
            out,
            format!(
                "#[allow(dead_code)] pub const BIT_WRITER_REGISTERS:&'static [usize]=&{:?};",
                registers(false)?
            ),
        );
        if !schema.lifetime() {
            ln(
                out,
                "/// Decode into raw uninitialized storage or reuse an initialized destination.",
            );
            ln(out, "/// # Safety");
            ln(out,"/// Input is schema-valid and disjoint. Raw destinations are aligned/uninitialized;");
            ln(
                out,
                "/// prepared destinations have capacity for each reused field.",
            );
            ln(out,"#[inline(always)] pub unsafe fn deserialize_into<D: ::qsl::DeserializeDestination<Self>>(bytes:*const u8,len:usize,output:D) { output.decode_into(bytes,len); }");
        }
        for f in &schema.fields {
            if let Some((min, max, header, offset)) = string_layout(&f.data) {
                let name = f.name.to_ascii_uppercase();
                let header_bits =
                    if schema.state_bits() && f.optional && qps_codegen::variable(&f.data) {
                        0
                    } else if schema.compact()
                        && matches!(
                            f.data,
                            QslType::String(_)
                                | QslType::RangedString { .. }
                                | QslType::BinaryString { .. }
                                | QslType::GuardString { .. }
                                | QslType::CopyGuardString { .. }
                        )
                    {
                        f.data.rank_width(f.bits)?
                    } else {
                        header * 8
                    };
                let header = header_bits.div_ceil(8);
                ln(
                    out,
                    format!("#[allow(dead_code)] pub const {name}_MIN_BYTES:usize={min};"),
                );
                if let Some(max) = max {
                    ln(
                        out,
                        format!("#[allow(dead_code)] pub const {name}_MAX_BYTES:usize={max};"),
                    );
                }
                ln(
                    out,
                    format!("#[allow(dead_code)] pub const {name}_HEADER_BYTES:usize={header};"),
                );
                ln(
                    out,
                    format!(
                        "#[allow(dead_code)] pub const {name}_HEADER_BITS:usize={header_bits};"
                    ),
                );
                ln(
                    out,
                    format!("#[allow(dead_code)] pub const {name}_OFFSET_BYTES:usize={offset};"),
                );
            }
        }
        if !(fixed_flat_layout(schema)
            || compact_flat_layout(schema)
            || optional_flat_layout(schema))
        {
            emit_size(schema, out)?;
        }
        qps_codegen::callbacks(schema, out)?;
        if schema.opt_table() && !schema.state_bits() {
            static_optional_codegen::methods(schema, out)?;
        }
        if schema.state_bits() {
            let p = crate::static_index::PredictPlan::analyze(schema)?;
            ln(out,format!("pub const D_WORD_BYTES:usize=4;pub const D_USED_BITS:u32={};pub const Q_TABLE:usize={};",p.used_bits,p.quantum));
        }
        validation_codegen::methods(schema, out)?;
        if self.config.qsl_rt {
            qps_codegen::region_method(schema, out, self.config.qsl_rt_default)?;
        }
        emit_fast_write(schema, out, &plan)?;
        emit_fast_read(schema, out, &plan)?;
        if direct_read_layout(schema) {
            emit_direct_read(schema, out)?;
        }
        if schema.compact() {
            if schema.lifetime() {
                ln(out, "pub fn comp_size(&self) -> usize { <Self as ::qsl::BorrowSerializable<'a>>::serialized_size(self) }");
            } else {
                ln(out, "pub fn comp_size(&self) -> usize { <Self as ::qsl::Serializable>::serialized_size(self) }");
            }
            ln(out, "pub fn comp_write(&self)->Vec<u8> {");
            ln(out, "let mut out=Vec::with_capacity(self.comp_size());");
            let trait_name = if schema.lifetime() {
                "::qsl::BorrowSerializable<'a>"
            } else {
                "::qsl::Serializable"
            };
            ln(out,format!("let used=unsafe {{<Self as {trait_name}>::serialize_into_unsafe(self,out.as_mut_ptr())}};"));
            ln(out, "unsafe {out.set_len(used);} out }");
            ln(out, "/// # Safety\n/// Input must be a complete valid record for this schema, with valid UTF-8 and ranges.");
            if schema.lifetime() {
                ln(out, "pub unsafe fn comp_read(input: &'a [u8]) -> Self {");
                ln(
                    out,
                    "<Self as ::qsl::BorrowSerializable<'a>>::deserialize_borrowed(input)",
                );
            } else {
                ln(out, "pub unsafe fn comp_read(input: &[u8]) -> Self {");
                ln(out, "<Self as ::qsl::Serializable>::deserialize(input)");
            }
            ln(out, "}");
        }
        ln(out, "}");
        emit_trait(schema, out)?;
        let hint = if schema.flags & crate::FLAG_INLINE_ALWAYS != 0 {
            "#[inline(always)]"
        } else if schema.flags & crate::FLAG_INLINE_NEVER != 0 {
            "#[inline(never)]"
        } else if schema.flags & crate::FLAG_INLINE != 0 {
            "#[inline]"
        } else {
            ""
        };
        let generated = out[generated_start..]
            .replace("#[inline(always)]", hint)
            .replace("#[inline]", hint);
        out.truncate(generated_start);
        out.push_str(&generated);
        Ok(())
    }

    fn emit_schema_hashes(&self, schema: &QslSchema, out: &mut String) -> QslResult<()> {
        let h = schema.hashes(self.config.use_hash)?;
        let head = if schema.lifetime() {
            format!("impl<'a> {}<'a>", schema.name)
        } else {
            format!("impl {}", schema.name)
        };
        ln(out, format!("{head} {{"));
        ln(
            out,
            format!(
                "pub const MAX_DATA_SIZE: usize = {};",
                self.config.max_data_size
            ),
        );
        ln(out, format!("pub const SCHEMA_HASHES: ::qsl::SchemaHashes = ::qsl::SchemaHashes {{ version: {}, algorithm: ::qsl::SchemaHashAlgorithm::{}, strict: 0x{}u128, light: 0x{}u128, complete: {} }};", h.version, h.algorithm.rust_variant(), h.strict_hex(), h.light_hex(), h.complete));
        ln(
            out,
            "pub const SCHEMA_HASH_STRICT: u128 = Self::SCHEMA_HASHES.strict;",
        );
        ln(
            out,
            "pub const SCHEMA_HASH_LIGHT: u128 = Self::SCHEMA_HASHES.light;",
        );
        ln(out, "}");
        let trait_head = if schema.lifetime() {
            format!("impl<'a> ::qsl::SchemaIdentity for {}<'a>", schema.name)
        } else {
            format!("impl ::qsl::SchemaIdentity for {}", schema.name)
        };
        ln(out, format!("{trait_head} {{ const SCHEMA_HASHES: ::qsl::SchemaHashes = Self::SCHEMA_HASHES; }}"));
        Ok(())
    }

    fn generate_default_to(&self, schema: &QslSchema, out: &mut String) -> QslResult<()> {
        self.generate_impl(schema, out)
    }
    fn generate_lifetime_to(&self, schema: &QslSchema, out: &mut String) -> QslResult<()> {
        // BorrowSerializable<'a> preserves the backing input memory's lifetime.
        self.generate_impl(schema, out)
    }
    pub fn generate_default(&mut self, schema: &QslSchema) -> QslResult<()> {
        schema.validate()?;
        let mut src = if self.output.is_empty() {
            import_header(std::slice::from_ref(schema)).to_owned()
        } else {
            String::new()
        };
        self.generate_default_to(schema, &mut src)?;
        self.output.extend_from_slice(src.as_bytes());
        Ok(())
    }
    pub fn generate_lifetime(&mut self, schema: &QslSchema) -> QslResult<()> {
        schema.validate()?;
        if !schema.lifetime() {
            return Err(QslError::InvalidSchema);
        }
        let mut src = if self.output.is_empty() {
            import_header(std::slice::from_ref(schema)).to_owned()
        } else {
            String::new()
        };
        self.generate_lifetime_to(schema, &mut src)?;
        self.output.extend_from_slice(src.as_bytes());
        Ok(())
    }
    pub fn push(&self) -> std::io::Result<()> {
        let out_path = std::env::var("OUT_DIR")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
        let dest_path = std::path::Path::new(&out_path).join("qsl_gen.rs");
        fs::write(&dest_path, &self.output)?;
        println!("cargo:rustc-env=qsl_gen_path={}", dest_path.display());
        println!("cargo:rerun-if-changed=build.rs");
        Ok(())
    }
}

/// The outer byte codec is explicitly selected. It never changes an existing
/// Serializable wire format or infers optional presence. Safety schemas supply
/// the validation proof needed before trusted restoration of owned/borrowed fields.
fn emit_qps_bytes(s: &QslSchema, out: &mut String) -> QslResult<()> {
    let name = &s.name;
    if s.lifetime() {
        ln(out,format!("unsafe impl<'a> ::qsl::qps_bytes::CheckedBorrowedRecord<'a> for {name}<'a> {{const MAX_DATA_SIZE:usize=Self::MAX_DATA_SIZE;fn qps_validate_values(&self)->::qsl::QslResult<()>{{self.validate_values()}}fn qps_validate_wire(input:&[u8])->::qsl::QslResult<()>{{Self::validate(input)}}}}"));
        if s.qps() {
            let q = s.quantum();
            ln(out,format!("impl<'a> {name}<'a> {{pub fn serialize_qps_to<'out>(&self,scratch:&mut[u8],out:&'out mut[u8])->::qsl::QslResult<::qsl::qps_bytes::ByteFrame<'out,{q}>>{{::qsl::qps_bytes::ByteQps::<{q}>::encode_borrowed_to(self,scratch,out)}}pub fn deserialize_qps(input:&[u8],restoration:&'a mut[u8])->::qsl::QslResult<Self>{{::qsl::qps_bytes::ByteQps::<{q}>::decode_borrowed(input,restoration)}}}}"));
        }
    } else {
        ln(out,format!("unsafe impl ::qsl::qps_bytes::CheckedRecord for {name} {{const MAX_DATA_SIZE:usize=Self::MAX_DATA_SIZE;fn qps_validate_values(&self)->::qsl::QslResult<()>{{self.validate_values()}}fn qps_validate_wire(input:&[u8])->::qsl::QslResult<()>{{Self::validate(input)}}}}"));
        if s.qps() {
            let q = s.quantum();
            ln(out,format!("impl {name} {{pub fn serialize_qps_to<'out>(&self,scratch:&mut[u8],out:&'out mut[u8])->::qsl::QslResult<::qsl::qps_bytes::ByteFrame<'out,{q}>>{{::qsl::qps_bytes::ByteQps::<{q}>::encode_record_to(self,scratch,out)}}pub fn deserialize_qps_into<'out>(input:&[u8],restoration:&mut[u8],out:&'out mut std::mem::MaybeUninit<Self>)->::qsl::QslResult<&'out mut Self>{{::qsl::qps_bytes::ByteQps::<{q}>::decode_record_into(input,restoration,out)}}}}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_api_is_universal_and_optimization_options_remain_explicit() {
        let s = crate::parse_qsl("&Legacy{u16 required;u8? optional;}")
            .unwrap()
            .remove(0);
        let generated = source(s);
        for marker in ["_OFFSET_TABLE", "_RT_REGIONS", "#[inline"] {
            assert!(!generated.contains(marker), "{marker}");
        }
        assert!(generated.contains("pub fn validate("));
        assert!(generated.contains("serialize_checked_to"));
        assert!(generated.contains("deserialize_checked_into"));
        let s = crate::parse_qsl("&Checked[safety,inline=never]{u16 required;u8? optional;}")
            .unwrap()
            .remove(0);
        let generated = source(s);
        assert!(generated.contains("pub fn validate("));
        assert!(generated.contains("#[inline(never)]"));
        assert!(crate::parse_qsl("&Strict{u8? optional;u16 required;}").is_err());
    }
    use crate::types::{FLAG_COMPACT, FLAG_LIFETIME, FLAG_OPTIONAL_BITMAP};

    fn field(name: &str, optional: bool, tag: usize, bits: usize, data: QslType) -> QslField {
        QslField {
            name: name.to_owned(),
            optional,
            tag,
            bits,
            data,
            default: None,
        }
    }
    fn sample(flags: usize) -> QslSchema {
        QslSchema {
            name: "Person".into(),
            max_tag: 8,
            flags,
            fields: vec![
                field(
                    "id",
                    false,
                    1,
                    16,
                    QslType::Integer {
                        bits: 32,
                        signed: false,
                        min: 0,
                    },
                ),
                field("name", true, 3, 6, QslType::String(0)),
                field("city", true, 7, 6, QslType::String(0)),
            ],
        }
    }
    fn source(schema: QslSchema) -> String {
        let mut generator = QslInterpreter::new();
        generator.generate(&[schema]).unwrap();
        String::from_utf8(generator.output).unwrap()
    }
    #[test]
    fn native_string_headers_are_selected_while_generating() {
        for (max, word) in [(127, "u8"), (128, "u16"), (32767, "u16"), (32768, "u32")] {
            let schema = crate::parse_qsl(&format!("&A{{str(0..{max}) value|decompress=true|;}}"))
                .unwrap()
                .remove(0);
            let src = source(schema);
            assert!(src.contains(&format!("write_inline_string_with::<{max},{word}>")));
            assert!(src.contains(&format!("read_inline_string_with::<{max},{word}>")));
            assert!(!src.contains("#[inline(always)]"));
            assert!(src.contains("unsafe fn deserialize_unsafe"));
        }
    }

    #[test]
    fn required_fields_never_have_tags() {
        let src = source(sample(FLAG_COMPACT));
        assert!(src.contains("writer.write_bits_unchecked(2, 3 *"));
        assert!(src.contains("writer.write_bits_unchecked(6, 3 *"));
        assert!(!src.contains("writer.write_bits_unchecked(0, 3 *"));
        assert!(src.contains("let __qsl_field_0"));
        assert!(src.contains("let __qsl_field_1"));
    }
    #[test]
    fn optional_bitmap_uses_msb_first_u32() {
        let src = source(sample(FLAG_OPTIONAL_BITMAP));
        assert!(src.contains("(1u32 << 31)"));
        assert!(src.contains("<< 30"));
        assert!(src.contains("<< 29"));
        assert!(src.contains("let __qsl_mask_0 = reader.read_u32();"));
        assert!(src.contains("const SIZE_TAG: usize = 0;"));
        assert!(src.contains("fn get_size() -> usize { 8 }"));
        assert!(!src.contains("pub fn comp_write"));
        assert!(!src.contains("pub fn comp_read"));
    }
    #[test]
    fn tags_use_three_bits_when_max_tag_is_eight() {
        let src = source(sample(FLAG_COMPACT));
        assert!(src.contains("const SIZE_TAG: usize = 3;"));
        assert!(src.contains("writer.write_bits_unchecked(0u64 +"));
        assert!(src.contains("let __qsl_tags = reader.read_bits((__qsl_count * 3) as u32);"));
        assert!(src.contains("pub unsafe fn comp_read"));
    }
    #[test]
    fn borrowed_generation_keeps_string_in_input_buffer() {
        let src = source(sample(FLAG_LIFETIME));
        assert!(src.contains("pub struct Person<'a>"));
        assert!(src.contains("pub name: Option<&'a str>"));
        assert!(src.contains("BorrowSerializable<'a>"));
        assert!(src.contains(
            "std::str::from_utf8_unchecked(std::slice::from_raw_parts(__qsl_ptr, __qsl_len))"
        ));
        assert!(!src.contains("String::from_utf8"));
    }
    #[test]
    fn no_duplicate_optional_tags() {
        let mut schema = sample(0);
        schema.fields[2].tag = schema.fields[1].tag;
        assert_eq!(schema.validate().unwrap_err(), QslError::InvalidSchema);
    }
    #[test]
    fn fixed_lengths_are_constant_folded() {
        let schema = QslSchema {
            name: "Packed".into(),
            max_tag: 8,
            flags: 0,
            fields: vec![
                field(
                    "a",
                    false,
                    1,
                    0,
                    QslType::Integer {
                        bits: 16,
                        signed: false,
                        min: 0,
                    },
                ),
                field(
                    "b",
                    false,
                    2,
                    0,
                    QslType::Integer {
                        bits: 24,
                        signed: false,
                        min: 0,
                    },
                ),
                field(
                    "c",
                    false,
                    3,
                    0,
                    QslType::Integer {
                        bits: 32,
                        signed: false,
                        min: 0,
                    },
                ),
            ],
        };
        // Only 8/16/32/64 are valid numeric widths, so use byte padding 3.
        let mut schema = schema;
        schema.fields[1].data = QslType::Padding(3);
        let src = source(schema);
        assert!(src.contains("fn get_size() -> usize { 9 }"));
        assert!(src.contains("fn serialized_size(&self) -> usize { 9 }"));
        assert!(!src.contains("fn __qsl_size(&self)"));
    }
    #[test]
    fn fixed_normal_layout_generates_literal_offsets() {
        let schema = QslSchema {
            flags: 0,
            name: "Offsets".into(),
            max_tag: 8,
            fields: vec![
                field(
                    "id",
                    false,
                    1,
                    0,
                    QslType::Integer {
                        bits: 32,
                        signed: false,
                        min: 0,
                    },
                ),
                field(
                    "age",
                    false,
                    2,
                    0,
                    QslType::Integer {
                        bits: 16,
                        signed: false,
                        min: 0,
                    },
                ),
            ],
        };
        let src = source(schema);
        assert!(src.contains("write_unaligned(to.add(0) as *mut u32"));
        assert!(src.contains("write_unaligned(to.add(4) as *mut u16"));
        assert!(src.contains("fn get_size() -> usize { 6 }"));
    }
    #[test]
    fn fixed_compact_layout_precomputes_packing() {
        let schema = QslSchema {
            flags: FLAG_COMPACT,
            name: "Bits".into(),
            max_tag: 8,
            fields: vec![
                field("a", false, 1, 3, QslType::Byte),
                field("b", false, 2, 5, QslType::Byte),
            ],
        };
        let src = source(schema);
        assert!(src.contains("let __qsl_word_0 ="));
        assert!(src.contains("copy_nonoverlapping(__qsl_word_0.as_ptr(), to.add(0), 1)"));
        assert!(!src.contains("let mut writer = ::qsl::QslUncheckedWriter::from_raw(to);"));
    }

    #[test]
    fn generator_emits_default_only_for_explicit_numeric_attribute() {
        let schema = QslSchema {
            flags: 0,
            name: "OptDefault".into(),
            max_tag: 1,
            fields: vec![QslField {
                name: "x".into(),
                optional: true,
                bits: 8,
                data: QslType::Byte,
                tag: 1,
                default: Some(9),
            }],
        };
        let src = source(schema);
        assert!(src.contains("impl Default for OptDefault"));
        assert!(src.contains("x: Some(9i128 as u8)"));
        assert!(src.contains("const SIZE_TAG: usize = 0;"));
        assert!(src.contains("std::ptr::write(to, __qsl_presence as u8);"));
        assert!(!src.contains("writer.write_bits_unchecked(0, 0 *"));
    }
}
