use super::*;
/// Exact byte-sized custom codecs are not ordinary 32-bit array scalars.
/// Keep this expression in generated validation code so Rust resolves COMP_SIZE.
pub(super) fn compact_element_bits(element: &QslType) -> QslResult<String> {
    if let QslType::Custom { rust_type, .. } = element {
        Ok(format!("<{rust_type} as ::qsl::QslCustom>::COMP_SIZE.checked_mul(8).ok_or(::qsl::QslError::LengthOverflow)?"))
    } else {
        Ok(element.default_compact_bits()?.to_string())
    }
}
pub(super) fn field(ty: &QslType, out: &mut String, width: usize, compact: bool) -> QslResult<()> {
    field_native(ty, out, width, compact, false)
}
pub(super) fn field_stored(
    f: &QslField,
    out: &mut String,
    width: usize,
    compact: bool,
) -> QslResult<()> {
    field_native(
        &f.data,
        out,
        width,
        compact,
        matches!(f.storage, qsl_lexer::layout::StorageKind::FixedArray(_)),
    )
}
fn field_native(
    ty: &QslType,
    out: &mut String,
    width: usize,
    compact: bool,
    native_fixed: bool,
) -> QslResult<()> {
    match ty {
        QslType::DomainInteger(d)=>numeric_codegen::checked(d,out,compact)?,
        QslType::None=>{},
        QslType::BitPadding(n)=>{ln(out,format!("c.charge({})?;",n/8));if *n>=64{ln(out,format!("for _ in 0..{}{{if c.bits(64)?!=0{{return Err(::qsl::QslError::InvalidPadding);}}}}",n/64));}
if !n.is_multiple_of(64){ln(out,format!("if c.bits({})?!=0{{return Err(::qsl::QslError::InvalidPadding);}}",n%64));}},
        QslType::Padding(n)=>ln(out,format!("c.charge({n})?;if c.bytes({n})?.iter().any(|&b|b!=0){{return Err(::qsl::QslError::InvalidPadding);}}")),
        QslType::Byte|QslType::Integer{..}|QslType::RangedInteger{..}|QslType::Flags(_)=>{
            let primitive=ty.primitive()?.ok_or(QslError::InvalidSchema)?;
            ln(out,format!("c.charge({})?;",bytes_of(ty)?));
            if compact && matches!(ty,QslType::RangedInteger{..}) {ln(out,format!("let value=(c.bits({width})? as i128)+{}i128;",ty.rank_base()));}
            else if compact {ln(out,format!("let value=(((c.bits({width})? as i128)+{}i128) as {primitive}) as i128;",ty.rank_base()));}
            else {let n=bytes_of(ty)?;ln(out,format!("let value=(c.uint({n})? as {primitive}) as i128;"));}
            let (min,max)=match ty {
                QslType::Byte=>(0,255),
                QslType::RangedInteger{min,max,..}=>(*min,*max),
                QslType::Integer{bits,signed,..}=>{let max=if *signed{(1i128<<(*bits-1))-1}else{(1i128<<*bits)-1};(if *signed{-(1i128<<(*bits-1))}else{0},max)},
                QslType::Flags(n)=>(0,(1i128<<*n)-1),_=>unreachable!()
            };
            ln(out,format!("if !({min}i128..={max}i128).contains(&value){{return Err(::qsl::QslError::InvalidValue);}}"));
        }
        QslType::FixedString(n)=>ln(out,format!("c.utf8({n},{n},{n})?;")),
        QslType::CompressedString{min,max}=>ln(out,format!("c.compressed::<{max}>({min},false,false)?;")),
        QslType::InlineRawString{min,max}=>ln(out,format!("c.compressed::<{max}>({min},true,false)?;")),
        QslType::FixedCompressedString(n)=>ln(out,format!("c.compressed::<{n}>({n},false,true)?;")),
        QslType::String(_)|QslType::RangedString{..}|QslType::BorrowedString{..}=>{
            let(min,max,_,_)=string_layout(ty).ok_or(QslError::InvalidSchema)?;
            if compact {ln(out,format!("let length=(c.bits({width})? as usize).checked_add({min}).ok_or(::qsl::QslError::LengthOverflow)?;"));}
            else {ln(out,"let length=c.uint(4)? as usize;");}
            ln(out,format!("c.utf8(length,{min},{})?;",max.unwrap_or(u32::MAX as usize)));
        }
        QslType::FixedArray(element,n)=>{if is_bytes(element){ln(out,format!("c.payload({n})?;"));}else{let et=if native_fixed{qsl_lexer::layout::native_array_rust_type(element)?}else{element.rust_type(false)?};ln(out,format!("c.check_native_allocation({n}usize.checked_mul(std::mem::size_of::<{et}>().max(1)).ok_or(::qsl::QslError::LengthOverflow)?)?;for _ in 0..{n}{{"));field_native(element,out,element.default_compact_bits()?,compact,native_fixed)?;ln(out,"}");if compact{ln(out,"c.align()?;");}}},
        QslType::Array(element,min)|QslType::RangedArray{element,min,..}=>{
            if compact{ln(out,format!("let length=(c.bits({width})? as usize).checked_add({min}).ok_or(::qsl::QslError::LengthOverflow)?;"));}else{ln(out,"let length=c.uint(4)? as usize;");}
            let max=if let QslType::RangedArray{max,..}=ty{*max}else{u32::MAX as usize};let native=element.fixed_wire_size()?;let et=element.rust_type(false)?;ln(out,format!("c.check_native_allocation(length.checked_mul(std::mem::size_of::<{et}>().max(1)).ok_or(::qsl::QslError::LengthOverflow)?)?;"));
            ln(out,format!("if !({min}..={max}).contains(&length){{return Err(::qsl::QslError::InvalidValue)}}"));
            if is_bytes(element){ln(out,"c.payload(length)?;");}else{let bits=element.default_compact_bits()?;if let Some(bytes)=native{if compact{let required_bits=compact_element_bits(element)?;ln(out,format!("c.check_values(length.checked_mul({bytes}usize).ok_or(::qsl::QslError::LengthOverflow)?,length.checked_mul({required_bits}).ok_or(::qsl::QslError::LengthOverflow)?)?;"));}else{ln(out,format!("c.check_allocation(length.checked_mul({bytes}usize).ok_or(::qsl::QslError::LengthOverflow)?)?;"));}}else{let et=element.rust_type(false)?;ln(out,format!("c.charge(length.checked_mul(std::mem::size_of::<{et}>()).ok_or(::qsl::QslError::LengthOverflow)?)?;"));}ln(out,"for _ in 0..length{");field(element,out,bits,compact)?;ln(out,"}");if compact{ln(out,"c.align()?;");}}
        }
        QslType::Custom{rust_type,..}=>{
            let size=if compact {"COMP_SIZE"}else{"SIZE"};
            ln(out,format!("c.charge(<{rust_type} as ::qsl::QslCustom>::SIZE)?;<{rust_type} as ::qsl::QslCustom>::qsl_validate(c.bytes(<{rust_type} as ::qsl::QslCustom>::{size})?,{compact})?;"));
        }
    }
    Ok(())
}
pub(super) fn methods(schema: &QslSchema, out: &mut String, plan: &QslIoPlan) -> QslResult<()> {
    values(schema, out)?;
    grouped_codegen::methods(schema, out)?;
    checked_codegen::callbacks(schema, out)?;
    body(schema, out, false)?;
    if schema.flags & crate::FLAG_SAFETY != 0 && !schema.options.native_overlay() {
        body(schema, out, true)?;
    }
    if plan.native_checked_into && schema.flags & crate::FLAG_SAFETY != 0 {
        checked_codegen::native_into(schema, out)?;
    }
    finish_pointing(
        schema,
        out,
        plan.pointing.as_ref(),
        plan.native_checked_into,
    )
}
fn body(schema: &QslSchema, out: &mut String, capture: bool) -> QslResult<()> {
    if capture {
        let lifetime = if schema.lifetime() { "'a " } else { "" };
        ln(out,format!("#[allow(unused_variables,unused_mut,unused_assignments,clippy::manual_range_contains,clippy::absurd_extreme_comparisons,clippy::unnecessary_cast)] fn __qsl_read_checked(input:&{lifetime}[u8])->::qsl::QslResult<Self>{{Self::__qsl_read_checked_with_limit(input,Self::MAX_DATA_SIZE)}}fn __qsl_read_checked_with_limit(input:&{lifetime}[u8],limit:usize)->::qsl::QslResult<Self>{{let mut c=::qsl::validation::ValidationCursor::with_budget(input,Self::MAX_DATA_SIZE,limit.min(Self::MAX_DATA_SIZE))?;"));
        if (!schema.opt_table() || schema.state_bits() || schema.predict_table())
            && !optional_codegen::applies(schema)
        {
            for f in schema.fields.iter().filter(|f| f.optional) {
                ln(
                    out,
                    format!(
                        "let mut field_{}:Option<{}>=None;",
                        f.tag,
                        qsl_lexer::layout::field_rust_type(f, schema.borrow_strings())?
                    ),
                );
            }
        }
    } else {
        ln(out,"/// Validate one complete frame before any trusted pointer decoding.\n#[allow(unused_variables,unused_mut,clippy::manual_range_contains,clippy::absurd_extreme_comparisons,clippy::unnecessary_cast)] pub fn validate(input:&[u8])->::qsl::QslResult<()> {Self::__qsl_validate_with_limit(input,Self::MAX_DATA_SIZE)}fn __qsl_validate_with_limit(input:&[u8],limit:usize)->::qsl::QslResult<()>{let mut c=::qsl::validation::ValidationCursor::with_budget(input,Self::MAX_DATA_SIZE,limit.min(Self::MAX_DATA_SIZE))?;");
    }
    emit_minimum_size_guard(schema, out)?;
    if optional_codegen::applies(schema) {
        return optional_codegen::checked(schema, out, capture);
    }
    if schema.predicted_opt_table() {
        return static_optional_codegen::validation(schema, out, capture);
    }
    let optional = schema
        .fields
        .iter()
        .filter(|f| f.optional)
        .collect::<Vec<_>>();
    let special = schema.state_bits() || schema.predict_table();
    if schema.state_bits() {
        let p = crate::static_index::PredictPlan::analyze(schema)?;
        ln(out, "let d=c.uint(4)? as u32;");
        if p.used_bits < 32 {
            ln(
                out,
                format!(
                    "if d>>{}!=0{{return Err(::qsl::QslError::InvalidValue);}}",
                    p.used_bits
                ),
            );
        }
    } else if !schema.sequential() {
        if schema.bitmap() {
            for (w, chunk) in schema.fields.chunks(32).enumerate() {
                ln(out, format!("let mask_{w}=c.uint(4)? as u32;"));
                let mut required = 0u32;
                let mut allowed = 0u32;
                for (i, f) in chunk.iter().enumerate() {
                    allowed |= 1 << (31 - i);
                    if !f.optional {
                        required |= 1 << (31 - i);
                    }
                }
                ln(out,format!("if mask_{w}&{required}!={required}||mask_{w}&!{allowed}u32!=0{{return Err(::qsl::QslError::InvalidValue);}}"));
            }
        } else if optional.len() == 1 {
            ln(out, "let presence=c.bits(1)?!=0;");
        } else if !optional.is_empty() {
            ln(out,format!("let count=c.bits({})? as usize;if count>{}{{return Err(::qsl::QslError::InvalidValue);}}let mut tags=0u64;let mut previous=0usize;",schema.count_bits(),optional.len()));
            let allowed = optional.iter().fold(0u64, |a, f| a | (1u64 << (f.tag - 1)));
            ln(out,format!("for i in 0..count{{let tag=c.bits({})? as usize;if tag>=64||(i!=0&&tag<=previous)||({allowed}u64&(1u64<<tag))==0{{return Err(::qsl::QslError::InvalidValue);}}tags|=1u64<<tag;previous=tag;}}",schema.tag_bits()));
        }
        if !schema.compact() {
            ln(out, "c.align()?;");
        }
    }
    if capture && schema.opt_table() && !schema.sequential() {
        checked_codegen::required(schema, out, true)?;
        checked_codegen::begin_result(schema, out);
        let mask = if schema.optional_count() == 1 {
            let f = schema.fields.iter().find(|f| f.optional).unwrap();
            format!("(presence as u64)<<{}", f.tag - 1)
        } else if schema.optional_count() == 0 {
            "0u64".into()
        } else {
            "tags".into()
        };
        ln(out,format!("let mut tags={mask};while tags!=0{{let tag=tags.trailing_zeros()as usize;tags&=tags-1;"));
        checked_codegen::dispatch(out, "tag", "0");
        ln(out, "}c.finish()?;Ok(result)}");
        return Ok(());
    }
    if !schema.sequential() {
        let mut i = 0;
        while i < schema.fields.len() {
            let n = checked_codegen::required_run(schema, out, i, capture)?;
            if n != 0 {
                i += n;
                continue;
            }
            let f = &schema.fields[i];
            if f.optional {
                let condition = if schema.bitmap() {
                    format!("mask_{}&(1u32<<{})!=0", i / 32, 31 - i % 32)
                } else if optional.len() == 1 {
                    "presence".into()
                } else {
                    format!("tags&(1u64<<{})!=0", f.tag - 1)
                };
                ln(out, format!("if {condition} {{"));
            }
            checked_codegen::field(
                f,
                out,
                f.data.rank_width(f.bits)?,
                schema.compact(),
                schema.borrow_strings(),
                capture,
                None,
            )?;
            if f.optional {
                ln(out, "}");
            }
            if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
                ln(out, "c.align()?;");
            }
            i += 1;
        }
        return checked_codegen::complete(schema, out, capture);
    }
    checked_codegen::required(schema, out, capture)?;
    if schema.legacy_opt_table() && !special {
        ln(
            out,
            format!(
                "if c.uint(1)?!={}{{return Err(::qsl::QslError::InvalidSchema);}}",
                optional.len()
            ),
        );
        for i in 0..optional.len() {
            ln(out, format!("let offset_{i}=c.uint(4)? as usize;"));
        }
        ln(out, "let payload_start=c.position();");
    }
    if schema.predict_table() {
        ln(out,format!("let length=input.len()-c.position();let rank=*{}_INDEX_TABLE.get(length).ok_or(::qsl::QslError::InvalidValue)? as usize;let row={}_OFFSET_TABLE.get(rank).ok_or(::qsl::QslError::InvalidValue)?;if row.0 as usize!=length{{return Err(::qsl::QslError::InvalidValue);}}let d=row.1;",schema.name,schema.name));
    }
    let plan = if special {
        Some(crate::static_index::PredictPlan::analyze(schema)?)
    } else {
        None
    };
    for (i, f) in optional.iter().enumerate() {
        let condition = if special {
            let p = &plan.as_ref().unwrap().fields[i];
            let mask = if p.bits == 32 {
                u32::MAX
            } else {
                (1u32 << p.bits) - 1
            };
            ln(out,format!("let code=(d>>{})&{mask}u32;if code>{}u32{{return Err(::qsl::QslError::InvalidValue);}}",p.shift,if p.fixed{1}else{p.max-p.min+1}));
            "code!=0".into()
        } else if schema.legacy_opt_table() {
            format!("offset_{i}!=0")
        } else if schema.predict_seq() {
            "c.position()<input.len()".into()
        } else if schema.bitmap() {
            let n = schema.fields.iter().position(|x| x.tag == f.tag).unwrap();
            format!("mask_{}&(1u32<<{})!=0", n / 32, 31 - n % 32)
        } else if optional.len() == 1 {
            "presence".into()
        } else {
            format!("tags&(1u64<<{})!=0", f.tag - 1)
        };
        ln(out, format!("if {condition} {{"));
        if schema.legacy_opt_table() && !special {
            ln(out,format!("if offset_{i}!=1+c.position()-payload_start{{return Err(::qsl::QslError::InvalidValue);}}"));
        }
        if special && qps_codegen::variable(&f.data) {
            let p = &plan.as_ref().unwrap().fields[i];
            let length = format!("code as usize-1+{}", p.min);
            checked_codegen::field(
                f,
                out,
                f.bits,
                false,
                schema.borrow_strings(),
                capture,
                Some(&length),
            )?;
        } else {
            checked_codegen::field(
                f,
                out,
                f.data.rank_width(f.bits)?,
                schema.compact(),
                schema.borrow_strings(),
                capture,
                None,
            )?;
        }
        ln(out, "}");
        if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "c.align()?;");
        }
    }
    checked_codegen::complete(schema, out, capture)
}
pub(super) fn finish(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    finish_pointing(schema, out, None, false)
}
fn finish_pointing(
    schema: &QslSchema,
    out: &mut String,
    pointing: Option<&crate::optimizer::PointingPlan>,
    native_into: bool,
) -> QslResult<()> {
    let checked_start = out.len();
    if schema.flags & crate::FLAG_SAFETY != 0 {
        let lifetime = if schema.lifetime() { "'a " } else { "" };
        if let Some(p) = pointing.filter(|p| p.all_bits_valid) {
            ln(
                out,
                "pub fn deserialize_checked(input:&[u8])->::qsl::QslResult<Self>{",
            );
            native_codegen::pointing_checked_prefix(p, out);
            ln(
                out,
                "Ok(unsafe{Self::__qsl_read_unchecked(input.as_ptr())})}",
            );
        } else {
            ln(out,format!("pub fn deserialize_checked(input:&{lifetime}[u8])->::qsl::QslResult<Self>{{Self::__qsl_read_checked(input)}}"));
        }
        if let Some(p) = pointing.filter(|p| p.all_bits_valid) {
            ln(out,"pub fn deserialize_checked_into<'out>(input:&[u8],output:&'out mut std::mem::MaybeUninit<Self>)->::qsl::QslResult<&'out mut Self>{");
            native_codegen::pointing_checked_prefix(p, out);
            ln(out,"unsafe{Self::__qsl_pointing_read_into(input.as_ptr(),output.as_mut_ptr());Ok(&mut*output.as_mut_ptr())}}");
        } else if native_into {
            ln(out,format!("/// A failed read may leave an initialized prefix inside output.
pub fn deserialize_checked_into<'out>(input:&{lifetime}[u8],output:&'out mut std::mem::MaybeUninit<Self>)->::qsl::QslResult<&'out mut Self>{{Self::__qsl_read_checked_native_into(input,output)}}"));
        } else if !schema.lifetime() {
            ln(out,"pub fn deserialize_checked_into<'out>(input:&[u8],output:&'out mut std::mem::MaybeUninit<Self>)->::qsl::QslResult<&'out mut Self>{let value=Self::__qsl_read_checked(input)?;Ok(output.write(value))}");
        }
        if schema.lifetime() && !native_into {
            ln(out,"pub fn deserialize_checked_into<'out>(input:&'a[u8],output:&'out mut std::mem::MaybeUninit<Self>)->::qsl::QslResult<&'out mut Self>{let value=Self::__qsl_read_checked(input)?;Ok(output.write(value))}");
        }
    }
    if schema.options.transmutate {
        let rewritten = out[checked_start..].replace(
            "Self::__qsl_read_checked(input)",
            "Self::deserialize_transmutate_copy(input)",
        );
        out.truncate(checked_start);
        out.push_str(&rewritten);
    }
    let encoder = if schema.lifetime() {
        "::qsl::BorrowSerializable<'a>"
    } else {
        "::qsl::Serializable"
    };
    let cap_limit = if schema.fields.iter().any(|f| output_may_shrink(&f.data)) {
        "usize::MAX"
    } else {
        "Self::MAX_DATA_SIZE"
    };
    ln(out,format!("/// Validate values once, then write into the caller's native byte region.
/// The output may be modified when an encoded-size limit error is returned.
pub fn serialize_checked_to(&self,out:&mut[u8])->::qsl::QslResult<usize>{{self.validate_values()?;let cap=<Self as {encoder}>::serialized_size(self);::qsl::validation::check_size(cap,{cap_limit})?;let dst=out.get_mut(..cap).ok_or(::qsl::QslError::BufferTooSmall)?;let n=unsafe{{<Self as {encoder}>::serialize_into_unsafe(self,dst.as_mut_ptr())}};::qsl::validation::check_size(n,Self::MAX_DATA_SIZE)?;Ok(n)}}"));
    ln(out,format!("/// Fallible owned output; prefer serialize_checked_to for prepared storage.
pub fn serialize_checked(&self)->::qsl::QslResult<Vec<u8>>{{self.validate_values()?;let cap=<Self as {encoder}>::serialized_size(self);::qsl::validation::check_size(cap,{cap_limit})?;let mut out=Vec::new();out.try_reserve_exact(cap).map_err(|_|::qsl::QslError::AllocationFailed)?;let n=unsafe{{<Self as {encoder}>::serialize_into_unsafe(self,out.as_mut_ptr())}};::qsl::validation::check_size(n,Self::MAX_DATA_SIZE)?;unsafe{{out.set_len(n);}}Ok(out)}}"));
    Ok(())
}

fn array_element_values(element: &QslType, out: &mut String) -> QslResult<()> {
    match element {
        QslType::DomainInteger(d)=>numeric_codegen::check_value(d,out,"*item"),
        QslType::Custom { rust_type, .. } => ln(out,format!("<{rust_type} as ::qsl::QslCustom>::qsl_validate_values(item)?;")),
        QslType::RangedInteger { min, max, .. } => ln(out,format!("if !({min}i128..={max}i128).contains(&(*item as i128)){{return Err(::qsl::QslError::InvalidValue)}}")),
        QslType::Flags(bits) if *bits < 64 => ln(out,format!("if (*item as u64)>={}u64{{return Err(::qsl::QslError::InvalidValue)}}",1u64<<*bits)),
        QslType::Array(e, _) | QslType::FixedArray(e, _) | QslType::RangedArray { element: e, .. } => {
            ln(out,"for item in item.iter(){");
            array_element_values(e, out)?;
            ln(out,"}");
        }
        _ => {}
    }
    Ok(())
}
pub(super) fn values(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    // Nested if is valid in both Rust 2021 and 2024 consumer crates.
    ln(out,"#[allow(unused_variables,unused_mut,clippy::manual_range_contains,clippy::absurd_extreme_comparisons,clippy::collapsible_if)] pub fn validate_values(&self)->::qsl::QslResult<()> {let mut budget=::qsl::validation::ValueBudget::new(Self::MAX_DATA_SIZE);");
    for f in &schema.fields {
        if f.optional {
            ln(
                out,
                format!("if let Some(value)=self.{}.as_ref(){{", f.name),
            );
        } else {
            ln(out, format!("let value=&self.{};", f.name));
        }
        value_field(f, out)?;
        if f.optional {
            ln(out, "}");
        }
    }
    ln(out, "Ok(()) }");
    Ok(())
}

/// Validate one initialized value, using the surrounding aggregate budget.
pub(super) fn value_field(f: &crate::QslField, out: &mut String) -> QslResult<()> {
    if let QslType::Array(e, _)
    | QslType::RangedArray { element: e, .. }
    | QslType::FixedArray(e, _) = &f.data
    {
        let (min, max) = match f.data {
            QslType::Array(_, min) => (min, u32::MAX as usize),
            QslType::RangedArray { min, max, .. } => (min, max),
            QslType::FixedArray(_, n) => (n, n),
            _ => unreachable!(),
        };
        ln(
            out,
            "if value.len()>Self::MAX_DATA_SIZE{return Err(::qsl::QslError::SizeLimitExceeded);}",
        );
        let invalid = if min == 0 {
            format!("value.len()>{max}")
        } else if min == max {
            format!("value.len()!={min}")
        } else {
            format!("value.len()<{min}||value.len()>{max}")
        };
        ln(
            out,
            format!("if {invalid}{{return Err(::qsl::QslError::InvalidValue);}}"),
        );
        if e.fixed_wire_size()?.is_none() {
            let et = e.rust_type(false)?;
            ln(out,format!("budget.charge(value.len().checked_mul(std::mem::size_of::<{et}>()).ok_or(::qsl::QslError::LengthOverflow)?)?;"));
        }
        let child = crate::QslField {
            data: (**e).clone(),
            ..Default::default()
        };
        ln(out, "for item in value.iter(){let value=item;");
        value_field(&child, out)?;
        ln(out, "}");
        return Ok(());
    }
    if let Some((min, max, _, _)) = string_layout(&f.data) {
        let bytes = if matches!(
            f.storage,
            qsl_lexer::layout::StorageKind::HeapString
                | qsl_lexer::layout::StorageKind::HeapInlineString(_)
                | qsl_lexer::layout::StorageKind::StrView
        ) {
            "value.as_bytes()"
        } else if matches!(
            f.data,
            QslType::FixedString(_) | QslType::FixedCompressedString(_)
        ) {
            "value.as_slice()"
        } else {
            "value.as_bytes()"
        };
        // &[u8] already is a slice; as_slice on a reference is unstable.
        let bytes = { bytes };
        let charge = if let qsl_lexer::layout::StorageKind::HeapInlineString(capacity) = f.storage {
            format!("std::mem::size_of::<::qsl::InlineString<{capacity}>>()")
        } else {
            "bytes.len()".into()
        };
        ln(out,format!("let bytes={bytes};budget.charge({charge})?;if !({min}..={}).contains(&bytes.len()){{return Err(::qsl::QslError::InvalidValue);}}",max.unwrap_or(u32::MAX as usize)));
        if matches!(
            f.data,
            QslType::FixedString(_) | QslType::FixedCompressedString(_)
        ) {
            ln(
                out,
                "std::str::from_utf8(bytes).map_err(|_|::qsl::QslError::InvalidUtf8)?;",
            );
        }
    } else {
        if let QslType::BitPadding(n) = f.data {
            ln(out, format!("budget.charge({})?;", n / 8));
        } else if let QslType::Custom { rust_type, .. } = &f.data {
            ln(
                out,
                format!("budget.charge(<{rust_type} as ::qsl::QslCustom>::SIZE)?;"),
            );
        } else if let Some(size) = f.data.fixed_wire_size()? {
            ln(out, format!("budget.charge({size})?;"));
        }
        match &f.data {
            QslType::Custom{rust_type,..}=>ln(out,format!("<{rust_type} as ::qsl::QslCustom>::qsl_validate_values(value)?;")),
            QslType::DomainInteger(d)=>numeric_codegen::check_value(d,out,"*value"),
            QslType::RangedInteger{min,max,..}=>ln(out,format!("if !({min}i128..={max}i128).contains(&(*value as i128)){{return Err(::qsl::QslError::InvalidValue);}}")),
            QslType::Integer{min,..} if *min!=0=>ln(out,format!("if (*value as i128)<{min}i128{{return Err(::qsl::QslError::InvalidValue);}}")),
            QslType::Flags(bits) if *bits<64=>ln(out,format!("if (*value as u64)>={}u64{{return Err(::qsl::QslError::InvalidValue);}}",1u64<<*bits)),
            QslType::Array(element,min)|QslType::RangedArray{element,min,..}=>{let size=element.fixed_wire_size()?.ok_or(QslError::InvalidSchema)?;ln(out,format!("budget.charge(value.len().checked_mul({size}usize).ok_or(::qsl::QslError::LengthOverflow)?)?;"));let max=if let QslType::RangedArray{max,..}=&f.data{*max}else{u32::MAX as usize};ln(out,format!("if !({min}..={max}).contains(&value.len()){{return Err(::qsl::QslError::InvalidValue);}}"));},
            QslType::FixedArray(_,n)=>ln(out,format!("if value.len()!={n}{{return Err(::qsl::QslError::InvalidValue);}}")),_=>{}
        }
    }
    if let QslType::Array(e, _)
    | QslType::FixedArray(e, _)
    | QslType::RangedArray { element: e, .. } = &f.data
    {
        ln(out, "for item in value.iter(){");
        array_element_values(e, out)?;
        ln(out, "}");
    }
    Ok(())
}
