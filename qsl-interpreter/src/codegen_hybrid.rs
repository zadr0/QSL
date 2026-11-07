//! Compose a QPS scalar prefix with the independent ordinary tail codec.
use super::*;
use crate::hybrid::HybridPlan;
pub(super) fn generate(
    i: &QslInterpreter,
    s: &QslSchema,
    p: &HybridPlan,
    out: &mut String,
) -> QslResult<()> {
    let name = &s.name;
    let prefix = &p.prefix.name;
    let tail = &p.tail.name;
    let lt = if s.lifetime() { "<'a>" } else { "" };
    let lifetime = if s.lifetime() { "'a " } else { "" };
    let h = p.length_header;
    let n = p.prefix_bytes;
    let min = p.prefix_minimum;
    let mut inner = QslInterpreter::new();
    inner.config = i.config;
    inner.config.qsl_rt = false;
    inner.generate_impl(&p.prefix, out)?;
    inner.generate_impl(&p.tail, out)?;
    QslInterpreter::generate_struct(s, out)?;
    i.emit_schema_hashes(s, out)?;
    ln(out,format!("impl{lt} {name}{lt}{{pub const QPS_SCALAR_PREFIX_BYTES:usize={n};pub const QPS_RAW_PREFIX_BYTES:usize={};pub const QPS_PREFIX_LENGTH_HEADER:usize={h};pub const QPS_LENGTHS_QUANTIZED:bool=false;",p.raw_prefix_bytes));
    validation_codegen::values(s, out)?;
    ln(out, format!("fn __qsl_prefix(&self)->{prefix}{{{prefix}{{"));
    for f in &p.prefix.fields {
        ln(out, format!("{}:self.{},", f.name, f.name));
    }
    ln(out, "}}");
    ln(out,format!("fn __qsl_with_tail<R>(&self,f:impl FnOnce(&{tail}{lt})->R)->R{{let tail=std::mem::ManuallyDrop::new({tail}{{"));
    for f in &p.tail.fields {
        ln(
            out,
            format!("{}:unsafe{{std::ptr::read(&self.{})}},", f.name, f.name),
        );
    }
    ln(out, "});f(&tail)}");
    ln(
        out,
        format!("fn __qsl_join(prefix:{prefix},tail:{tail}{lt})->Self{{Self{{"),
    );
    for f in &p.prefix.fields {
        ln(out, format!("{}:prefix.{},", f.name, f.name));
    }
    for f in &p.tail.fields {
        ln(out, format!("{}:tail.{},", f.name, f.name));
    }
    ln(out, "}}");
    ln(out,"fn __qsl_split(input:&[u8])->::qsl::QslResult<(&[u8],&[u8])>{::qsl::validation::check_size(input.len(),Self::MAX_DATA_SIZE)?;");
    if h == 0 {
        ln(out, format!("let length={n};"));
    } else {
        ln(out,format!("let mut c=::qsl::validation::ValidationCursor::new(input);let length=c.uint({h})? as usize;"));
    }
    ln(out,format!("if !({min}..={n}).contains(&length){{return Err(::qsl::QslError::InvalidValue)}}let end={h}usize.checked_add(length).ok_or(::qsl::QslError::LengthOverflow)?;let head=input.get({h}..end).ok_or(::qsl::QslError::UnexpectedEof)?;Ok((head,&input[end..]))}}"));
    let qp = crate::payload_quant::PayloadQuantPlan::analyze(&p.prefix)?;
    ln(
        out,
        "fn __qsl_prefix_charge(head:&[u8])->::qsl::QslResult<usize>{let _=head;",
    );
    if qp.header_bytes == 0 {
        ln(out, "let presence=0u64;let _=presence;");
    } else {
        ln(out,format!("let mut c=::qsl::validation::ValidationCursor::new(head);let presence=c.bits({})?;",qp.header_bytes*8));
    }
    let mut terms = Vec::new();
    let mut ordinal = 0;
    for f in &qp.fields {
        if f.optional {
            terms.push(format!(
                "{}usize*((presence>>{ordinal})&1)as usize",
                f.native_bytes
            ));
            ordinal += 1;
        } else {
            terms.push(format!("{}usize", f.native_bytes));
        }
    }
    ln(out, format!("Ok({})}}", terms.join("+")));
    ln(out,format!("pub fn validate(input:&[u8])->::qsl::QslResult<()>{{let(head,tail)=Self::__qsl_split(input)?;{prefix}::validate(head)?;let limit=Self::MAX_DATA_SIZE.checked_sub(Self::__qsl_prefix_charge(head)?).ok_or(::qsl::QslError::SizeLimitExceeded)?;{tail}::__qsl_validate_with_limit(tail,limit)}}"));
    if s.flags & crate::FLAG_SAFETY != 0 {
        ln(out,format!("fn __qsl_read_checked(input:&{lifetime}[u8])->::qsl::QslResult<Self>{{let(head,tail)=Self::__qsl_split(input)?;let prefix={prefix}::deserialize_checked(head)?;let limit=Self::MAX_DATA_SIZE.checked_sub(Self::__qsl_prefix_charge(head)?).ok_or(::qsl::QslError::SizeLimitExceeded)?;let tail={tail}::__qsl_read_checked_with_limit(tail,limit)?;Ok(Self::__qsl_join(prefix,tail))}}"));
    }
    validation_codegen::finish(s, out)?;
    if i.config.qsl_rt {
        qps_codegen::region_method(s, out, i.config.qsl_rt_default)?;
    }
    ln(out, "}");
    let target = if s.lifetime() {
        "::qsl::BorrowSerializable<'a>"
    } else {
        "::qsl::Serializable"
    };
    let method = if s.lifetime() {
        "deserialize_borrowed_unsafe"
    } else {
        "deserialize_unsafe"
    };
    let raw = if s.lifetime() {
        "deserialize_borrowed_from_raw_parts"
    } else {
        "deserialize_from_raw_parts"
    };
    let get_length = if h == 0 {
        format!("{n}")
    } else {
        let word = crate::modernization::register_word(h)?;
        format!("::qsl::modernization::load_{word}::<{h}>(bytes)as usize")
    };
    ln(out,format!("impl{lt} {target} for {name}{lt}{{const OPTIONAL_FIELDS_COUNT:usize={};const SIZE_TAG:usize=<{tail}{lt} as {target}>::SIZE_TAG;fn get_size()->usize{{{h}+{min}+<{tail}{lt} as {target}>::get_size()}}fn serialized_size(&self)->usize{{{h}+< {prefix} as ::qsl::Serializable>::serialized_size(&self.__qsl_prefix())+self.__qsl_with_tail(|tail|<{tail}{lt} as {target}>::serialized_size(tail))}}unsafe fn serialize_unsafe(&self,to:*mut u8){{self.serialize_into_unsafe(to);}}unsafe fn serialize_into_unsafe(&self,to:*mut u8)->usize{{let prefix=self.__qsl_prefix();let length=<{prefix} as ::qsl::Serializable>::serialized_size(&prefix);::qsl::modernization::store_word(to,length as u128,{h});<{prefix} as ::qsl::Serializable>::serialize_into_unsafe(&prefix,to.add({h}));{h}+length+self.__qsl_with_tail(|tail|<{tail}{lt} as {target}>::serialize_into_unsafe(tail,to.add({h}+length)))}}unsafe fn {method}(bytes:*const u8)->Self{{let length={get_length};let prefix=<{prefix} as ::qsl::Serializable>::deserialize_unsafe(bytes.add({h}));let tail=<{tail}{lt} as {target}>::{method}(bytes.add({h}+length));Self::__qsl_join(prefix,tail)}}unsafe fn {raw}(bytes:*const u8,len:usize)->Self{{let length={get_length};let prefix=<{prefix} as ::qsl::Serializable>::deserialize_unsafe(bytes.add({h}));let tail=<{tail}{lt} as {target}>::{raw}(bytes.add({h}+length),len-{h}-length);Self::__qsl_join(prefix,tail)}}}}",s.optional_count()));
    if i.config.qsl_rt {
        qps_codegen::region_static(s, out, i.config.qsl_rt_default)?;
    }
    Ok(())
}
