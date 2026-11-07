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
            let ty = crate::modernization::field_rust_type(f, false)?;
            let layout = if let QslType::FixedString(n) = f.data {
                format!("::qsl::modernization::NativeLayout::leaf({n},200,{n})")
            } else {
                format!("<{ty} as ::qsl::modernization::NativeField>::NATIVE_LAYOUT")
            };
            layouts.push(format!("(std::mem::offset_of!(Self,{}),{layout})", f.name));
        }
        ln(out,format!("unsafe impl ::qsl::modernization::NativeField for {name}{{const NATIVE_LAYOUT: ::qsl::modernization::NativeLayout=::qsl::modernization::NativeLayout::fields(&[{}]);unsafe fn validate_native_fields(address:*const u8)->::qsl::QslResult<()>{{let _=address;",layouts.join(",")));
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
   _=>{let ty=crate::modernization::field_rust_type(f,false)?;ln(out,format!("<{ty} as ::qsl::modernization::NativeField>::validate_native_fields({address})?;"));}
  }
        }
        ln(out, "Ok(())}}");
    }
    ln(out,format!("impl {name}{{pub fn __qsl_region_view(input:&[u8])->::qsl::QslResult<&Self>{{if !Self::NATIVE_WIRE_COMPATIBLE{{return Err(::qsl::QslError::InvalidSchema)}}if input.len()!={size}||input.as_ptr().align_offset(std::mem::align_of::<Self>())!=0{{return Err(::qsl::QslError::InvalidValue)}}Self::validate(input)?;Ok(unsafe{{&*input.as_ptr().cast::<Self>()}})}}}}"));
    Ok(())
}
