//! Checked scalar islands: reserve the complete validated optional span once.
//! Values remain ordinary Rust scalars until a single aggregate domain check passes.
//! No user-defined types, references or NonZero values are created by these kernels.
use super::*;
pub(super) fn applies(s: &QslSchema) -> bool {
    s.options.aggressive_group
        && !s.compact()
        && !s.sequential()
        && !s.qps()
        && s.options.dynamic.is_none()
        && s.optional_count() >= 2
        && s.fields.iter().filter(|f| f.optional).all(|f| {
            matches!(
                f.data,
                QslType::Byte
                    | QslType::Integer { .. }
                    | QslType::RangedInteger { .. }
                    | QslType::Flags(_)
            ) && f
                .data
                .fixed_wire_size()
                .ok()
                .flatten()
                .is_some_and(|n| n <= 8)
        })
}
fn optional(s: &QslSchema) -> Vec<&QslField> {
    s.fields.iter().filter(|f| f.optional).collect()
}
pub(super) fn methods(s: &QslSchema, out: &mut String) -> QslResult<()> {
    if !applies(s) {
        return Ok(());
    }
    let fields = optional(s);
    let sizes = fields
        .iter()
        .map(|f| bytes_of(&f.data))
        .collect::<QslResult<Vec<_>>>()?;
    ln(
        out,
        format!(
            "#[allow(dead_code)]const __QSL_GROUP_SIZES:[usize;{}]={sizes:?};",
            fields.len()
        ),
    );
    for f in &fields {
        let ty = f.data.primitive()?.ok_or(QslError::InvalidSchema)?;
        let r =
            qsl_lexer::layout::optional_representation(f, false).ok_or(QslError::InvalidSchema)?;
        for capture in [false, true] {
            let mode = if capture { "store" } else { "valid" };
            let output = if capture { ",output:&mut Self" } else { "" };
            ln(out,format!("/// # Safety
/// bytes addresses {} initialized input bytes, disjoint from the initialized output.
/// Every bit pattern is valid for the primitive Rust scalar; logical bounds are returned separately.
#[allow(dead_code,clippy::manual_range_contains)]
unsafe fn __qsl_group_{mode}_{}(bytes:*const u8{output})->bool{{let value=<{ty}>::from_le(std::ptr::read_unaligned(bytes.cast::<{ty}>()));",sizes[fields.iter().position(|x|x.tag==f.tag).unwrap()],f.tag));
            if capture {
                ln(out, format!("output.{}=Some(value);", f.name));
            }
            ln(
                out,
                format!(
                    "let value=value as i128;({}i128<=value)&(value<={}i128)}}",
                    r.minimum, r.maximum
                ),
            );
        }
    }
    if s.opt_table() {
        for capture in [false, true] {
            let mode = if capture { "store" } else { "valid" };
            let output = if capture { ",&mut Self" } else { "" };
            let entries = fields
                .iter()
                .map(|f| format!("Self::__qsl_group_{mode}_{}", f.tag))
                .collect::<Vec<_>>()
                .join(",");
            ln(
                out,
                format!(
                    "#[allow(dead_code)]const __QSL_GROUP_{}:[unsafe fn(*const u8{output})->bool;{}]=[{entries}];",
                    mode.to_uppercase(),
                    fields.len()
                ),
            );
        }
    }
    Ok(())
}
fn begin_indices(s: &QslSchema, out: &mut String) {
    if s.bitmap() {
        ln(out,format!("let mut pending=bit_mask;while pending!=0{{let index={}-pending.trailing_zeros()as usize;pending&=pending-1;",s.bitmap_width()));
    } else {
        ln(
            out,
            "for marker in optional_tags.iter().take(count){let index=*marker as usize;",
        );
    }
}
pub(super) fn read(s: &QslSchema, out: &mut String, capture: bool) -> QslResult<()> {
    let fields = optional(s);
    let sizes = fields
        .iter()
        .map(|f| bytes_of(&f.data))
        .collect::<QslResult<Vec<_>>>()?;
    if sizes.iter().all(|&n| n == sizes[0]) {
        let count = if s.bitmap() {
            format!("(bit_mask as u{}).count_ones()as usize", s.bitmap_width())
        } else {
            "count".into()
        };
        ln(out, format!("let span=({count})*{};", sizes[0]));
    } else {
        ln(out, "let mut span=0usize;");
        begin_indices(s, out);
        ln(out,"// Header validation proves 1<=index<=optional_count; <=128 scalars of <=8 B cannot overflow usize.
span+=unsafe{*Self::__QSL_GROUP_SIZES.get_unchecked(index-1)};}");
    }
    ln(
        out,
        "c.charge(span)?;let group=c.bytes(span)?;let mut offset=0usize;let mut valid=true;",
    );
    begin_indices(s, out);
    let mode = if capture { "store" } else { "valid" };
    let output = if capture { ",&mut result" } else { "" };
    if s.opt_table() {
        ln(out,format!("// The complete scalar island is readable. Table widths and pointer increments share the same validated index.
valid&=unsafe{{(Self::__QSL_GROUP_{}.get_unchecked(index-1))(group.as_ptr().add(offset){output})}};",mode.to_uppercase()));
    } else {
        ln(out, "match index{");
        for (i, f) in fields.iter().enumerate() {
            ln(out,format!("{}=>{{valid&=unsafe{{Self::__qsl_group_{mode}_{}(group.as_ptr().add(offset){output})}};}},",i+1,f.tag));
        }
        ln(out, "_=>unsafe{std::hint::unreachable_unchecked()},}");
    }
    if sizes.iter().all(|&n| n == sizes[0]) {
        ln(out, format!("offset+={};", sizes[0]));
    } else {
        ln(
            out,
            "offset+=unsafe{*Self::__QSL_GROUP_SIZES.get_unchecked(index-1)};",
        );
    }
    ln(
        out,
        "}let _=offset;if !valid{return Err(::qsl::QslError::InvalidValue);}",
    );
    ln(
        out,
        if capture {
            "c.finish()?;Ok(result)}"
        } else {
            "c.finish()}"
        },
    );
    Ok(())
}
