use super::*;
use crate::static_index::PredictPlan;

/// Finite wire upper bound. An unbounded string/array or unknown custom size
/// cannot receive static RT storage. This is a build-time calculation only.
pub(crate) fn maximum_size(schema: &QslSchema) -> QslResult<Option<usize>> {
    let quant = crate::payload_quant::PayloadQuantPlan::analyze(schema)?;
    if quant.active {
        return Ok(Some(quant.max_bytes));
    }
    let mut s = QslSizer::default();
    let graph_raw = schema.predicted_opt_table()
        && matches!(
            crate::static_optional::StaticOptionalPlan::analyze(schema)?.presence,
            crate::static_optional::StaticPresence::Graph(_)
        );
    if schema.predicted_opt_table() {
        s.bytes(crate::static_optional::StaticOptionalPlan::analyze(schema)?.header_bytes())?;
    } else if schema.state_bits() {
        s.bytes(4)?;
    } else if schema.predict_table() || schema.predict_seq() {
    } else if schema.legacy_opt_table() {
        s.bytes(1 + 4 * schema.optional_count())?;
    } else if schema.bitmap() {
        s.bytes(4 * schema.bitmap_words())?;
    } else if schema.optional_count() == 1 {
        s.bits(1)?;
    } else if schema.optional_count() > 1 {
        s.bits(schema.count_bits() + schema.optional_count() * schema.tag_bits())?;
    }
    if !schema.compact() {
        s.align()?;
    }
    for f in &schema.fields {
        let raw_qps = (schema.state_bits() || graph_raw) && f.optional && variable(&f.data);
        if let Some((_, max, header, _)) = string_layout(&f.data) {
            let Some(max) = max else { return Ok(None) };
            if matches!(
                f.data,
                QslType::FixedString(_) | QslType::FixedCompressedString(_)
            ) || raw_qps
            {
                s.bytes(max)?;
            } else if schema.compact()
                && matches!(
                    f.data,
                    QslType::RangedString { .. }
                        | QslType::BinaryString { .. }
                        | QslType::GuardString { .. }
                        | QslType::CopyGuardString { .. }
                )
            {
                s.bits(f.data.rank_width(f.bits)?)?;
                s.bytes(max)?;
            } else {
                s.bytes(header + max)?;
            }
        } else {
            match &f.data {
                QslType::Array(_, _) => return Ok(None),
                QslType::RangedArray { max, .. } => {
                    if schema.compact() {
                        s.bits(f.data.rank_width(f.bits)?)?;
                    } else {
                        s.bytes(4)?;
                    }
                    s.bytes(*max)?;
                }
                QslType::Custom {
                    fixed_wire_size: None,
                    ..
                } => return Ok(None),
                _ => {
                    if schema.compact()
                        && matches!(
                            f.data,
                            QslType::Byte
                                | QslType::Integer { .. }
                                | QslType::RangedInteger { .. }
                                | QslType::Flags(_)
                        )
                    {
                        s.bits(f.data.rank_width(f.bits)?)?;
                    } else {
                        s.bytes(bytes_of(&f.data)?)?;
                    }
                }
            }
        }
        if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
            s.align()?;
        }
    }
    Ok(Some(s.finish()?))
}
pub(super) fn region_static(schema: &QslSchema, out: &mut String, slots: usize) -> QslResult<()> {
    if let Some(cap) = maximum_size(schema)? {
        if cap.checked_mul(slots).is_none_or(|n| n > 64 * 1024 * 1024) {
            return Err(QslError::InvalidSchema);
        }
        ln(out,format!("#[allow(non_upper_case_globals)] static {}_RT_REGIONS: ::qsl::rt::StaticRegions<{cap},{slots}>=::qsl::rt::StaticRegions::new();",schema.name));
    }
    Ok(())
}
pub(super) fn region_method(schema: &QslSchema, out: &mut String, slots: usize) -> QslResult<()> {
    if let Some(cap) = maximum_size(schema)? {
        let target = if schema.lifetime() {
            "::qsl::BorrowSerializable<'a>"
        } else {
            "::qsl::Serializable"
        };
        ln(out,format!("/// Encode into an exclusive static region; no heap allocation.\n/// A live lease keeps the bytes stable. Exhaustion returns PoolExhausted.\npub fn serialize(&self)->::qsl::QslResult<::qsl::rt::RegionLease<{cap},{slots}>>{{self.validate_values()?;let mut lease={}_RT_REGIONS.try_acquire()?;unsafe{{let n=<Self as {target}>::serialize_into_unsafe(self,lease.as_mut_ptr());::qsl::validation::check_size(n,Self::MAX_DATA_SIZE)?;lease.set_len(n);}}Ok(lease)}}",schema.name));
    }
    Ok(())
}

