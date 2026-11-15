//! Dyn v1 field framing. Scalar field kernels are shared with ordinary codegen.
//! Index validation uses caller scratch; no temporary Vec of tags or indices.
use super::*;
use qsl_core::dynamic::{DynHeader, DynMode, DynWidth};
fn header_expr(h: DynHeader) -> String {
    let mode = match h.mode {
        DynMode::Line => "Line",
        DynMode::StrictIndex => "StrictIndex",
        DynMode::DynamicIndex => "DynamicIndex",
    };
    format!("::qsl::dynamic::DynHeader{{mode: ::qsl::dynamic::DynMode::{mode},width: ::qsl::dynamic::DynWidth::W{},revision:{}}}",h.width as u8,h.revision)
}
fn scalar_bits(s: &QslSchema, f: &QslField) -> Option<usize> {
    if s.compact()
        && matches!(
            f.data,
            QslType::Byte
                | QslType::Integer { .. }
                | QslType::RangedInteger { .. }
                | QslType::DomainInteger(_)
                | QslType::Flags(_)
        )
    {
        f.data.rank_width(f.bits).ok()
    } else {
        None
    }
}
fn index_size_bits(s: &QslSchema, f: &QslField) -> QslResult<String> {
    if let Some(w) = scalar_bits(s, f) {
        Ok(w.to_string())
    } else {
        size_bits(f)
    }
}
fn scalar_read(
    s: &QslSchema,
    f: &QslField,
    out: &mut String,
    checked: bool,
    capture: bool,
) -> QslResult<()> {
    let w = scalar_bits(s, f).ok_or(QslError::InvalidSchema)?;
    let ty = f.data.primitive()?.ok_or(QslError::InvalidSchema)?;
    if let QslType::DomainInteger(d) = &f.data {
        if checked {
            ln(out,format!("budget.charge({})?;let rank=field.unsigned({w})?;field.validate_scalar_tail({w})?;",d.physical_bits()/8));
            if d.maximum_rank() < qsl_lexer::domain::low_mask(d.encoded_bits()) {
                ln(
                    out,
                    format!(
                        "if rank>{}u128{{return Err(::qsl::QslError::InvalidValue);}}",
                        d.maximum_rank()
                    ),
                );
            }
        } else {
            ln(out, format!("let rank=field.unsigned_unchecked({w});"));
        }
        ln(out, "let _=rank;");
        if capture {
            ln(
                out,
                format!(
                    "field_{}=Some({});",
                    f.tag,
                    numeric_codegen::decode(d, "rank")
                ),
            );
        } else if !f.optional {
            ln(out, format!("seen_{}=true;", f.tag));
        }
        return Ok(());
    }
    let base = f.data.rank_base();
    if checked {
        ln(out,format!("budget.charge({})?;let rank=field.unsigned({w})?;field.validate_scalar_tail({w})?;",bytes_of(&f.data)?));
    } else {
        ln(out, format!("let rank=field.unsigned_unchecked({w});"));
    }
    if !capture && !matches!(f.data, QslType::RangedInteger { .. }) {
        ln(out, "let _=rank;");
    }
    let expr = if matches!(f.data, QslType::RangedInteger { .. }) {
        format!("(rank as i128)+{}", min_literal(base))
    } else {
        format!("(((rank as i128)+{})as {ty})as i128", min_literal(base))
    };
    if capture || matches!(f.data, QslType::RangedInteger { .. }) {
        ln(out, format!("let value={expr};"));
    }
    if checked {
        if let QslType::RangedInteger { min, max, .. } = f.data {
            ln(out,format!("if !({min}i128..={max}i128).contains(&value){{return Err(::qsl::QslError::InvalidValue);}}"));
        }
    }
    if capture {
        ln(out, format!("field_{}=Some(value as {ty});", f.tag));
    } else if !f.optional {
        ln(out, format!("seen_{}=true;", f.tag));
    }
    Ok(())
}
fn size_bits(f: &QslField) -> QslResult<String> {
    Ok(match &f.data {
  QslType::String(_)|QslType::RangedString{..}|QslType::BorrowedString{..}|QslType::Array(..)|QslType::RangedArray{..}=>"32".into(),
  QslType::Custom{rust_type,..}=>format!("<{rust_type} as ::qsl::QslCustom>::SIZE.checked_mul(8).ok_or(::qsl::QslError::LengthOverflow)?"),
  _=>f.data.fixed_wire_size()?.ok_or(QslError::InvalidSchema)?.checked_mul(8).ok_or(QslError::LengthOverflow)?.to_string(),
 })
}
fn size_body(s: &QslSchema, out: &mut String, checked: bool) -> QslResult<()> {
    let h = s.options.dynamic.unwrap();
    let w = h.width.bytes();
    ln(out, "let mut __qsl_bits=0usize;let mut entries=0usize;");
    for f in &s.fields {
        let value = if f.optional {
            ln(
                out,
                format!("if let Some(value)=self.{}.as_ref(){{", f.name),
            );
            "value".to_string()
        } else {
            format!("&self.{}", f.name)
        };
        if scalar_bits(s, f).is_none() {
            ln(out, "__qsl_bits=(__qsl_bits+7)&!7usize;");
        }
        if checked && h.mode != DynMode::Line {
            let max = if h.width == DynWidth::W32 {
                u16::MAX as u64
            } else {
                u32::MAX as u64
            };
            ln(
                out,
                format!("if __qsl_bits>{max}usize{{return Err(::qsl::QslError::LengthOverflow);}}"),
            );
            if h.mode == DynMode::DynamicIndex {
                let max = if w == 4 { 255 } else { 65535 };
                ln(
                    out,
                    format!(
                        "if {}>{max}usize{{return Err(::qsl::QslError::LengthOverflow);}}",
                        index_size_bits(s, f)?
                    ),
                );
            }
        }
        ln(out, "let before=__qsl_bits;");
        if let Some(width) = scalar_bits(s, f) {
            ln(out, format!("__qsl_bits+={width};"));
            if s.flags & crate::FLAG_ALIGNMENT != 0 {
                ln(out, "__qsl_bits=(__qsl_bits+7)&!7usize;");
            }
        } else {
            emit_size_stored(f, out, &value, f.bits, false)?;
        }
        ln(out, "entries+=1;");
        if checked && h.mode == DynMode::Line && w < 8 {
            let max = if w == 2 {
                u16::MAX as u64
            } else {
                u32::MAX as u64
            };
            ln(out,format!("if (__qsl_bits-before)/8>{max}usize{{return Err(::qsl::QslError::LengthOverflow);}}"));
        }
        if f.optional {
            ln(out, "}");
        }
    }
    ln(out, "__qsl_bits=(__qsl_bits+7)&!7usize;");
    let overhead = if h.mode == DynMode::Line {
        format!("entries*{}", 2 * w)
    } else {
        format!("entries*{w}+4")
    };
    if checked {
        ln(out,format!("let size=(__qsl_bits/8).checked_add(12).and_then(|n|n.checked_add({overhead})).ok_or(::qsl::QslError::LengthOverflow)?;::qsl::validation::check_size(size,Self::MAX_DATA_SIZE)?;Ok(size)"));
    } else {
        ln(out, format!("12+__qsl_bits/8+{overhead}"));
    }
    Ok(())
}
fn checked_read(s: &QslSchema, out: &mut String, capture: bool) -> QslResult<()> {
    let h = s.options.dynamic.unwrap();
    let lt = if s.lifetime() { "'a " } else { "" };
    let scratch = if h.mode == DynMode::Line {
        "u64"
    } else {
        "u32"
    };
    let ret = if capture { "Self" } else { "()" };
    let method = if capture {
        "deserialize_checked"
    } else {
        "validate"
    };
    ln(
        out,
        format!(
            "pub fn {method}(input:&{lt}[u8],scratch:&mut[{scratch}])->::qsl::QslResult<{ret}>{{"
        ),
    );
    if h.mode == DynMode::Line {
        ln(
            out,
            "let view=::qsl::dynamic::DynLineView::parse(input,Self::MAX_DATA_SIZE,scratch)?;",
        );
    } else {
        ln(
            out,
            "let view=::qsl::dynamic::DynIndexView::parse(input,Self::MAX_DATA_SIZE,scratch)?;",
        );
    }
    ln(out,"if view.header!=Self::DYN_HEADER{return Err(::qsl::QslError::InvalidSchema);}let mut budget=::qsl::validation::ValueBudget::new(Self::MAX_DATA_SIZE);");
    for f in &s.fields {
        if capture {
            let ty = qsl_lexer::layout::field_rust_type(f, s.borrow_strings())?;
            ln(out, format!("let mut field_{}:Option<{ty}>=None;", f.tag));
        } else if !f.optional {
            ln(out, format!("let mut seen_{}=false;", f.tag));
        }
    }
    if h.mode == DynMode::Line {
        ln(
            out,
            "for field in view.fields(){let bytes=field.bytes;match field.tag{",
        );
    } else {
        ln(
            out,
            "for row in 0..view.len(){let field=view.at(row).unwrap();match field.entry.tag{",
        );
    }
    for f in &s.fields {
        ln(out, format!("{}=>{{", f.tag));
        if h.mode == DynMode::DynamicIndex {
            ln(out,format!("if field.entry.size_bits as usize!={}{{return Err(::qsl::QslError::InvalidWidth);}}",index_size_bits(s,f)?));
        }
        if scalar_bits(s, f).is_some() {
            scalar_read(s, f, out, true, capture)?;
            ln(out, "},");
            continue;
        }
        if h.mode != DynMode::Line {
            ln(out, "let bytes=field.byte_extent()?;");
        }
        ln(out,"let before=budget.remaining();let mut c=::qsl::validation::ValidationCursor::with_budget(bytes,Self::MAX_DATA_SIZE,before)?;");
        if capture {
            let mut field = f.clone();
            field.optional = true;
            checked_codegen::field(&field, out, f.bits, false, s.borrow_strings(), true, None)?;
        } else {
            validation_codegen::field_stored(f, out, f.bits, false)?;
            if !f.optional {
                ln(out, format!("seen_{}=true;", f.tag));
            }
        }
        ln(
            out,
            "budget.charge(before-c.remaining_budget())?;c.finish()?;},",
        );
    }
    ln(out, "_=>{}}}");
    if capture {
        ln(out, "Ok(Self{");
        for f in &s.fields {
            ln(
                out,
                if f.optional {
                    format!("{}:field_{},", f.name, f.tag)
                } else {
                    format!(
                        "{}:field_{}.ok_or(::qsl::QslError::InvalidValue)?,",
                        f.name, f.tag
                    )
                },
            );
        }
        ln(out, "})}");
    } else {
        for f in s.fields.iter().filter(|f| !f.optional) {
            ln(
                out,
                format!(
                    "if !seen_{}{{return Err(::qsl::QslError::InvalidValue);}}",
                    f.tag
                ),
            );
        }
        ln(out, "Ok(())}");
    }
    Ok(())
}
pub(super) fn generate(i: &QslInterpreter, s: &QslSchema, out: &mut String) -> QslResult<()> {
    let h = s.options.dynamic.ok_or(QslError::InvalidSchema)?;
    let name = &s.name;
    let w = h.width.bytes();
    let lt = if s.lifetime() { "<'a>" } else { "" };
    if i.config.qsl_rt || s.options.loyal_cache || s.options.cache_limit.is_some() {
        return Err(QslError::InvalidSchema);
    }
    QslInterpreter::generate_struct(s, out)?;
    i.emit_schema_hashes(s, out)?;
    ln(
        out,
        format!(
            "impl{lt} {name}{lt}{{pub const DYN_HEADER: ::qsl::dynamic::DynHeader={};",
            header_expr(h)
        ),
    );
    validation_codegen::values(s, out)?;
    ln(
        out,
        "#[allow(unused_variables)]pub fn serialized_size(&self)->usize{",
    );
    size_body(s, out, false)?;
    ln(out, "}");
    ln(out,"#[allow(unused_variables,unused_comparisons,clippy::absurd_extreme_comparisons)]fn __qsl_dyn_checked_size(&self)->::qsl::QslResult<usize>{");
    size_body(s, out, true)?;
    ln(out, "}");
    ln(out,"pub fn serialize_checked_to(&self,out:&mut[u8])->::qsl::QslResult<usize>{self.validate_values()?;let size=self.__qsl_dyn_checked_size()?;let out=out.get_mut(..size).ok_or(::qsl::QslError::BufferTooSmall)?;Ok(unsafe{self.serialize_dyn_unchecked(out.as_mut_ptr())})}");
    ln(out,"pub fn serialize_checked(&self)->::qsl::QslResult<Vec<u8>>{self.validate_values()?;let size=self.__qsl_dyn_checked_size()?;let mut out=Vec::new();out.try_reserve_exact(size).map_err(|_|::qsl::QslError::AllocationFailed)?;let used=unsafe{self.serialize_dyn_unchecked(out.as_mut_ptr())};unsafe{out.set_len(used);}Ok(out)}");
    ln(out,"/// # Safety
/// Values satisfy this schema, all tags/lengths/offsets are representable, and out holds serialized_size() exclusive writable bytes.
pub unsafe fn serialize_dyn_unchecked(&self,out:*mut u8)->usize{");
    let mut header = [0u8; 12];
    h.write(&mut header)?;
    ln(
        out,
        format!(
            "std::ptr::copy_nonoverlapping({header:?}.as_ptr(),out,12);let payload=out.add(12);"
        ),
    );
    ln(
        out,
        if h.mode == DynMode::Line {
            "let mut writer=::qsl::QslByteWriter::from_raw(payload);"
        } else {
            "let mut writer=::qsl::dynamic::DynBitWriter::from_raw(payload);"
        },
    );
    if h.mode != DynMode::Line {
        ln(out,format!("let mut entries=[::qsl::dynamic::DynEntry{{offset_bits:0,tag:0,size_bits:0}};{}];let mut count=0usize;",s.fields.len()));
    }
    for f in &s.fields {
        let value = if f.optional {
            ln(
                out,
                format!("if let Some(value)=self.{}.as_ref(){{", f.name),
            );
            "value".to_string()
        } else {
            ln(out, format!("let value=&self.{};", f.name));
            "value".to_string()
        };
        if h.mode != DynMode::Line && scalar_bits(s, f).is_none() {
            ln(out, "writer.align();");
        }
        if h.mode == DynMode::Line {
            ln(out, "let mut __qsl_bits=0usize;");
            emit_size_stored(f, out, &value, f.bits, false)?;
            ln(out,format!("writer.write_bytes_unchecked(&({} as u{}).to_le_bytes());writer.write_bytes_unchecked(&((__qsl_bits/8)as u{}).to_le_bytes());",f.tag,h.width as u8,h.width as u8));
        } else {
            ln(out,format!("entries[count]=::qsl::dynamic::DynEntry{{offset_bits:writer.bit_position()as u32,tag:{} as u32,size_bits:({})as u32}};count+=1;",f.tag,index_size_bits(s,f)?.replace(".checked_mul(8).ok_or(::qsl::QslError::LengthOverflow)?","*8")));
        }
        if let Some(width) = scalar_bits(s, f) {
            if let QslType::DomainInteger(d) = &f.data {
                ln(
                    out,
                    format!(
                        "writer.write_unsigned_unchecked({}, {width});",
                        numeric_codegen::encode(d, &numeric_codegen::copied(&value))
                    ),
                );
            } else {
                ln(out,format!("writer.write_unsigned_unchecked(((*{value} as i128).wrapping_sub({}))as u128&::qsl::domain::low_mask({width}),{width});",min_literal(f.data.rank_base())));
            }
            if s.flags & crate::FLAG_ALIGNMENT != 0 {
                ln(out, "writer.align();");
            }
        } else if h.mode == DynMode::Line {
            fast_stored_write(f, out, &value, f.bits, false)?;
        } else {
            ln(out,"let start=writer.as_mut_ptr();let used={let mut writer=::qsl::QslByteWriter::from_raw(start);");
            fast_stored_write(f, out, &value, f.bits, false)?;
            ln(out,"writer.as_mut_ptr().offset_from(start)as usize};writer.claim_initialized_bytes(used);");
        }
        if f.optional {
            ln(out, "}");
        }
    }
    if h.mode != DynMode::Line {
        ln(out,"writer.align();let mut writer=::qsl::QslByteWriter::from_raw(writer.as_mut_ptr());let table=writer.as_mut_ptr();for entry in entries.iter().take(count){");
        if h.mode == DynMode::StrictIndex {
            let bits = if w == 4 { 16 } else { 32 };
            ln(out,format!("writer.write_bytes_unchecked(&(entry.offset_bits as u{bits}).to_le_bytes());writer.write_bytes_unchecked(&(entry.tag as u{bits}).to_le_bytes());"));
        } else {
            let half = if w == 4 { 16 } else { 32 };
            let quarter = if w == 4 { 8 } else { 16 };
            ln(out,format!("writer.write_bytes_unchecked(&(entry.offset_bits as u{half}).to_le_bytes());writer.write_bytes_unchecked(&(entry.tag as u{quarter}).to_le_bytes());writer.write_bytes_unchecked(&(entry.size_bits as u{quarter}).to_le_bytes());"));
        }
        ln(out,"}let back=writer.as_mut_ptr().offset_from(table)as u32;writer.write_bytes_unchecked(&back.to_le_bytes());");
    }
    ln(out, "writer.as_mut_ptr().offset_from(out)as usize}");
    if s.flags & crate::FLAG_SAFETY != 0 {
        checked_read(s, out, false)?;
        checked_read(s, out, true)?;
        let borrow = if s.lifetime() { "'a " } else { "" };
        let scratch = if h.mode == DynMode::Line {
            "u64"
        } else {
            "u32"
        };
        ln(out,format!("pub fn deserialize_checked_into<'out>(input:&{borrow}[u8],scratch:&mut[{scratch}],out:&'out mut std::mem::MaybeUninit<Self>)->::qsl::QslResult<&'out mut Self>{{Ok(out.write(Self::deserialize_checked(input,scratch)?))}}"));
    }
    ln(out,"/// # Safety
/// Input is an immutable complete schema-valid dyn frame with this exact header and required tags. Every field satisfies its codec and allocation budget.
pub unsafe fn deserialize_dyn_unchecked(bytes:*const u8,len:usize)->Self{");
    ln(out, "let input=std::slice::from_raw_parts(bytes,len);");
    if h.mode == DynMode::Line {
        ln(out,"let view=::qsl::dynamic::DynLineView::from_validated_unchecked(input,Self::DYN_HEADER);");
    } else {
        ln(out,"let view=::qsl::dynamic::DynIndexView::from_validated_unchecked(input,Self::DYN_HEADER);");
    }
    for f in &s.fields {
        let ty = qsl_lexer::layout::field_rust_type(f, s.borrow_strings())?;
        ln(out, format!("let mut field_{}:Option<{ty}>=None;", f.tag));
    }
    if h.mode == DynMode::Line {
        ln(
            out,
            "for field in view.fields(){let bytes=field.bytes;match field.tag{",
        );
    } else {
        ln(out,"for row in 0..view.len(){let field=view.at(row).unwrap_unchecked();match field.entry.tag{");
    }
    for f in &s.fields {
        ln(out, format!("{}=>{{", f.tag));
        if scalar_bits(s, f).is_some() {
            scalar_read(s, f, out, false, true)?;
            ln(out, "},");
            continue;
        }
        if h.mode != DynMode::Line {
            ln(out, "let bytes=field.byte_extent().unwrap_unchecked();");
        }
        ln(
            out,
            "let mut reader=::qsl::QslByteReader::new(bytes.as_ptr());",
        );
        emit_stored_value(f, out, "value", f.bits, false, s.borrow_strings())?;
        ln(out, format!("field_{}=Some(value);}},", f.tag));
    }
    ln(out, "_=>{}}}Self{");
    for f in &s.fields {
        ln(
            out,
            if f.optional {
                format!("{}:field_{},", f.name, f.tag)
            } else {
                format!("{}:field_{}.unwrap_unchecked(),", f.name, f.tag)
            },
        );
    }
    ln(out, "}}");
    ln(out,"/// # Safety
/// Input follows deserialize_dyn_unchecked; out is aligned, exclusive and uninitialized, and disjoint from input.
pub unsafe fn deserialize_into(bytes:*const u8,len:usize,out:*mut Self){out.write(Self::deserialize_dyn_unchecked(bytes,len));}");
    ln(out, "}");
    Ok(())
}
