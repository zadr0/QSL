//! Native optional storage is independent of the ordinary wire reader/writer.
//! The temporary wire view is read-only and never drops aliased owned fields.
use super::*;
use crate::modernization::{field_rust_type, optional_representation, OptionalRepresentation};
pub(super) fn applies(s: &QslSchema) -> bool {
    s.fields
        .iter()
        .any(|f| optional_representation(f, s.options.aggressive).is_some())
}
fn getter(r: &OptionalRepresentation, name: &str, bit: Option<usize>) -> String {
    if r.niche.is_some() {
        if r.shifted {
            format!(
                "self.{name}.map(|v|((v.get() as i128)-1+({}i128))as {})",
                r.minimum, r.rust_type
            )
        } else {
            format!("self.{name}.map(|v|v.get())")
        }
    } else {
        format!(
            "(self.__qsl_presence&(1<<{})!=0).then_some(self.{name})",
            bit.unwrap()
        )
    }
}
fn encoded(r: &OptionalRepresentation, value: &str) -> String {
    if let Some(niche) = &r.niche {
        if r.shifted {
            format!(
                "{value}.and_then(|v|{niche}::new(((v as i128)-({}i128)+1)as _))",
                r.minimum
            )
        } else {
            format!("{value}.and_then({niche}::new)")
        }
    } else {
        format!("{value}.unwrap_or(0)")
    }
}
pub(super) fn generate(i: &QslInterpreter, s: &QslSchema, out: &mut String) -> QslResult<()> {
    let name = &s.name;
    let wire = format!("__QslFields_{name}");
    let lt = if s.lifetime() { "<'a>" } else { "" };
    let mut raw = s.clone();
    raw.name = wire.clone();
    raw.options.loyal_cache = false;
    raw.options.cache_limit = None;
    raw.options.regions.clear();
    let mut inner = QslInterpreter::new();
    inner.config = i.config;
    inner.config.qsl_rt = false;
    if let Some(plan) = crate::hybrid::HybridPlan::analyze(&raw)? {
        hybrid_codegen::generate(&inner, &raw, &plan, out)?;
    } else {
        inner.generate_impl(&raw, out)?;
    }
    let reps = s
        .fields
        .iter()
        .map(|f| optional_representation(f, s.options.aggressive))
        .collect::<Vec<_>>();
    let mut bits = Vec::new();
    let mut bit = 0usize;
    for r in &reps {
        let b = if r.as_ref().is_some_and(|r| r.niche.is_none()) {
            let n = bit;
            bit += 1;
            Some(n)
        } else {
            None
        };
        bits.push(b);
    }
    let mask_type = if bit <= 8 {
        "u8"
    } else if bit <= 16 {
        "u16"
    } else if bit <= 32 {
        "u32"
    } else {
        "u64"
    };
    ln(out, "#[repr(C)]");
    ln(out, format!("pub struct {name}{lt}{{"));
    for (f, r) in s.fields.iter().zip(&reps) {
        if let Some(r) = r {
            let ty = r
                .niche
                .as_ref()
                .map_or(r.rust_type.clone(), |n| format!("Option<{n}>"));
            ln(out, format!("{}:{ty},", f.name));
        } else {
            let ty = field_rust_type(f, s.borrow_strings())?;
            ln(
                out,
                format!(
                    "pub {}: {},",
                    f.name,
                    if f.optional {
                        format!("Option<{ty}>")
                    } else {
                        ty
                    }
                ),
            );
        }
    }
    if bit != 0 {
        ln(out, format!("__qsl_presence:{mask_type},"));
    }
    ln(out, "}");
    i.emit_schema_hashes(s, out)?;
    ln(
        out,
        format!("impl{lt} {name}{lt}{{fn __qsl_from_fields(fields:{wire}{lt})->Self{{Self{{"),
    );
    for (f, r) in s.fields.iter().zip(&reps) {
        let v = format!("fields.{}", f.name);
        ln(
            out,
            format!(
                "{}:{},",
                f.name,
                r.as_ref().map_or(v.clone(), |r| encoded(r, &v))
            ),
        );
    }
    if bit != 0 {
        let mask = s
            .fields
            .iter()
            .zip(&bits)
            .filter_map(|(f, b)| {
                b.map(|b| format!("((fields.{}.is_some()as {mask_type})<<{b})", f.name))
            })
            .collect::<Vec<_>>()
            .join("|");
        ln(out, format!("__qsl_presence:{mask},"));
    }
    ln(out, "}}");
    ln(out,format!("fn __qsl_with_fields<R>(&self,f:impl FnOnce(&{wire}{lt})->R)->R{{let fields=std::mem::ManuallyDrop::new({wire}{{"));
    for (f, r) in s.fields.iter().zip(&reps) {
        let v = if r.is_some() {
            format!("self.{}()", f.name)
        } else {
            format!("unsafe{{std::ptr::read(&self.{})}}", f.name)
        };
        ln(out, format!("{}:{v},", f.name));
    }
    ln(out, "});f(&fields)}");
    let arguments = s
        .fields
        .iter()
        .map(|f| {
            let ty = field_rust_type(f, s.borrow_strings()).unwrap();
            format!(
                "{}: {}",
                f.name,
                if f.optional {
                    format!("Option<{ty}>")
                } else {
                    ty
                }
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let names = s
        .fields
        .iter()
        .map(|f| f.name.clone())
        .collect::<Vec<_>>()
        .join(",");
    ln(out,format!("#[allow(clippy::too_many_arguments)]pub fn try_new({arguments})->::qsl::QslResult<Self>{{let fields={wire}{{{names}}};fields.validate_values()?;Ok(Self::__qsl_from_fields(fields))}}"));
    for ((f, r), b) in s.fields.iter().zip(&reps).zip(&bits) {
        if let Some(r) = r {
            ln(
                out,
                format!(
                    "pub fn {}(&self)->Option<{}>{{{}}}",
                    f.name,
                    r.rust_type,
                    getter(r, &f.name, *b)
                ),
            );
            ln(out,format!("pub fn set_{}(&mut self,value:Option<{}>)->::qsl::QslResult<()>{{if value.is_some_and(|v|!({}i128..={}i128).contains(&(v as i128))){{return Err(::qsl::QslError::InvalidValue)}}self.{}={};",f.name,r.rust_type,r.minimum,r.maximum,f.name,encoded(r,"value")));
            if let Some(b) = b {
                ln(out,format!("self.__qsl_presence=(self.__qsl_presence&!(1<<{b}))|((value.is_some()as {mask_type})<<{b});"));
            }
            ln(out, "Ok(())}");
        }
    }
    ln(out,format!("pub fn validate(input:&[u8])->::qsl::QslResult<()>{{{wire}::validate(input)}}pub fn validate_values(&self)->::qsl::QslResult<()>{{self.__qsl_with_fields(|f|f.validate_values())}}pub fn serialize_checked_to(&self,out:&mut[u8])->::qsl::QslResult<usize>{{self.__qsl_with_fields(|f|f.serialize_checked_to(out))}}pub fn serialize_checked(&self)->::qsl::QslResult<Vec<u8>>{{self.__qsl_with_fields(|f|f.serialize_checked())}}"));
    if s.flags & crate::FLAG_SAFETY != 0 {
        let borrow = if s.lifetime() { "'a " } else { "" };
        ln(out,format!("pub fn deserialize_checked(input:&{borrow}[u8])->::qsl::QslResult<Self>{{{wire}::deserialize_checked(input).map(Self::__qsl_from_fields)}}pub fn deserialize_checked_into<'out>(input:&{borrow}[u8],out:&'out mut std::mem::MaybeUninit<Self>)->::qsl::QslResult<&'out mut Self>{{let value=Self::deserialize_checked(input)?;Ok(out.write(value))}}"));
    }
    if crate::hybrid::HybridPlan::analyze(s)?.is_some() {
        for key in [
            "QPS_SCALAR_PREFIX_BYTES",
            "QPS_RAW_PREFIX_BYTES",
            "QPS_PREFIX_LENGTH_HEADER",
        ] {
            ln(out, format!("pub const {key}:usize={wire}::{key};"));
        }
    }
    let quant = crate::payload_quant::PayloadQuantPlan::analyze(s)?;
    if quant.active {
        for key in [
            "QUANT_ACTIVE",
            "QUANT_WHOLE_WORD_READ",
            "QUANT_WHOLE_WORD_WRITE",
        ] {
            ln(out, format!("pub const {key}:bool={wire}::{key};"));
        }
        for key in ["QUANTUM", "QUANT_PADDING_BYTES", "QUANT_MAX_BYTES"] {
            ln(out, format!("pub const {key}:usize={wire}::{key};"));
        }
        ln(
            out,
            "pub const QUANT_RESTORE_BYTES:usize=std::mem::size_of::<Self>();",
        );
        ln(out,"pub fn serialize_quantized_to<'out>(&self,out:&'out mut[u8])->::qsl::QslResult<::qsl::QuantizedFrame<'out,Self>>{self.validate_values()?;let len=<Self as ::qsl::Serializable>::serialized_size(self);::qsl::validation::check_size(len,Self::MAX_DATA_SIZE)?;let storage=out.get_mut(..len+Self::QUANT_PADDING_BYTES).ok_or(::qsl::QslError::BufferTooSmall)?;unsafe{<Self as ::qsl::payload_quant::QuantizedRecord>::quant_write_prepared(self,storage.as_mut_ptr());Ok(::qsl::QuantizedFrame::from_validated(storage,len))}}\n/// # Safety\n/// Input is a complete schema-valid frame. Destination storage is aligned, exclusive,\n/// disjoint from input, and follows QuantizedDestination initialization requirements.\npub unsafe fn deserialize_into<D: ::qsl::payload_quant::QuantizedDestination<Self>>(bytes:*const u8,len:usize,out:D){out.decode_padded(bytes,len)}");
        if i.config.qsl_rt {
            let cap = quant.max_bytes + quant.padding_bytes;
            let slots = s.options.slots.unwrap_or(i.config.qsl_rt_default);
            ln(out,format!("pub fn serialize(&self)->::qsl::QslResult<::qsl::QuantizedLease<Self,{cap},{slots}>>{{{name}_QUANT_REGIONS.encode(self)}}"));
        } else {
            ln(out,"pub fn serialize(&self)->::qsl::QslResult<::qsl::QuantizedOut<Self>>{::qsl::QuantizedOut::encode(self)}");
        }
    } else if i.config.qsl_rt {
        qps_codegen::region_method(s, out, i.config.qsl_rt_default)?;
    }
    ln(out, "}");
    let target: String = if s.lifetime() {
        "::qsl::BorrowSerializable<'a>".into()
    } else {
        "::qsl::Serializable".into()
    };
    let unsafe_read = if s.lifetime() {
        "deserialize_borrowed_unsafe"
    } else {
        "deserialize_unsafe"
    };
    let raw_read = if s.lifetime() {
        "deserialize_borrowed_from_raw_parts"
    } else {
        "deserialize_from_raw_parts"
    };
    ln(out,format!("impl{lt} {target} for {name}{lt}{{const OPTIONAL_FIELDS_COUNT:usize={};const SIZE_TAG:usize=<{wire}{lt} as {target}>::SIZE_TAG;fn get_size()->usize{{<{wire}{lt} as {target}>::get_size()}}fn serialized_size(&self)->usize{{self.__qsl_with_fields(|f|<{wire}{lt} as {target}>::serialized_size(f))}}unsafe fn serialize_unsafe(&self,to:*mut u8){{self.__qsl_with_fields(|f|<{wire}{lt} as {target}>::serialize_unsafe(f,to))}}unsafe fn serialize_into_unsafe(&self,to:*mut u8)->usize{{self.__qsl_with_fields(|f|<{wire}{lt} as {target}>::serialize_into_unsafe(f,to))}}unsafe fn {unsafe_read}(bytes:*const u8)->Self{{Self::__qsl_from_fields(<{wire}{lt} as {target}>::{unsafe_read}(bytes))}}unsafe fn {raw_read}(bytes:*const u8,len:usize)->Self{{Self::__qsl_from_fields(<{wire}{lt} as {target}>::{raw_read}(bytes,len))}}}}",s.optional_count()));
    if quant.active {
        ln(out,format!("unsafe impl ::qsl::payload_quant::QuantizedRecord for {name}{{const MAX_DATA_SIZE:usize=Self::MAX_DATA_SIZE;const QUANTUM:usize=Self::QUANTUM;const PADDING_BYTES:usize=Self::QUANT_PADDING_BYTES;const MAX_QUANTIZED_BYTES:usize=Self::QUANT_MAX_BYTES;fn quant_validate_values(&self)->::qsl::QslResult<()>{{self.validate_values()}}unsafe fn quant_write_prepared(&self,out:*mut u8)->usize{{self.__qsl_with_fields(|f|<{wire} as ::qsl::payload_quant::QuantizedRecord>::quant_write_prepared(f,out))}}unsafe fn quant_read_padded(bytes:*const u8,len:usize,out:*mut Self){{let mut fields=std::mem::MaybeUninit::<{wire}>::uninit();<{wire} as ::qsl::payload_quant::QuantizedRecord>::quant_read_padded(bytes,len,fields.as_mut_ptr());out.write(Self::__qsl_from_fields(fields.assume_init()));}}}}"));
        if s.flags & crate::FLAG_SAFETY != 0 {
            ln(out,format!("impl ::qsl::payload_quant::CheckedQuantizedRecord for {name}{{fn quant_validate(input:&[u8])->::qsl::QslResult<()>{{Self::validate(input)}}}}"));
        }
        if i.config.qsl_rt {
            let cap = quant.max_bytes + quant.padding_bytes;
            let slots = s.options.slots.unwrap_or(i.config.qsl_rt_default);
            ln(out,format!("#[allow(non_upper_case_globals)]static {name}_QUANT_REGIONS: ::qsl::QuantizedRegions<{name},{cap},{slots}>=::qsl::QuantizedRegions::new();"));
        }
    } else if i.config.qsl_rt {
        qps_codegen::region_static(s, out, i.config.qsl_rt_default)?;
    }
    if s.fields.iter().any(|f| f.default.is_some()) {
        ln(out,format!("impl{lt} Default for {name}{lt}{{fn default()->Self{{Self::__qsl_from_fields({wire}::default())}}}}"));
    }
    Ok(())
}
