//! Private optional slots: only a presence bit authorizes read/drop of a slot.
//! Native mask bits follow logical tag-1, independently of the wire codec.
use super::*;
use qsl_lexer::layout::field_rust_type;
fn copy_value(f: &crate::QslField) -> bool {
    matches!(
        f.storage,
        qsl_lexer::layout::StorageKind::Default | qsl_lexer::layout::StorageKind::FixedArray(_)
    ) && (matches!(
        f.data,
        QslType::Byte
            | QslType::Integer { .. }
            | QslType::RangedInteger { .. }
            | QslType::DomainInteger(_)
            | QslType::Flags(_)
            | QslType::FixedString(_)
    ))
}
pub(super) fn generate(i: &QslInterpreter, s: &QslSchema, out: &mut String) -> QslResult<()> {
    let direct = representation_codegen::direct_encoding(s);
    let name = &s.name;
    let wire = format!("__QslFields_{name}");
    let lt = if s.lifetime() { "<'a>" } else { "" };
    let max_tag = s
        .fields
        .iter()
        .filter(|f| f.optional)
        .map(|f| f.tag)
        .max()
        .unwrap_or(0);
    if max_tag == 0 || max_tag > 128 {
        return Err(QslError::InvalidSchema);
    }
    let mask = match max_tag {
        0..=8 => "u8",
        9..=16 => "u16",
        17..=32 => "u32",
        33..=64 => "u64",
        _ => "u128",
    };
    let mut raw = s.clone();
    raw.name = wire.clone();
    raw.options.aggressive_bitmap = false;
    raw.options.auto_layout = false;
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
    ln(out, format!("#[repr(C)] pub struct {name}{lt}{{"));
    for f in &s.fields {
        let ty = field_rust_type(f, s.borrow_strings())?;
        ln(
            out,
            if f.optional {
                format!("{}:std::mem::MaybeUninit<{ty}>,", f.name)
            } else {
                format!("pub {}: {ty},", f.name)
            },
        );
    }
    ln(out, format!("__qsl_presence:{mask},}}"));
    i.emit_schema_hashes(s, out)?;
    ln(
        out,
        format!(
            "impl{lt} {name}{lt}{{fn __qsl_from_fields(fields:{wire}{lt})->Self{{let mask:{mask}=0"
        ),
    );
    for f in s.fields.iter().filter(|f| f.optional) {
        ln(
            out,
            format!("|((fields.{}.is_some()as {mask})<<{})", f.name, f.tag - 1),
        );
    }
    ln(out, ";Self{");
    for f in &s.fields {
        let n = &f.name;
        ln(
            out,
            if f.optional {
                format!("{n}:match fields.{n}{{Some(v)=>std::mem::MaybeUninit::new(v),None=>std::mem::MaybeUninit::uninit()}},")
            } else {
                format!("{n}:fields.{n},")
            },
        );
    }
    ln(out, "__qsl_presence:mask}}");
    if !direct {
        ln(
            out,
            format!(
                "fn __qsl_with_fields<R>(&self,f:impl FnOnce(&{wire}{lt})->R)->R{{
// Aliased owners are never moved or dropped; only private read-only callbacks use this view.
let fields=std::mem::ManuallyDrop::new({wire}{{"
            ),
        );
        for f in &s.fields {
            let n = &f.name;
            ln(
                out,
                if f.optional {
                    format!("{n}:if self.__qsl_presence&(1<<{})!=0{{Some(unsafe{{std::ptr::read(self.{n}.as_ptr())}})}}else{{None}},",f.tag-1)
                } else {
                    format!("{n}:unsafe{{std::ptr::read(&self.{n})}},")
                },
            );
        }
        ln(out, "});f(&fields)}");
    }
    let args = s
        .declaration_fields()
        .into_iter()
        .map(|f| {
            let ty = field_rust_type(f, s.borrow_strings())?;
            Ok(format!(
                "{}: {}",
                f.name,
                if f.optional {
                    format!("Option<{ty}>")
                } else {
                    ty
                }
            ))
        })
        .collect::<QslResult<Vec<_>>>()?
        .join(",");
    let names = s
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .collect::<Vec<_>>()
        .join(",");
    ln(out,format!("#[allow(clippy::too_many_arguments)]pub fn try_new({args})->::qsl::QslResult<Self>{{let fields={wire}{{{names}}};fields.validate_values()?;Ok(Self::__qsl_from_fields(fields))}}"));
    for f in s.fields.iter().filter(|f| f.optional) {
        let n = &f.name;
        let ty = field_rust_type(f, s.borrow_strings())?;
        let bit = f.tag - 1;
        let (ret, read) = if copy_value(f) {
            (ty.clone(), format!("unsafe{{*self.{n}.assume_init_ref()}}"))
        } else {
            (
                format!("&{ty}"),
                format!("unsafe{{self.{n}.assume_init_ref()}}"),
            )
        };
        ln(out,format!("pub fn get_{n}(&self)->Option<{ret}>{{if self.__qsl_presence&(1<<{bit})==0{{None}}else{{Some({read})}}}}"));
        ln(out,format!("#[allow(unused_variables)]pub fn set_{n}(&mut self,value: {ty})->::qsl::QslResult<()>{{let mut budget=::qsl::validation::ValueBudget::new(Self::MAX_DATA_SIZE);{{let value=&value;"));
        validation_codegen::value_field(f, out)?;
        ln(out, "}");
        ln(out,format!("// Clear presence before running a destructor; an unwinding drop leaves an absent slot.
self.clear_{n}();self.{n}.write(value);self.__qsl_presence|=1<<{bit};Ok(())}}"));
        ln(out,format!("pub fn clear_{n}(&mut self){{if self.__qsl_presence&(1<<{bit})!=0{{self.__qsl_presence&=!(1<<{bit});unsafe{{self.{n}.assume_init_drop();}}}}}}"));
    }
    if direct {
        representation_codegen::direct_encoder(s, out, true)?;
    }
    representation_codegen::delegates(i, s, out, &wire, lt, direct)?;
    ln(
        out,
        format!("impl{lt} Drop for {name}{lt}{{fn drop(&mut self){{"),
    );
    for f in s.fields.iter().filter(|f| f.optional) {
        ln(out, format!("self.clear_{}();", f.name));
    }
    ln(out, "}}");
    Ok(())
}
