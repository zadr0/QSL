//! Single-pass checked materialization. A failed cursor never publishes Self.
use super::*;
use crate::modernization::StorageKind;
pub(super) fn field(
    f: &QslField,
    out: &mut String,
    width: usize,
    compact: bool,
    borrowed: bool,
    capture: bool,
    length: Option<&str>,
) -> QslResult<()> {
    if !capture && f.storage == StorageKind::HeapString {
        match f.data {
            QslType::CompressedString { min, max } | QslType::InlineRawString { min, max }
                if length.is_none() =>
            {
                ln(
                    out,
                    format!(
                        "c.decode_compressed_owned::<{max}>({min},{},false)?;",
                        matches!(f.data, QslType::InlineRawString { .. })
                    ),
                );
                return Ok(());
            }
            QslType::FixedCompressedString(n) => {
                ln(
                    out,
                    format!("c.decode_compressed_owned::<{n}>({n},false,true)?;"),
                );
                return Ok(());
            }
            _ => {}
        }
    }
    if !capture {
        if let Some(length) = length {
            let (min, max, _, _) = string_layout(&f.data).ok_or(QslError::InvalidSchema)?;
            ln(
                out,
                format!(
                    "c.utf8({length},{min},{})?;",
                    max.unwrap_or(u32::MAX as usize)
                ),
            );
            return Ok(());
        }
        return validation_codegen::field(&f.data, out, width, compact);
    }
    ln(
        out,
        if f.optional {
            format!("field_{}=Some({{", f.tag)
        } else {
            format!("let field_{}={{", f.tag)
        },
    );
    value(&f.data, out, width, compact, borrowed, f.storage, length)?;
    ln(out, if f.optional { "});" } else { "};" });
    Ok(())
}
fn value(
    ty: &QslType,
    out: &mut String,
    width: usize,
    compact: bool,
    borrowed: bool,
    storage: StorageKind,
    length: Option<&str>,
) -> QslResult<()> {
    if storage == StorageKind::NativeView {
        let QslType::Custom {
            rust_type,
            fixed_wire_size: Some(n),
            ..
        } = ty
        else {
            return Err(QslError::InvalidSchema);
        };
        ln(
            out,
            format!("{{c.charge({n})?;{rust_type}::__qsl_region_view(c.bytes({n})?)?}}"),
        );
        return Ok(());
    }
    if storage == StorageKind::StrView {
        if let QslType::InlineRawString { min, max } = ty {
            if let Some(length) = length {
                ln(out, format!("c.text({length},{min},{max})?"));
            } else {
                let header = if *max <= 127 {
                    1
                } else if *max <= 32767 {
                    2
                } else {
                    4
                };
                ln(
                    out,
                    format!(
                        "{{let length=c.uint({header})? as usize;c.text(length,{min},{max})?}}"
                    ),
                );
            }
            return Ok(());
        }
    }
    if storage == StorageKind::HeapString {
        match ty {
            QslType::CompressedString { min, max } | QslType::InlineRawString { min, max }
                if length.is_none() =>
            {
                ln(
                    out,
                    format!(
                        "c.decode_compressed_owned::<{max}>({min},{},false)?",
                        matches!(ty, QslType::InlineRawString { .. })
                    ),
                );
                return Ok(());
            }
            QslType::FixedCompressedString(n) => {
                ln(
                    out,
                    format!("c.decode_compressed_owned::<{n}>({n},false,true)?"),
                );
                return Ok(());
            }
            _ => {}
        }
    }
    match ty{
 QslType::None=>ln(out,"()"),
 QslType::Padding(_)=>{validation_codegen::field(ty,out,width,compact)?;ln(out,"()");},
 QslType::Byte|QslType::Integer{..}|QslType::RangedInteger{..}|QslType::Flags(_)=>{validation_codegen::field(ty,out,width,compact)?;ln(out,format!("value as {}",ty.primitive()?.ok_or(QslError::InvalidSchema)?));},
 QslType::InlineRawString{min,max} if length.is_some()=>{ln(out,format!("let text=c.text({},{min},{max})?;",length.unwrap()));if storage==StorageKind::HeapString{ln(out,"{let mut owned=String::new();owned.try_reserve_exact(text.len()).map_err(|_|::qsl::QslError::AllocationFailed)?;owned.push_str(text);owned}");}else{ln(out,format!("unsafe{{::qsl::InlineString::<{max}>::from_str_unchecked(text)}}"));}},
 QslType::CompressedString{min,max}|QslType::InlineRawString{min,max}=>ln(out,format!("c.decode_compressed::<{max}>({min},{},false)?",matches!(ty,QslType::InlineRawString{..}))),
 QslType::FixedCompressedString(n)=>ln(out,format!("{{let text=c.decode_compressed::<{n}>({n},false,true)?;let mut bytes=[0u8;{n}];bytes.copy_from_slice(text.as_bytes());bytes}}")),
 QslType::FixedString(n)=>{ln(out,format!("let text=c.text({n},{n},{n})?;"));if storage==StorageKind::StrView{ln(out,"text");}else if storage==StorageKind::HeapString{ln(out,"{let mut owned=String::new();owned.try_reserve_exact(text.len()).map_err(|_|::qsl::QslError::AllocationFailed)?;owned.push_str(text);owned}");}else{ln(out,format!("{{let mut bytes=std::mem::MaybeUninit::<[u8;{n}]>::uninit();unsafe{{std::ptr::copy_nonoverlapping(text.as_bytes().as_ptr(),bytes.as_mut_ptr().cast(),{n});bytes.assume_init()}}}}"));}},
 QslType::String(_)|QslType::RangedString{..}|QslType::BorrowedString{..}=>{
  let(min,max,_,_)=string_layout(ty).ok_or(QslError::InvalidSchema)?;
  if let Some(length)=length{ln(out,format!("let length={length};"));}else if compact{ln(out,format!("let length=(c.bits({width})? as usize).checked_add({min}).ok_or(::qsl::QslError::LengthOverflow)?;"));}else{ln(out,"let length=c.uint(4)? as usize;");}
  ln(out,format!("let text=c.text(length,{min},{})?;",max.unwrap_or(u32::MAX as usize)));
  match storage{StorageKind::StackString(n)=>ln(out,format!("::qsl::StackString::<{n}>::try_from_str(text)?")),StorageKind::StrView=>ln(out,"text"),StorageKind::HeapString=>ln(out,"{let mut owned=String::new();owned.try_reserve_exact(text.len()).map_err(|_|::qsl::QslError::AllocationFailed)?;owned.push_str(text);owned}"),_=>if borrowed{ln(out,"text")}else{ln(out,"{let mut owned=String::new();owned.try_reserve_exact(text.len()).map_err(|_|::qsl::QslError::AllocationFailed)?;owned.push_str(text);owned}")}}
 },
 QslType::FixedArray(e,n)|QslType::Array(e,n)|QslType::RangedArray{element:e,min:n,..}=>{
  let fixed=matches!(ty,QslType::FixedArray(..));let max=if let QslType::RangedArray{max,..}=ty{*max}else{u32::MAX as usize};
  if fixed{ln(out,format!("let length={n}usize;"));}else if compact{ln(out,format!("let length=(c.bits({width})? as usize).checked_add({n}).ok_or(::qsl::QslError::LengthOverflow)?;"));}else{ln(out,"let length=c.uint(4)? as usize;");}
  if !fixed{ln(out,format!("if !({n}..={max}).contains(&length){{return Err(::qsl::QslError::InvalidValue)}}"));}
  let size=e.fixed_wire_size()?.ok_or(QslError::InvalidSchema)?;ln(out,format!("let bytes=length.checked_mul({size}usize).ok_or(::qsl::QslError::LengthOverflow)?;"));
  if compact&&!is_bytes(e){let bits=validation_codegen::compact_element_bits(e)?;ln(out,format!("c.check_values(bytes,length.checked_mul({bits}).ok_or(::qsl::QslError::LengthOverflow)?)?;"));}else{ln(out,"c.check_allocation(bytes)?;");}
  if borrowed&&!matches!(storage,StorageKind::FixedArray(_)){ln(out,"c.payload(bytes)?");}else{
   let et=e.rust_type(false)?;match storage{StorageKind::FixedArray(n)|StorageKind::StackArray(n)=>ln(out,format!("let mut values=::qsl::InlineArray::<{et},{n}>::new();")),_=>ln(out,format!("let mut values=Vec::<{et}>::new();values.try_reserve_exact(length).map_err(|_|::qsl::QslError::AllocationFailed)?;"))}
   ln(out,"for _ in 0..length{let item={");value(e,out,e.default_compact_bits()?,compact,false,StorageKind::Default,None)?;ln(out,"};");if matches!(storage,StorageKind::FixedArray(_)|StorageKind::StackArray(_)){ln(out,"values.push(item)?;");}else{ln(out,"values.push(item);");}ln(out,"}");if compact&&!is_bytes(e){ln(out,"c.align()?;");}ln(out,if matches!(storage,StorageKind::FixedArray(_)){"values.into_array()?"}else{"values"});
  }
 },
 QslType::Custom{rust_type,..}=>{let size=if compact{"COMP_SIZE"}else{"SIZE"};let method=if compact{"qsl_comp_read_unchecked"}else{"qsl_read_unchecked"};ln(out,format!("c.charge(<{rust_type} as ::qsl::QslCustom>::SIZE)?;let bytes=c.bytes(<{rust_type} as ::qsl::QslCustom>::{size})?;<{rust_type} as ::qsl::QslCustom>::qsl_validate(bytes,{compact})?;unsafe{{<{rust_type} as ::qsl::QslCustom>::{method}(bytes.as_ptr())}}"));}
 };
    Ok(())
}
pub(super) fn complete(s: &QslSchema, out: &mut String, capture: bool) -> QslResult<()> {
    if capture {
        ln(out, "c.finish()?;Ok(Self{");
        for f in &s.fields {
            ln(out, format!("{}:field_{},", f.name, f.tag));
        }
        ln(out, "})}");
    } else {
        ln(out, "c.finish() }");
    }
    Ok(())
}