pub(super) fn tables(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    if !(schema.state_bits() || schema.predict_table() || schema.legacy_opt_table()) {
        return callback_table(schema, out);
    }
    let name = &schema.name;
    if schema.state_bits() || schema.predict_table() {
        let plan = PredictPlan::analyze(schema)?;
        if let Some(reason) = plan.reason {
            ln(out, format!("// Predict plan: {reason}."));
        }
        // Descriptors are exact even where length-only inference is impossible.
        let desc: Vec<_> = plan
            .fields
            .iter()
            .map(|f| (f.shift, f.bits, f.min, f.max))
            .collect();
        ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_STATE_TABLE:[(u32,u32,usize,usize);{}]={desc:?};",desc.len()));
        let rows = plan
            .rows
            .iter()
            .map(|r| format!("({}, {}, {:?})", r.len, r.mask, r.offsets))
            .collect::<Vec<_>>()
            .join(",");
        if plan.rows.is_empty() {
            ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_OFFSET_TABLE:[(u32,u32,usize,usize);{}]={desc:?};",desc.len()));
        } else {
            ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_OFFSET_TABLE:[(u32,u32,[u32;{}]);{}]=[{rows}];",plan.fields.len(),plan.rows.len()));
        }
        let index = plan.packed_index()?;
        ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_INDEX_TABLE:[u32;{}]={index:?};",index.len()));
    } else {
        // Legacy catalogue entries have a statically known byte position. The
        // value in each entry remains a per-message payload-relative offset.
        let offsets: Vec<_> = (0..schema.optional_count()).map(|i| 1 + 4 * i).collect();
        ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_OFFSET_TABLE:[usize;{}]={offsets:?};",offsets.len()));
    }
    callback_table(schema, out)
}
/// Readers are keyed by the actual encoded zero-based tag, not by optional
/// ordinal. Required-field tags deliberately have no callable entry.
pub(super) fn callback_table(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    if !schema.opt_table() {
        return Ok(());
    }
    let name = &schema.name;
    let ty = if schema.lifetime() {
        format!("{name}<'a>")
    } else {
        name.clone()
    };
    let hrtb = if schema.lifetime() { "for<'a> " } else { "" };
    if schema.lifetime() {
        for f in schema.fields.iter().filter(|f| f.optional) {
            ln(out,format!("#[allow(dead_code,non_snake_case)] unsafe fn {name}_opt_{}<'a>(bytes:*const u8,out:*mut {name}<'a>,len:usize)->usize{{{name}::__qsl_read_{}(bytes,out,len)}}",f.name,f.name));
        }
    }
    let entries = (1..=schema.max_tag)
        .map(
            |tag| match schema.fields.iter().find(|f| f.optional && f.tag == tag) {
                Some(f) if schema.lifetime() => format!("Some({name}_opt_{})", f.name),
                Some(f) => format!("Some({name}::__qsl_read_{})", f.name),
                None => "None".into(),
            },
        )
        .collect::<Vec<_>>()
        .join(",");
    ln(out,format!("#[allow(dead_code,non_upper_case_globals,clippy::type_complexity)] static {name}_OPT_TABLE:[Option<{hrtb}unsafe fn(*const u8,*mut {ty},usize)->usize>;{}]=[{entries}];",schema.max_tag));
    Ok(())
}
pub(super) fn callbacks(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    if !schema.opt_table() {
        return Ok(());
    }
    let graph_raw = schema.predicted_opt_table()
        && matches!(
            crate::static_optional::StaticOptionalPlan::analyze(schema)?.presence,
            crate::static_optional::StaticPresence::Graph(_)
        );
    for f in schema.fields.iter().filter(|f| f.optional) {
        ln(out,format!("/// # Safety\n/// Input is a validated field; out points to an initialized, disjoint Self.\n#[allow(dead_code)] unsafe fn __qsl_read_{}(bytes:*const u8,out:*mut Self,len:usize)->usize{{let mut reader=::qsl::QslByteReader::new(bytes);let _=len;",f.name));
        if (schema.state_bits() || graph_raw) && variable(&f.data) {
            raw_read(&f.data, out, "value", "len", schema.borrow_strings())?;
        } else {
            emit_unchecked_value(
                &f.data,
                out,
                "value",
                f.bits,
                false,
                schema.borrow_strings(),
            )?;
        }
        ln(
            out,
            format!(
                "(*out).{}=Some(value);reader.as_ptr().offset_from(bytes) as usize}}",
                f.name
            ),
        );
    }
    Ok(())
}

