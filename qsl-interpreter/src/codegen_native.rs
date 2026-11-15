use super::*;
pub(super) fn extra(s: &QslSchema, out: &mut String) -> QslResult<()> {
    if !s.options.generated_region {
        return Ok(());
    }
    let name = &s.name;
    let l = s
        .options
        .native_types
        .get(name)
        .ok_or(QslError::InvalidSchema)?;
    let size = l.wire_bytes;
    ln(out,format!("impl ::qsl::QslCustom for {name}{{const SIZE:usize={size};fn qsl_validate(bytes:&[u8],compact:bool)->::qsl::QslResult<()>{{if compact{{return Err(::qsl::QslError::InvalidSchema)}}Self::validate(bytes)}}fn qsl_validate_values(&self)->::qsl::QslResult<()>{{self.validate_values()}}unsafe fn qsl_write_unchecked(&self,to:*mut u8){{<Self as ::qsl::Serializable>::serialize_unsafe(self,to)}}unsafe fn qsl_read_unchecked(from:*const u8)->Self{{<Self as ::qsl::Serializable>::deserialize_from_raw_parts(from,Self::SIZE)}}}}"));
    if !s.options.external_definition {
        let mut layouts = Vec::new();
        for f in &s.fields {
            if matches!(f.data, QslType::Padding(_) | QslType::None) {
                continue;
            }
            let ty = qsl_lexer::layout::field_rust_type(f, false)?;
            let layout = if let QslType::FixedString(n) = f.data {
                format!("::qsl::storage::NativeLayout::leaf({n},200,{n})")
            } else {
                format!("<{ty} as ::qsl::storage::NativeField>::NATIVE_LAYOUT")
            };
            layouts.push(format!("(std::mem::offset_of!(Self,{}),{layout})", f.name));
        }
        ln(out,format!("unsafe impl ::qsl::storage::NativeField for {name}{{const NATIVE_LAYOUT: ::qsl::storage::NativeLayout=::qsl::storage::NativeLayout::fields(&[{}]);unsafe fn validate_native_fields(address:*const u8)->::qsl::QslResult<()>{{let _=address;",layouts.join(",")));
        for f in &s.fields {
            let address = format!("address.add(std::mem::offset_of!(Self,{}))", f.name);
            match &f.data{
   QslType::Padding(_)|QslType::None=>{},
   QslType::FixedString(n)=>ln(out,format!("std::str::from_utf8(std::slice::from_raw_parts({address},{n})).map_err(|_|::qsl::QslError::InvalidUtf8)?;")),
   QslType::Byte|QslType::Integer{..}|QslType::RangedInteger{..}|QslType::Flags(_)=>{
    let ty=f.data.primitive()?.ok_or(QslError::InvalidSchema)?;
    let domain=match f.data {QslType::RangedInteger{min,max,..}=>Some((min,max)),QslType::Flags(bits)=>Some((0,(1i128<<bits)-1)),_=>None};
    if let Some((min,max))=domain{ln(out,format!("let value=std::ptr::read_unaligned({address}.cast::<{ty}>())as i128;if !({min}i128..={max}i128).contains(&value){{return Err(::qsl::QslError::InvalidValue)}}"));}
   },
   _=>{let ty=qsl_lexer::layout::field_rust_type(f,false)?;ln(out,format!("<{ty} as ::qsl::storage::NativeField>::validate_native_fields({address})?;"));}
  }
        }
        ln(out, "Ok(())}}");
    }
    ln(out,format!("impl {name}{{pub fn __qsl_region_view(input:&[u8])->::qsl::QslResult<&Self>{{if !Self::NATIVE_WIRE_COMPATIBLE{{return Err(::qsl::QslError::InvalidSchema)}}if input.len()!={size}||input.as_ptr().align_offset(std::mem::align_of::<Self>())!=0{{return Err(::qsl::QslError::InvalidValue)}}Self::validate(input)?;Ok(unsafe{{&*input.as_ptr().cast::<Self>()}})}}}}"));
    Ok(())
}

pub(super) fn pointing_methods(
    s: &QslSchema,
    p: &crate::optimizer::PointingPlan,
    out: &mut String,
) -> QslResult<()> {
    let bytes = p.memory.bytes.ok_or(QslError::InvalidSchema)?;
    ln(
        out,
        "#[cfg(all(target_pointer_width=\"64\",target_endian=\"little\"))]",
    );
    let alignment = p.memory.alignment;
    ln(out,format!("const __QSL_POINTING_LAYOUT:()={{assert!(!std::mem::needs_drop::<Self>());assert!(std::mem::size_of::<Self>()=={bytes});assert!(std::mem::align_of::<Self>()=={alignment});"));
    for (i, f) in s.fields.iter().enumerate() {
        let loc = &p.memory.fields[i];
        let offset = loc.offset.ok_or(QslError::InvalidSchema)?;
        let size = loc.bytes.ok_or(QslError::InvalidSchema)?;
        let ty = qsl_lexer::layout::field_rust_type(f, false)?;
        ln(out,format!("assert!(std::mem::offset_of!(Self,{})=={offset});assert!(std::mem::size_of::<{ty}>()=={size});",f.name));
    }
    ln(out, "};");
    ln(out,"/// # Safety
/// Input contains one complete valid raw frame, and output is aligned, uninitialized, exclusive, and disjoint.
/// Layout assertions and the initialized-byte map justify each exact copy; padding is never copied.
unsafe fn __qsl_pointing_read_into(bytes:*const u8,out:*mut Self){let out=out.cast::<u8>();");
    ln(
        out,
        "#[cfg(all(target_pointer_width=\"64\",target_endian=\"little\"))] {",
    );
    ln(out, "let ()=Self::__QSL_POINTING_LAYOUT;");
    for c in &p.segments {
        ln(
            out,
            format!(
                "std::ptr::copy_nonoverlapping(bytes.add({}),out.add({}),{});",
                c.wire_offset, c.native_offset, c.bytes
            ),
        );
    }
    ln(out, "}");
    ln(out,"#[cfg(not(all(target_pointer_width=\"64\",target_endian=\"little\")))] { std::ptr::write(out.cast::<Self>(),Self::__qsl_read_unchecked(bytes)); }");
    ln(out, "}");
    Ok(())
}
pub(super) fn pointing_write(p: &crate::optimizer::PointingPlan, out: &mut String) {
    ln(
        out,
        "let ()=Self::__QSL_POINTING_LAYOUT;let source=(self as *const Self).cast::<u8>();",
    );
    for c in &p.segments {
        ln(
            out,
            format!(
                "std::ptr::copy_nonoverlapping(source.add({}),to.add({}),{});",
                c.native_offset, c.wire_offset, c.bytes
            ),
        );
    }
}
pub(super) fn pointing_read(out: &mut String) {
    ln(out,"let mut out=std::mem::MaybeUninit::<Self>::uninit();Self::__qsl_pointing_read_into(bytes,out.as_mut_ptr());out.assume_init()");
}
pub(super) fn pointing_checked_prefix(p: &crate::optimizer::PointingPlan, out: &mut String) {
    ln(out,format!("::qsl::validation::check_size(input.len(),Self::MAX_DATA_SIZE)?;if input.len()!={}{{return Err(if input.len()<{}{{::qsl::QslError::UnexpectedEof}}else{{::qsl::QslError::InvalidValue}});}}",p.wire_bytes,p.wire_bytes));
}
