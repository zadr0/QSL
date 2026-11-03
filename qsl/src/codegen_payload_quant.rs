//! Build-time exact scalar packing. The prepared kernel has no dynamic loop,
//! tag scan, allocation, or register-width selection. Exact-slice I/O remains
//! available through Serializable; its cursor never reads the private padding.
use super::*;
use crate::payload_quant::{PayloadQuantPlan, QuantField};

fn mask(bits: usize) -> u128 {
    if bits == 0 {
        0
    } else {
        (1u128 << bits) - 1
    }
}
fn operands(f: &QuantField, q: usize, inverse: bool) -> String {
    let k = (q.trailing_zeros() as usize).min(f.bits);
    let a = f.bits - k;
    if f.bits == 0 {
        return "0u64".into();
    }
    if inverse {
        format!("((code & {}u64)<<{k}) | (code>>{a})", mask(a))
    } else {
        format!("(rank>>{k}) | ((rank & {}u64)<<{a})", mask(k))
    }
}
fn present(f: &QuantField, ordinal: usize) -> String {
    if f.optional {
        format!("((presence>>{ordinal})&1) as usize")
    } else {
        "1usize".into()
    }
}
fn presence(s: &QslSchema) -> String {
    let mut terms = vec!["0u64".into()];
    terms.extend(
        s.fields
            .iter()
            .filter(|f| f.optional)
            .enumerate()
            .map(|(i, f)| format!("((self.{}.is_some() as u64)<<{i})", f.name)),
    );
    terms.join("|")
}
fn offset_expressions(p: &PayloadQuantPlan) -> Vec<String> {
    let mut terms = vec![format!("{}usize", p.header_bytes * 8)];
    let mut result = Vec::new();
    let mut ordinal = 0;
    for f in &p.fields {
        result.push(terms.join("+"));
        if f.optional {
            terms.push(format!("{}*(((presence>>{ordinal})&1) as usize)", f.bits));
            ordinal += 1;
        } else {
            terms.push(f.bits.to_string());
        }
    }
    result.push(terms.join("+"));
    result
}
fn header_read(p: &PayloadQuantPlan, out: &mut String, fast: bool) {
    if fast {
        let terms = (0..p.header_bytes)
            .map(|i| format!("((*bytes.add({i}) as u64)<<{})", i * 8))
            .collect::<Vec<_>>();
        ln(
            out,
            format!(
                "let presence=0u64{};",
                terms
                    .into_iter()
                    .map(|x| format!("|{x}"))
                    .collect::<String>()
            ),
        );
    } else if p.header_bytes > 0 {
        ln(
            out,
            format!("let presence=reader.read_bits({});", p.header_bytes * 8),
        );
    } else {
        ln(out, "let presence=0u64;");
    }
}
fn source(f: &QuantField, s: &QslSchema, q: usize, out: &mut String) {
    let name = &s.fields[f.index].name;
    if f.optional {
        ln(out,format!("let zero={}i128 as {};let source=std::hint::select_unpredictable(p!=0,self.{name}.as_slice().as_ptr(),&zero as *const {});let source=::qsl::payload_quant::selected_address(source);let value=source.read();",f.min,f.rust_type,f.rust_type));
    } else {
        ln(out, format!("let value=self.{name};"));
    }
    ln(
        out,
        format!(
            "let rank=((value as i128)-({}i128)) as u64;let code={};",
            f.min,
            operands(f, q, false)
        ),
    );
}
fn chunks(f: &QuantField) -> Vec<(usize, usize)> {
    if f.bits == 0 {
        return Vec::new();
    }
    let width = if f.word_bytes * 8 >= f.bits + f.max_start {
        f.bits
    } else {
        f.word_bytes * 8 - 7
    };
    (0..f.bits)
        .step_by(width)
        .map(|start| (start, (f.bits - start).min(width)))
        .collect()
}
fn packed_word(p: &PayloadQuantPlan) -> Option<usize> {
    // The private tail already exists. A whole-register operation is legal
    // only if even the shortest frame has initialized capacity for that word.
    [1, 2, 4, 8, 16]
        .into_iter()
        .find(|&bytes| bytes >= p.max_bytes && bytes <= p.min_bytes + p.padding_bytes)
}
fn prepared_word_write(s: &QslSchema, p: &PayloadQuantPlan, bytes: usize, out: &mut String) {
    let word = format!("u{}", bytes * 8);
    ln(out,format!("#[inline] unsafe fn __qsl_quant_write(&self,output:&mut[u8])->usize{{let presence={};let mut packed=presence as {word};",presence(s)));
    let mut ordinal = 0;
    for f in &p.fields {
        if f.bits != 0 {
            ln(out, format!("{{let p={};", present(f, ordinal)));
            source(f, s, p.quantum, out);
            ln(
                out,
                format!(
                    "packed|=(code as {word})<<Self::__qsl_quant_offset_{}(presence);}}",
                    f.index
                ),
            );
        }
        if f.optional {
            ordinal += 1;
        }
    }
    ln(out,format!("output.as_mut_ptr().cast::<{word}>().write_unaligned(packed.to_le());self.__qsl_size()}}"));
}
fn prepared_word_read(s: &QslSchema, p: &PayloadQuantPlan, bytes: usize, out: &mut String) {
    let word = format!("u{}", bytes * 8);
    ln(out,format!("#[inline] unsafe fn __qsl_quant_read(bytes:*const u8,output:&mut std::mem::MaybeUninit<Self>){{let out=output.as_mut_ptr();let packed=<{word}>::from_le(bytes.cast::<{word}>().read_unaligned());let presence=(packed as u64)&{}u64;",mask(p.header_bytes*8)));
    let mut ordinal = 0;
    for f in &p.fields {
        let code = if f.bits == 0 {
            "0u64".into()
        } else {
            format!(
                "((packed>>Self::__qsl_quant_offset_{}(presence))&{} as {word}) as u64",
                f.index,
                mask(f.bits)
            )
        };
        ln(
            out,
            format!(
                "{{let p={};let code={code};let rank={};let value=((rank as i128)+({}i128)) as {};",
                present(f, ordinal),
                operands(f, p.quantum, true),
                f.min,
                f.rust_type
            ),
        );
        if f.optional {
            ordinal += 1;
            ln(
                out,
                format!(
                    "let value=::qsl::payload_quant::scalar_operand(value as u64) as {};",
                    f.rust_type
                ),
            );
        }
        let value = if f.optional {
            "(p!=0).then_some(value)"
        } else {
            "value"
        };
        ln(
            out,
            format!(
                "std::ptr::addr_of_mut!((*out).{}).write({value});}}",
                s.fields[f.index].name
            ),
        );
    }
    ln(out, "}");
}
fn prepared(s: &QslSchema, p: &PayloadQuantPlan, out: &mut String) {
    if let Some(bytes) = packed_word(p) {
        prepared_word_write(s, p, bytes, out);
    } else {
        prepared_field_write(s, p, out);
    }
    if let Some(bytes) = reader_word(p) {
        prepared_word_read(s, p, bytes, out);
    } else {
        prepared_field_read(s, p, out);
    }
}
fn reader_word(p: &PayloadQuantPlan) -> Option<usize> {
    // Variable optional prefixes made a u128 aggregate reader slower on the
    // measured target. Keep field addressing there, independently of writing.
    if p.fields.iter().any(|f| f.optional && f.bits != 0) {
        None
    } else {
        packed_word(p)
    }
}
fn prepared_field_write(s: &QslSchema, p: &PayloadQuantPlan, out: &mut String) {
    ln(out,"/// # Safety\n/// Values are valid; output is exclusive, initialized, disjoint and includes QUANT_PADDING_BYTES.\n#[inline] unsafe fn __qsl_quant_write(&self,output:&mut [u8])->usize {let to=output.as_mut_ptr();");
    ln(out, format!("let presence={};", presence(s)));
    if p.header_bytes > 0 {
        ln(
            out,
            format!(
                "std::ptr::copy_nonoverlapping(presence.to_le_bytes().as_ptr(),to,{});",
                p.header_bytes
            ),
        );
    }
    let mut ordinal = 0;
    for f in &p.fields {
        ln(
            out,
            format!("{{let bit=Self::__qsl_quant_offset_{}(presence);", f.index),
        );
        ln(out, format!("let p={};", present(f, ordinal)));
        if f.optional {
            ordinal += 1;
        }
        source(f, s, p.quantum, out);
        let word = format!("u{}", f.word_bytes * 8);
        for (start, width) in chunks(f) {
            // Absent fields select the native minimum, hence code==0. A full
            // Boolean mask is cheaper than u128 multiplication by 0/1 on x64.
            ln(out,format!("let at=bit+{start}*p;let shift=at&7;let address=to.add(at>>3).cast::<{word}>();let old=<{word}>::from_le(address.read_unaligned());let active=(({} as {word})<<shift)&(0 as {word}).wrapping_sub(p as {word});let value=(((code>>{start})&{}u64) as {word})<<shift;address.write_unaligned(((old & !active)|value).to_le());",mask(width),mask(width)));
        }
        ln(out, "}");
    }
    ln(
        out,
        format!(
            "let bit=Self::__qsl_quant_offset_{}(presence);",
            p.fields.len()
        ),
    );
    ln(
        out,
        "*to.add(bit>>3)&=((1u16<<(bit&7))-1) as u8;(bit+7)>>3 }",
    );
}
fn prepared_field_read(s: &QslSchema, p: &PayloadQuantPlan, out: &mut String) {
    ln(out,"/// # Safety\n/// Input is validated and includes initialized padding; output is disjoint, aligned storage.\n#[inline] unsafe fn __qsl_quant_read(bytes:*const u8,output:&mut std::mem::MaybeUninit<Self>) {let out=output.as_mut_ptr();");
    header_read(p, out, true);
    let mut ordinal = 0;
    for f in &p.fields {
        ln(
            out,
            format!("{{let bit=Self::__qsl_quant_offset_{}(presence);", f.index),
        );
        ln(
            out,
            format!("let p={};let mut code=0u64;", present(f, ordinal)),
        );
        if f.optional {
            ordinal += 1;
        }
        let word = format!("u{}", f.word_bytes * 8);
        for (start, width) in chunks(f) {
            ln(out,format!("let at=bit+{start}*p;let word=<{word}>::from_le(bytes.add(at>>3).cast::<{word}>().read_unaligned());code|=(((word as u128>>(at&7))&{}u128) as u64)<<{start};",mask(width)));
        }
        ln(
            out,
            format!(
                "let rank={};let value=((rank as i128)+({}i128)) as {};",
                operands(f, p.quantum, true),
                f.min,
                f.rust_type
            ),
        );
        let name = &s.fields[f.index].name;
        let value = if f.optional {
            "(p!=0).then_some(value)"
        } else {
            "value"
        };
        ln(out,format!("let value=::qsl::payload_quant::scalar_operand(value as u64) as {rust};std::ptr::addr_of_mut!((*out).{name}).write({value});}}",rust=f.rust_type));
    }
    ln(out, "}");
}
fn exact(s: &QslSchema, p: &PayloadQuantPlan, out: &mut String) {
    ln(out,"#[inline] unsafe fn __qsl_write_unchecked(&self,to:*mut u8)->usize {let mut writer=::qsl::QslUncheckedWriter::from_raw(to);");
    ln(out, format!("let presence={};", presence(s)));
    if p.header_bytes > 0 {
        ln(
            out,
            format!(
                "writer.write_bits_unchecked(presence,{});",
                p.header_bytes * 8
            ),
        );
    }
    let mut ordinal = 0;
    for f in &p.fields {
        ln(out, "{");
        ln(out, format!("let p={};", present(f, ordinal)));
        if f.optional {
            ordinal += 1;
        }
        source(f, s, p.quantum, out);
        ln(
            out,
            format!("writer.write_bits_unchecked(code,({}*p) as u32);}}", f.bits),
        );
    }
    ln(
        out,
        "writer.flush_unchecked();writer.as_mut_ptr().offset_from(to) as usize }",
    );
    ln(out,"#[inline] unsafe fn __qsl_read_unchecked(bytes:*const u8)->Self {let mut reader=::qsl::QslFastBitsReader::new(bytes);");
    header_read(p, out, false);
    ordinal = 0;
    for f in &p.fields {
        ln(out,format!("let p={};let code=reader.read_bits(({}*p) as u32);let rank={};let value=((rank as i128)+({}i128)) as {};",present(f,ordinal),f.bits,operands(f,p.quantum,true),f.min,f.rust_type));
        if f.optional {
            ordinal += 1;
        }
        ln(
            out,
            format!(
                "let field_{}={};",
                f.index,
                if f.optional {
                    "(p!=0).then_some(value)"
                } else {
                    "value"
                }
            ),
        );
    }
    ln(out, "Self{");
    for f in &p.fields {
        ln(
            out,
            format!("{}:field_{},", s.fields[f.index].name, f.index),
        );
    }
    ln(out, "} }");
}
fn validation(s: &QslSchema, p: &PayloadQuantPlan, out: &mut String) -> QslResult<()> {
    ln(out,"pub fn validate(input:&[u8])->::qsl::QslResult<()> {let mut c=::qsl::validation::ValidationCursor::with_limit(input,Self::MAX_DATA_SIZE)?;");
    if p.header_bytes > 0 {
        ln(
            out,
            format!("let presence=c.bits({})?;", p.header_bytes * 8),
        );
    } else {
        ln(out, "let presence=0u64;");
    }
    let n = s.optional_count();
    if n < 64 {
        ln(
            out,
            format!("if presence>>{n}!=0{{return Err(::qsl::QslError::InvalidValue);}}"),
        );
    }
    let mut ordinal = 0;
    for f in &p.fields {
        if f.optional {
            ln(out, format!("if (presence>>{ordinal})&1!=0 {{"));
            ordinal += 1;
        }
        ln(out,format!("c.charge({})?;let code=c.bits({})?;let rank={};if rank>{}u64{{return Err(::qsl::QslError::InvalidValue);}}",f.native_bytes,f.bits,operands(f,p.quantum,true),f.max-f.min));
        if f.optional {
            ln(out, "}");
        }
    }
    validation_codegen::finish(s, out)
}
pub(super) fn generate(
    i: &QslInterpreter,
    s: &QslSchema,
    p: &PayloadQuantPlan,
    out: &mut String,
) -> QslResult<()> {
    QslInterpreter::generate_struct(s, out)?;
    // Native offsets are compiler-derived: repr(Rust) may reorder fields.
    let name = &s.name;
    let graph = p
        .fields
        .iter()
        .map(|f| {
            format!(
                "(std::mem::offset_of!({name},{}),{},{},{})",
                s.fields[f.index].name, f.bits, f.word_bytes, f.optional
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_QUANT_GRAPH:[(usize,usize,usize,bool);{}]=[{graph}];",p.fields.len()));
    if s.opt_table() {
        let callbacks = (0..p.fields.len())
            .map(|n| format!("{name}::__qsl_quant_offset_{n}"))
            .collect::<Vec<_>>()
            .join(",");
        ln(out,format!("/// Encoded BIT offsets from Boolean presence; never raw offsets inferred from packed byte length.\n#[allow(dead_code,non_upper_case_globals)] static {name}_OFFSET_TABLE:[fn(u64)->usize;{}]=[{callbacks}];",p.fields.len()));
        let critical = p
            .fields
            .iter()
            .map(|f| (f.bits, f.native_bytes))
            .collect::<Vec<_>>();
        ln(out,format!("/// Encoded bit widths and native restoration widths.\n#[allow(dead_code,non_upper_case_globals)] static {name}_CRITICAL_POINTS:[(usize,usize);{}]={critical:?};",p.fields.len()));
    }
    if i.config.qsl_rt {
        let cap = p.max_bytes + p.padding_bytes;
        let slots = i.config.qsl_rt_default;
        if cap.checked_mul(slots).is_none_or(|n| n > 64 * 1024 * 1024) {
            return Err(QslError::InvalidSchema);
        }
        ln(out,format!("#[allow(non_upper_case_globals)] static {name}_QUANT_REGIONS: ::qsl::QuantizedRegions<{name},{cap},{slots}>=::qsl::QuantizedRegions::new();"));
    }
    QslInterpreter::emit_default_impl(s, out)?;
    let registers = reader_word(p).map_or_else(
        || {
            p.fields
                .iter()
                .map(|f| f.word_bytes * 8)
                .collect::<Vec<_>>()
        },
        |w| vec![w * 8],
    );
    ln(out,format!("#[allow(unused_variables,unused_mut,unused_comparisons,clippy::identity_op,clippy::erasing_op,clippy::unnecessary_cast,clippy::absurd_extreme_comparisons)] impl {name} {{pub const QUANT_ACTIVE:bool=true;pub const QUANTUM:usize={};pub const QUANT_PADDING_BYTES:usize={};pub const QUANT_MAX_BYTES:usize={};pub const QUANT_RESTORE_BYTES:usize=std::mem::size_of::<Self>();pub const QUANT_READER_REGISTERS:&'static[usize]=&{registers:?};pub const QUANT_WHOLE_WORD_READ:bool={};pub const QUANT_WHOLE_WORD_WRITE:bool={};",p.quantum,p.padding_bytes,p.max_bytes,reader_word(p).is_some(),packed_word(p).is_some()));
    for (index, expression) in offset_expressions(p).iter().enumerate() {
        ln(
            out,
            format!("#[inline] fn __qsl_quant_offset_{index}(presence:u64)->usize{{{expression}}}"),
        );
    }
    if s.state_bits() {
        ln(
            out,
            format!(
                "pub const D_WORD_BYTES:usize=4;pub const D_USED_BITS:u32={};",
                s.optional_count()
            ),
        );
    }
    ln(
        out,
        format!(
            "#[inline] fn __qsl_size(&self)->usize {{let presence={};let mut bits={}usize;",
            presence(s),
            p.header_bytes * 8
        ),
    );
    let mut ordinal = 0;
    for f in &p.fields {
        ln(out, format!("bits+={}*({});", f.bits, present(f, ordinal)));
        if f.optional {
            ordinal += 1;
        }
    }
    ln(out, "(bits+7)>>3 }");
    validation_codegen::values(s, out)?;
    validation(s, p, out)?;
    if i.config.qsl_rt {
        ln(out,format!("pub fn serialize(&self)->::qsl::QslResult<::qsl::QuantizedLease<Self,{},{}>>{{{name}_QUANT_REGIONS.try_encode(self)}}",p.max_bytes+p.padding_bytes,i.config.qsl_rt_default));
    } else {
        ln(out,"pub fn serialize(&self)->::qsl::QslResult<::qsl::QuantizedOut<Self>>{::qsl::QuantizedOut::encode(self)}");
    }
    ln(out,"/// Reuse initialized storage. Validation and capacity checks precede the prepared kernel.\npub fn serialize_quantized_to<'out>(&self,out:&'out mut [u8])->::qsl::QslResult<::qsl::QuantizedFrame<'out,Self>> {self.validate_values()?;let len=self.__qsl_size();::qsl::validation::check_size(len,Self::MAX_DATA_SIZE)?;let end=len+Self::QUANT_PADDING_BYTES;let storage=out.get_mut(..end).ok_or(::qsl::QslError::BufferTooSmall)?;unsafe{self.__qsl_quant_write(storage);Ok(::qsl::QuantizedFrame::from_validated(storage,len))}}\n/// # Safety\n/// Input is valid and has len+QUANT_PADDING_BYTES initialized readable bytes; destination is disjoint and aligned.\n#[inline] pub unsafe fn deserialize_into<D: ::qsl::payload_quant::QuantizedDestination<Self>>(bytes:*const u8,len:usize,out:D){out.decode_padded(bytes,len);}");
    exact(s, p, out);
    prepared(s, p, out);

    ln(out, "}");
    ln(out,format!("impl ::qsl::Serializable for {name} {{const OPTIONAL_FIELDS_COUNT:usize={};const SIZE_TAG:usize=0;fn get_size()->usize{{{}}}#[inline] fn serialized_size(&self)->usize{{self.__qsl_size()}}#[inline] unsafe fn serialize_unsafe(&self,to:*mut u8){{self.__qsl_write_unchecked(to);}}#[inline] unsafe fn serialize_into_unsafe(&self,to:*mut u8)->usize{{self.__qsl_write_unchecked(to)}}#[inline] unsafe fn deserialize_unsafe(bytes:*const u8)->Self{{Self::__qsl_read_unchecked(bytes)}}}}",s.optional_count(),p.min_bytes));
    ln(out,format!("unsafe impl ::qsl::payload_quant::QuantizedRecord for {name} {{const MAX_DATA_SIZE:usize=Self::MAX_DATA_SIZE;const QUANTUM:usize={};const PADDING_BYTES:usize={};const MAX_QUANTIZED_BYTES:usize={};fn quant_validate_values(&self)->::qsl::QslResult<()>{{self.validate_values()}}#[inline] unsafe fn quant_write_prepared(&self,out:*mut u8)->usize{{self.__qsl_quant_write(std::slice::from_raw_parts_mut(out,self.__qsl_size()+Self::QUANT_PADDING_BYTES))}}#[inline] unsafe fn quant_read_padded(bytes:*const u8,_len:usize,out:*mut Self){{Self::__qsl_quant_read(bytes,&mut *out.cast::<std::mem::MaybeUninit<Self>>());}}}}",p.quantum,p.padding_bytes,p.max_bytes));
    ln(out,format!("impl ::qsl::payload_quant::CheckedQuantizedRecord for {name}{{fn quant_validate(input:&[u8])->::qsl::QslResult<()>{{Self::validate(input)}}}}"));
    Ok(())
}