pub(super) fn variable(ty: &QslType) -> bool {
    matches!(
        ty,
        QslType::String(_)
            | QslType::RangedString { .. }
            | QslType::InlineRawString { .. }
            | QslType::BinaryString { .. }
            | QslType::GuardString { .. }
            | QslType::CopyGuardString { .. }
    )
}
pub(super) fn raw_read(
    ty: &QslType,
    out: &mut String,
    name: &str,
    len: &str,
    borrowed: bool,
) -> QslResult<()> {
    ln(
        out,
        format!("let __bytes=std::slice::from_raw_parts(reader.read_exact({len}),{len});"),
    );
    let expr=match ty {
        QslType::BinaryString{..}=>"__bytes".into(),
        QslType::GuardString{..}=>"::qsl::Utf8Guard::from_bytes_unchecked(__bytes)".into(),
        QslType::CopyGuardString{..}=>"::qsl::Utf8CopyGuard::from_bytes_unchecked(__bytes)".into(),
        QslType::InlineRawString{max,..}=>format!("::qsl::InlineString::<{max}>::from_str_unchecked(std::str::from_utf8_unchecked(__bytes))"),
        _ if borrowed=>"std::str::from_utf8_unchecked(__bytes)".into(),
        _=>"String::from_utf8_unchecked(__bytes.to_vec())".into()
    };
    ln(out, format!("let {name}={expr};"));
    Ok(())
}
pub(super) fn size(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    ln(
        out,
        format!(
            "#[inline(always)] fn __qsl_size(&self)->usize {{ let mut n={}usize;",
            if schema.state_bits() { 4 } else { 0 }
        ),
    );
    for f in &schema.fields {
        if f.optional {
            ln(
                out,
                format!(
                    "if let Some(value)=self.{}.as_ref() {{ let _=value;",
                    f.name
                ),
            );
        }
        let expr = if f.optional {
            "value".into()
        } else {
            format!("&self.{}", f.name)
        };
        if f.optional && variable(&f.data) {
            ln(out, format!("n+=({expr}).len();"));
        } else {
            ln(out, "let mut __qsl_bits=0usize;");
            emit_size_field(&f.data, out, &expr, f.bits, false)?;
            ln(out, "n+=(__qsl_bits+7)>>3;");
        }
        if f.optional {
            ln(out, "}");
        }
    }
    ln(out, "n }");
    Ok(())
}
pub(super) fn write(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let plan = PredictPlan::analyze(schema)?;
    ln(out, "let mut writer=::qsl::QslByteWriter::from_raw(to);");
    if schema.state_bits() {
        ln(out, "let mut d=0u32;");
        for (f, c) in schema
            .fields
            .iter()
            .filter(|f| f.optional)
            .zip(&plan.fields)
        {
            if c.fixed {
                ln(
                    out,
                    format!("d|=(self.{}.is_some() as u32)<<{};", f.name, c.shift),
                );
                continue;
            }
            let code = format!("(value.len()-{}+1) as u32", c.min);
            ln(
                out,
                format!(
                    "if let Some(value)=self.{}.as_ref() {{let _=value;d|=({code})<<{};}}",
                    f.name, c.shift
                ),
            );
        }
        ln(out, "writer.write_bytes_unchecked(&d.to_le_bytes());");
    }
    for f in &schema.fields {
        if f.optional && optional_scalar_write(f, out)? {
            continue;
        }
        if f.optional {
            ln(
                out,
                format!("if let Some(value)=self.{}.as_ref() {{", f.name),
            );
        }
        let expr = if f.optional {
            "value".into()
        } else {
            format!("&self.{}", f.name)
        };
        if f.optional && variable(&f.data) {
            let bytes = if matches!(f.data, QslType::BinaryString { .. }) {
                expr.to_string()
            } else {
                format!("({expr}).as_bytes()")
            };
            ln(out, format!("writer.write_bytes_unchecked({bytes});"));
        } else {
            fast_type_write(&f.data, out, &expr, f.bits, false)?;
        }
        if f.optional {
            ln(out, "}");
        }
    }
    ln(out, "writer.as_mut_ptr().offset_from(to) as usize");
    Ok(())
}
pub(super) fn read(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let plan = PredictPlan::analyze(schema)?;
    let signature = if schema.state_bits() {
        "unsafe fn __qsl_read_unchecked(bytes:*const u8)->Self {"
    } else {
        "unsafe fn __qsl_read_sized(bytes:*const u8,len:usize)->Self {"
    };
    ln(out, "#[inline(always)]");
    ln(out, signature);
    ln(out, "let mut reader=::qsl::QslByteReader::new(bytes);");
    if schema.state_bits() {
        ln(out, "let d=reader.read_u32();");
    }
    for (i, f) in schema
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| !f.optional)
    {
        emit_unchecked_value(
            &f.data,
            out,
            &format!("field_{i}"),
            f.bits,
            false,
            schema.borrow_strings(),
        )?;
    }
    if schema.predict_table() {
        ln(out,format!("let total=len-reader.as_ptr().offset_from(bytes) as usize;let rank=*{}_INDEX_TABLE.get_unchecked(total) as usize;let row={}_OFFSET_TABLE.get_unchecked(rank);let d=row.1;",schema.name,schema.name));
    } else if !plan.rows.is_empty() {
        let sums = plan
            .fields
            .iter()
            .map(|f| format!("(((d>>{})&1) as usize)*{}", f.shift, f.min))
            .collect::<Vec<_>>()
            .join("+");
        let q = plan.quantum;
        if q == 1 {
            ln(out,format!("let total={sums};let rank=*{}_INDEX_TABLE.get_unchecked(total);let row={}_OFFSET_TABLE.get_unchecked(rank as usize);",schema.name,schema.name));
        } else {
            ln(out,format!("let total={sums};let word=*{}_INDEX_TABLE.get_unchecked(total>>{});let r=total&{};let rank=(word>>{})+(word&((1u32<<r)-1)).count_ones();let row={}_OFFSET_TABLE.get_unchecked(rank as usize);",schema.name,q.trailing_zeros(),q-1,q-1,schema.name));
        }
        // Use recovered mask, so the quantized lookup is an actual decoding
        // strategy, not decorative metadata that LLVM immediately discards.
        ln(out, "let d=row.1;");
    }
    if schema.opt_table() {
        tagged_optional_codegen::result(schema, out);
    }
    for ((i, f), c) in schema
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| f.optional)
        .zip(&plan.fields)
    {
        let mask = if c.bits == 32 {
            u32::MAX
        } else {
            (1u32 << c.bits) - 1
        };
        ln(out, format!("let code=(d>>{})&{mask}u32;", c.shift));
        if schema.opt_table() {
            ln(out, "if code!=0{");
            let length = if variable(&f.data) {
                format!("(code as usize-1)+{}", c.min)
            } else {
                "0".into()
            };
            tagged_optional_codegen::dispatch(schema, out, &(f.tag - 1).to_string(), &length);
            ln(out, "}");
            continue;
        }
        if optional_scalar_read(&f.data, out, &format!("field_{i}"), "code!=0")? {
            continue;
        }
        ln(
            out,
            format!("let field_{i}=if ((d>>{})&{mask}u32)!=0 {{", c.shift),
        );
        if variable(&f.data) {
            ln(out, format!("let length=(code as usize-1)+{};", c.min));
            raw_read(&f.data, out, "value", "length", schema.borrow_strings())?;
        } else {
            emit_unchecked_value(
                &f.data,
                out,
                "value",
                f.bits,
                false,
                schema.borrow_strings(),
            )?;
        }
        ln(out, "Some(value)}else{None};");
    }
    if schema.opt_table() {
        ln(out, "result}");
        return Ok(());
    }
    ln(out, "Self {");
    for (i, f) in schema.fields.iter().enumerate() {
        ln(out, format!("{}:field_{i},", f.name));
    }
    ln(out, "} }");
    Ok(())
}
