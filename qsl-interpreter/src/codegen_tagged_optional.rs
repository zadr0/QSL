//! Optional tag dispatch. Wire bytes are identical to the ordinary codec.
use super::*;

/// Start with initialized optional slots: assignment then drops correctly and
/// partially constructed owned fields remain unwind-safe.
pub(super) fn result(schema: &QslSchema, out: &mut String) {
    ln(out, "#[allow(unused_mut)] let mut result=Self{");
    for (i, f) in schema.fields.iter().enumerate() {
        ln(
            out,
            format!(
                "{}:{},",
                f.name,
                if f.optional {
                    "None".into()
                } else {
                    format!("field_{i}")
                }
            ),
        );
    }
    ln(out, "};");
}

/// Both raw-tag and predicted layouts use this same tag-indexed reader table.
pub(super) fn dispatch(schema: &QslSchema, out: &mut String, tag: &str, length: &str) {
    ln(out,format!("let callback=(*{}_OPT_TABLE.get_unchecked({tag})).unwrap_unchecked();let used=callback(reader.as_ptr(),&mut result,{length});reader.read_exact(used);",schema.name));
}

pub(super) fn read(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    ln(
        out,
        "#[inline] unsafe fn __qsl_read_unchecked(bytes:*const u8)->Self{",
    );
    let n = schema.optional_count();
    if n > 0 {
        ln(out, "let mut header=::qsl::QslFastBitsReader::new(bytes);");
        if n == 1 {
            ln(out, "let count=header.read_bits(1) as usize;");
        } else if n * schema.tag_bits() <= 64 {
            ln(out,format!("let count=header.read_bits({}) as usize;let mut tags=header.read_bits((count*{})as u32);",schema.count_bits(),schema.tag_bits()));
        } else {
            ln(out,format!("let count=header.read_bits({})as usize;let mut tag_mask=0u64;for _ in 0..count{{tag_mask|=1u64<<header.read_bits({});}}",schema.count_bits(),schema.tag_bits()));
        }
        ln(
            out,
            "header.align();let mut reader=::qsl::QslByteReader::new(header.as_ptr());",
        );
    } else {
        ln(out, "let mut reader=::qsl::QslByteReader::new(bytes);");
    }
    for (i, f) in schema
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| !f.optional)
    {
        emit_stored_value(
            f,
            out,
            &format!("field_{i}"),
            f.bits,
            false,
            schema.borrow_strings(),
        )?;
    }
    result(schema, out);
    if n == 1 {
        ln(out, "if count!=0{");
        dispatch(
            schema,
            out,
            &(schema.fields.iter().find(|f| f.optional).unwrap().tag - 1).to_string(),
            "0",
        );
        ln(out, "}");
    } else if n * schema.tag_bits() > 64 {
        ln(
            out,
            "while tag_mask!=0{let tag=tag_mask.trailing_zeros()as usize;tag_mask&=tag_mask-1;",
        );
        dispatch(schema, out, "tag", "0");
        ln(out, "}");
    } else if n > 1 {
        let width = schema.tag_bits();
        let mask = (1u64 << width) - 1;
        ln(
            out,
            format!("for _ in 0..count{{let tag=(tags&{mask})as usize;tags>>={width};"),
        );
        dispatch(schema, out, "tag", "0");
        ln(out, "}");
    }
    ln(out, "result}");
    Ok(())
}
