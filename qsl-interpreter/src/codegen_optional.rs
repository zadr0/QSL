//! Ordinary tagged/BM optional dispatch. No length-prediction order is applied to BM.
use super::*;
pub(super) fn applies(s: &QslSchema) -> bool {
    !s.sequential() && (s.bitmap() || s.optional_count() != 0)
}
fn cursor(s: &QslSchema, write: bool) -> QslResult<&'static str> {
    fn field_bits(data: &QslType, width: usize) -> QslResult<usize> {
        let child = match data {
            QslType::Array(e, _)
            | QslType::FixedArray(e, _)
            | QslType::RangedArray { element: e, .. } => field_bits(e, e.default_compact_bits()?)?,
            _ => 0,
        };
        Ok(width.max(child))
    }
    let max = s
        .fields
        .iter()
        .map(|f| field_bits(&f.data, f.data.rank_width(f.bits)?))
        .collect::<QslResult<Vec<_>>>()?
        .into_iter()
        .max()
        .unwrap_or(8)
        .max(8);
    let header = if !s.bitmap() && s.optional_count() > 1 {
        s.tag_bits() + 7
    } else {
        8
    };
    let bits = if s.compact() {
        max + 7
    } else {
        max.max(header)
    };
    let reg = crate::BitRegister::for_bits(bits)?;
    Ok(if write { reg.writer() } else { reg.reader() })
}
pub(super) fn ordered_fields(s: &QslSchema) -> Vec<&QslField> {
    if s.bitmap() {
        s.fields
            .iter()
            .filter(|f| !f.optional)
            .chain(s.fields.iter().rev().filter(|f| f.optional))
            .collect()
    } else {
        s.fields.iter().collect()
    }
}
pub(super) fn tables(s: &QslSchema, out: &mut String) -> QslResult<()> {
    if !s.opt_table() || s.optional_count() == 0 {
        return Ok(());
    }
    let name = &s.name;
    let reader = cursor(s, false)?;
    let ty = if s.lifetime() {
        format!("{name}<'a>")
    } else {
        name.clone()
    };
    let hrtb = if s.lifetime() { "for<'a> " } else { "" };
    if s.lifetime() {
        for f in s.fields.iter().filter(|f| f.optional) {
            ln(out,format!("#[allow(non_snake_case)]unsafe fn {name}_bm_{}<'a>(reader:&mut {reader},out:*mut {name}<'a>){{{name}::__qsl_cursor_{}(reader,out)}}",f.name,f.name));
        }
    }
    let entries = s
        .fields
        .iter()
        .filter(|f| f.optional)
        .map(|f| {
            if s.lifetime() {
                format!("{name}_bm_{}", f.name)
            } else {
                format!("{name}::__qsl_cursor_{}", f.name)
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    ln(out,format!("#[allow(non_upper_case_globals,clippy::type_complexity)]static {name}_OPT_TABLE:[{hrtb}unsafe fn(&mut {reader},*mut {ty});{}]=[{entries}];",s.optional_count()));
    Ok(())
}
pub(super) fn callbacks(s: &QslSchema, out: &mut String) -> QslResult<()> {
    let reader = cursor(s, false)?;
    for f in s.fields.iter().filter(|f| f.optional) {
        ln(
            out,
            format!(
                "unsafe fn __qsl_cursor_{}(reader:&mut {reader},out:*mut Self){{",
                f.name
            ),
        );
        emit_stored_value(
            f,
            out,
            "value",
            f.data.rank_width(f.bits)?,
            s.compact(),
            s.borrow_strings(),
        )?;
        if s.compact() && s.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "reader.align();");
        }
        ln(out, format!("(*out).{}=Some(value);}}", f.name));
    }
    Ok(())
}
fn bitmap_header(s: &QslSchema, out: &mut String, checked: bool) {
    let width = s.bitmap_width();
    let ty = if width == 0 {
        "u8".into()
    } else {
        format!("u{width}")
    };
    let bytes = width / 8;
    if width == 0 {
        return;
    } else if checked {
        if width <= 64 {
            ln(out, format!("let bit_mask=c.uint({bytes})? as {ty};"));
        } else {
            ln(
                out,
                "let bit_mask=u128::from_le_bytes(c.bytes(16)?.try_into().unwrap());",
            );
        }
    } else {
        ln(out,format!("let bit_mask={ty}::from_le(std::ptr::read_unaligned(reader.read_exact({bytes}).cast::<{ty}>()));"));
    }
    if checked && width != 0 && s.optional_count() < width {
        ln(out,format!("if bit_mask!=0 && {width}-bit_mask.trailing_zeros()as usize>{}{{return Err(::qsl::QslError::InvalidValue)}}",s.optional_count()));
    }
}
fn tag_header(s: &QslSchema, out: &mut String, checked: bool) {
    let n = s.optional_count();
    let r = if checked { "c" } else { "reader" };
    let tail = if checked { "?" } else { "" };
    let method = if checked { "bits" } else { "read_bits" };
    let count_bits = if n == 1 { 1 } else { s.count_bits() };
    ln(
        out,
        format!(
            "let count={r}.{method}({count_bits}){tail} as usize;let mut optional_tags=[0u8;{n}];"
        ),
    );
    if checked {
        ln(
            out,
            format!("if count>{n}{{return Err(::qsl::QslError::InvalidValue)}}"),
        );
    }
    if n == 1 {
        ln(out, "if count!=0{optional_tags[0]=1;}");
    } else if n > 1 {
        if checked {
            ln(out, "let mut previous=0usize;");
        }
        ln(out, "for slot in optional_tags.iter_mut().take(count){");
        ln(
            out,
            format!("let tag={r}.{method}({}){tail} as usize;", s.tag_bits()),
        );
        let base = s.optional_tag_base();
        let fields = s.fields.iter().filter(|f| f.optional).collect::<Vec<_>>();
        let dense = fields
            .iter()
            .enumerate()
            .all(|(i, f)| f.tag - 1 == base + i);
        if checked {
            ln(
                out,
                "if tag<previous{return Err(::qsl::QslError::InvalidValue)}previous=tag+1;",
            );
        }
        if dense {
            if checked {
                ln(
                    out,
                    format!(
                        "if !({base}..{}).contains(&tag){{return Err(::qsl::QslError::InvalidValue)}}",
                        base + n
                    ),
                );
            }
            ln(out, format!("*slot=(tag-{base}+1)as u8;}}"));
        } else {
            ln(out, "*slot=match tag{");
            for (i, f) in fields.iter().enumerate() {
                ln(out, format!("{}=>{},", f.tag - 1, i + 1));
            }
            ln(
                out,
                if checked {
                    "_=>return Err(::qsl::QslError::InvalidValue),};}"
                } else {
                    "_=>std::hint::unreachable_unchecked(),};}"
                },
            );
        }
    }
    if !s.compact() {
        ln(out, format!("{r}.align(){tail};"));
    }
}
fn raw_dispatch(s: &QslSchema, out: &mut String) {
    if s.opt_table() {
        ln(
            out,
            format!(
                "({}_OPT_TABLE.get_unchecked(index-1))(&mut reader,&mut result);",
                s.name
            ),
        );
    } else {
        ln(out, "match index{");
        for (i, f) in s.fields.iter().filter(|f| f.optional).enumerate() {
            ln(
                out,
                format!(
                    "{}=>Self::__qsl_cursor_{}(&mut reader,&mut result),",
                    i + 1,
                    f.name
                ),
            );
        }
        ln(out, "_=>std::hint::unreachable_unchecked(),}");
    }
}
pub(super) fn read(s: &QslSchema, out: &mut String) -> QslResult<()> {
    ln(
        out,
        format!(
            "unsafe fn __qsl_read_unchecked(bytes:*const u8)->Self{{let mut reader={}::new(bytes);",
            cursor(s, false)?
        ),
    );
    if s.bitmap() {
        bitmap_header(s, out, false);
    } else {
        tag_header(s, out, false);
    }
    for (i, f) in s.fields.iter().enumerate().filter(|(_, f)| !f.optional) {
        emit_stored_value(
            f,
            out,
            &format!("field_{i}"),
            f.data.rank_width(f.bits)?,
            s.compact(),
            s.borrow_strings(),
        )?;
        if s.compact() && s.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "reader.align();");
        }
    }
    tagged_optional_codegen::result(s, out);
    if s.optional_count() == 0 {
        ln(out, "result}");
        return Ok(());
    }
    if s.bitmap() {
        ln(out,format!("let mut bit_mask=bit_mask;while bit_mask!=0{{let index={}-bit_mask.trailing_zeros()as usize;bit_mask&=bit_mask-1;",s.bitmap_width()));
    } else {
        ln(
            out,
            "for marker in optional_tags.into_iter().take(count){let index=marker as usize;",
        );
    }
    raw_dispatch(s, out);
    ln(out, "}result}");
    Ok(())
}
pub(super) fn write(s: &QslSchema, out: &mut String) -> QslResult<()> {
    ln(
        out,
        format!("let mut writer={}::from_raw(to);", cursor(s, true)?),
    );
    emit_fast_header(s, out);
    for f in ordered_fields(s) {
        let value = if f.optional {
            ln(
                out,
                format!("if let Some(value)=self.{}.as_ref(){{", f.name),
            );
            "value".into()
        } else {
            format!("&self.{}", f.name)
        };
        fast_stored_write(f, out, &value, f.data.rank_width(f.bits)?, s.compact())?;
        if s.compact() && s.flags & FLAG_ALIGNMENT != 0 {
            ln(out, "writer.flush_unchecked();");
        }
        if f.optional {
            ln(out, "}");
        }
    }
    ln(out, "writer.flush_unchecked();");
    if needs_written_length(s) {
        ln(out, "writer.as_mut_ptr().offset_from(to)as usize");
    }
    Ok(())
}
fn checked_dispatch(s: &QslSchema, out: &mut String, capture: bool) -> QslResult<()> {
    if capture && s.opt_table() {
        ln(out,"(unsafe{Self::__QSL_CHECKED_OPT_TABLE.get_unchecked(index-1).unwrap_unchecked()})(&mut c,&mut result,0)?;");
        return Ok(());
    }
    ln(out, "match index{");
    for (i, f) in s.fields.iter().filter(|f| f.optional).enumerate() {
        ln(out, format!("{}=>{{", i + 1));
        if capture {
            ln(
                out,
                format!("Self::__qsl_checked_opt_{}(&mut c,&mut result,0)?;", f.name),
            );
        } else {
            checked_codegen::field(
                f,
                out,
                f.data.rank_width(f.bits)?,
                s.compact(),
                s.borrow_strings(),
                false,
                None,
            )?;
            if s.compact() && s.flags & FLAG_ALIGNMENT != 0 {
                ln(out, "c.align()?;");
            }
        }
        ln(out, "},");
    }
    ln(out, "_=>return Err(::qsl::QslError::InvalidValue),}");
    Ok(())
}
pub(super) fn checked(s: &QslSchema, out: &mut String, capture: bool) -> QslResult<()> {
    if s.bitmap() {
        bitmap_header(s, out, true);
    } else {
        tag_header(s, out, true);
    }
    for f in s.fields.iter().filter(|f| !f.optional) {
        checked_codegen::field(
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
    }
    if capture {
        checked_codegen::begin_result(s, out);
    }
    if s.optional_count() == 0 {
        ln(
            out,
            if capture {
                "c.finish()?;Ok(result)}"
            } else {
                "c.finish()}"
            },
        );
        return Ok(());
    }
    if s.bitmap() {
        ln(out,format!("let mut bit_mask=bit_mask;while bit_mask!=0{{let index={}-bit_mask.trailing_zeros()as usize;bit_mask&=bit_mask-1;",s.bitmap_width()));
    } else {
        ln(
            out,
            "for marker in optional_tags.into_iter().take(count){let index=marker as usize;",
        );
    }
    checked_dispatch(s, out, capture)?;
    ln(out, "}");
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