pub(super) fn required_run(
    s: &QslSchema,
    out: &mut String,
    start: usize,
    capture: bool,
) -> QslResult<usize> {
    let (count, bytes, metadata) = required_numeric_run(s, start);
    if count == 0 {
        return Ok(0);
    }
    ln(
        out,
        format!(
            "c.charge({bytes})?;let group=c.bytes({})?;",
            bytes + metadata
        ),
    );
    emit_register_group(
        s,
        out,
        start,
        count,
        "group.as_ptr()",
        "__qsl_scalar_",
        metadata,
    )?;
    for (i, f) in s.fields[start..start + count].iter().enumerate() {
        let mut tmp = f.clone();
        tmp.optional = true;
        let r = crate::modernization::optional_representation(&tmp, false)
            .ok_or(QslError::InvalidSchema)?;
        ln(out,format!("if !({}i128..={}i128).contains(&(__qsl_scalar_{} as i128)){{return Err(::qsl::QslError::InvalidValue)}}",r.minimum,r.maximum,start+i));
        if capture {
            ln(
                out,
                format!("let field_{}=__qsl_scalar_{};", f.tag, start + i),
            );
        }
    }
    if metadata != 0 {
        let f = &s.fields[start + count];
        field(
            f,
            out,
            f.data.rank_width(f.bits)?,
            s.compact(),
            s.borrow_strings(),
            capture,
            Some(&format!("__qsl_header_{start} as usize")),
        )?;
    }
    Ok(count + usize::from(metadata != 0))
}
pub(super) fn required(s: &QslSchema, out: &mut String, capture: bool) -> QslResult<()> {
    let mut i = 0;
    while i < s.fields.len() && !s.fields[i].optional {
        let n = required_run(s, out, i, capture)?;
        if n != 0 {
            i += n;
            continue;
        }
        let f = &s.fields[i];
        field(
            f,
            out,
            f.data.rank_width(f.bits)?,
            s.compact(),
            s.borrow_strings(),
            capture,
            None,
        )?;
        if s.compact() && s.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "c.align()?;");
        }
        i += 1;
    }
    Ok(())
}
pub(super) fn begin_result(s: &QslSchema, out: &mut String) {
    ln(out, "#[allow(unused_mut)]let mut result=Self{");
    for f in &s.fields {
        ln(
            out,
            format!(
                "{}:{},",
                f.name,
                if f.optional {
                    "None".into()
                } else {
                    format!("field_{}", f.tag)
                }
            ),
        );
    }
    ln(out, "};");
}
pub(super) fn callbacks(s: &QslSchema, out: &mut String) -> QslResult<()> {
    if (!s.opt_table() && !optional_codegen::applies(s)) || s.flags & crate::FLAG_SAFETY == 0 {
        return Ok(());
    }
    let lifetime = if s.lifetime() { "'a" } else { "'input" };
    let generic = if s.lifetime() { "" } else { "<'input>" };
    let hrtb = if s.lifetime() { "" } else { "for<'input> " };
    let graph_raw = s.predicted_opt_table()
        && matches!(
            crate::static_optional::StaticOptionalPlan::analyze(s)?.presence,
            crate::static_optional::StaticPresence::Graph(_)
        );
    for f in s.fields.iter().filter(|f| f.optional) {
        ln(out,format!("#[allow(dead_code,unused_variables,unused_assignments)]fn __qsl_checked_opt_{}{generic}(c:&mut ::qsl::validation::ValidationCursor<{lifetime}>,output:&mut Self,length:usize)->::qsl::QslResult<()>{{let mut field_{}=None;",f.name,f.tag));
        field(
            f,
            out,
            f.data.rank_width(f.bits)?,
            s.compact(),
            s.borrow_strings(),
            true,
            if (s.state_bits() || graph_raw) && qps_codegen::variable(&f.data) {
                Some("length")
            } else {
                None
            },
        )?;
        if s.compact() && s.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "c.align()?;");
        }
        ln(out, format!("output.{}=field_{};Ok(())}}", f.name, f.tag));
    }
    if !s.opt_table() {
        return Ok(());
    }
    let dynamic = optional_codegen::applies(s);
    let table_len = if dynamic {
        s.optional_count()
    } else {
        s.max_tag
    };
    let table_base = if dynamic { s.optional_tag_base() } else { 0 };
    let entries = (1 + table_base..=table_len + table_base)
        .map(|tag| {
            s.fields
                .iter()
                .find(|f| f.optional && f.tag == tag)
                .map_or("None".into(), |f| {
                    format!("Some(Self::__qsl_checked_opt_{})", f.name)
                })
        })
        .collect::<Vec<_>>()
        .join(",");
    ln(out,format!("#[allow(dead_code,clippy::type_complexity)]const __QSL_CHECKED_OPT_TABLE:[Option<{hrtb}fn(&mut ::qsl::validation::ValidationCursor<{lifetime}>,&mut Self,usize)->::qsl::QslResult<()>>;{}]=[{entries}];",table_len));
    Ok(())
}
pub(super) fn dispatch(out: &mut String, tag: &str, length: &str) {
    ln(out,format!("// Index is proven optional by the checked header or generated fixed tag.\nlet callback_length={length};(unsafe{{Self::__QSL_CHECKED_OPT_TABLE.get_unchecked({tag}).unwrap_unchecked()}})(&mut c,&mut result,callback_length)?;"));
}
