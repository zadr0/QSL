use super::*;
use crate::static_optional::{StaticOptionalPlan, StaticPresence};

pub(super) fn tables(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let p = StaticOptionalPlan::analyze(schema)?;
    if let StaticPresence::Graph(ref g) = p.presence {
        return length_graph_codegen::tables(schema, g, out);
    }
    let name = &schema.name;
    let points: Vec<(usize, u64)> = match &p.presence {
        StaticPresence::Eof => vec![(0, 0), (1, 1)],
        StaticPresence::Length(plan) => plan
            .rows
            .iter()
            .map(|r| (r.len as usize, r.mask as u64))
            .collect(),
        StaticPresence::Mask { .. } => Vec::new(),
        StaticPresence::Graph(_) => unreachable!(),
    };
    ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_CRITICAL_POINTS:[(usize,u64);{}]={points:?};",points.len()));
    let mut recipes = Vec::new();
    for (i, f) in schema.fields.iter().filter(|f| f.optional).enumerate() {
        let prefix = if i == 0 { 0 } else { (1u64 << i) - 1 };
        recipes.push((prefix, f.data.fixed_wire_size()?));
    }
    ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_OFFSET_TABLE:[(u64,Option<usize>);{}]={recipes:?};",recipes.len()));
    if let StaticPresence::Length(plan) = &p.presence {
        ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_LENGTH_NODES:[u32;{}]={:?};",plan.index.len(),plan.index));
    }
    qps_codegen::callback_table(schema, out)
}
pub(super) fn methods(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let p = StaticOptionalPlan::analyze(schema)?;
    if let StaticPresence::Graph(ref g) = p.presence {
        length_graph_codegen::methods(schema, g, out);
        return Ok(());
    }
    ln(
        out,
        format!(
            "pub const OPT_TABLE_HEADER_BYTES:usize={};",
            p.header_bytes()
        ),
    );
    ln(out,"/// Boolean field-state function synthesized from critical lengths.\n/// # Safety\n/// optional_len and explicit_mask describe a schema-valid frame.\n#[inline(always)] unsafe fn __qsl_optional_mask(optional_len:usize,explicit_mask:u64)->u64 {let _=explicit_mask;");
    match &p.presence {
        StaticPresence::Eof => ln(out, "(optional_len!=0) as u64"),
        StaticPresence::Length(plan) => {
            let powers = plan.fields.iter().all(|f| f.min.is_power_of_two());
            if powers {
                let expression = plan
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        format!(
                            "(((optional_len>>{})&1) as u64)<<{i}",
                            f.min.trailing_zeros()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                ln(out, expression);
            } else {
                ln(out,format!("let node=*{}_LENGTH_NODES.get_unchecked(optional_len) as usize;{}_CRITICAL_POINTS.get_unchecked(node).1",schema.name,schema.name));
            }
        }
        StaticPresence::Mask { .. } => ln(out, "let _=optional_len;explicit_mask"),
        StaticPresence::Graph(_) => unreachable!(),
    }
    ln(out, "}");
    Ok(())
}
pub(super) fn write(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let p = StaticOptionalPlan::analyze(schema)?;
    if let StaticPresence::Graph(ref g) = p.presence {
        return length_graph_codegen::write(schema, g, out);
    }
    ln(out, "let mut writer=::qsl::QslByteWriter::from_raw(to);");
    if let StaticPresence::Mask { bytes } = &p.presence {
        let expression = schema
            .fields
            .iter()
            .filter(|f| f.optional)
            .enumerate()
            .map(|(i, f)| format!("((self.{}.is_some() as u64)<<{i})", f.name))
            .collect::<Vec<_>>()
            .join("|");
        if *bytes != 0 {
            ln(out,format!("let presence=0u64|{expression};writer.write_bytes_unchecked(&presence.to_le_bytes()[..{bytes}]);"));
        }
    }
    for f in &schema.fields {
        if f.optional && optional_scalar_write(f, out)? {
            continue;
        }
        if f.optional {
            ln(
                out,
                format!("if let Some(value)=self.{}.as_ref(){{", f.name),
            );
        }
        let expr = if f.optional {
            "value".into()
        } else {
            format!("&self.{}", f.name)
        };
        fast_type_write(&f.data, out, &expr, f.bits, false)?;
        if f.optional {
            ln(out, "}");
        }
    }
    ln(out, "writer.as_mut_ptr().offset_from(to) as usize");
    Ok(())
}
pub(super) fn read(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let p = StaticOptionalPlan::analyze(schema)?;
    if let StaticPresence::Graph(ref g) = p.presence {
        return length_graph_codegen::read(schema, g, out);
    }
    ln(out,"#[inline(always)] unsafe fn __qsl_read_sized(bytes:*const u8,len:usize)->Self{let mut reader=::qsl::QslByteReader::new(bytes);");
    if let StaticPresence::Mask { bytes } = &p.presence {
        ln(out,format!("let mut header=[0u8;8];std::ptr::copy_nonoverlapping(reader.read_exact({bytes}),header.as_mut_ptr(),{bytes});let explicit_mask=u64::from_le_bytes(header);"));
    } else {
        ln(out, "let explicit_mask=0u64;");
    }
    for (i, f) in schema
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| !f.optional)
    {
        emit_unchecked_value(
            &f.data,
            out,
            &format!("field_{i}"),
            f.bits,
            false,
            schema.borrow_strings(),
        )?;
    }
    ln(out,"let optional_len=len-reader.as_ptr().offset_from(bytes) as usize;let presence=Self::__qsl_optional_mask(optional_len,explicit_mask);let _=presence;");
    for (j, (i, f)) in schema
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| f.optional)
        .enumerate()
    {
        if optional_scalar_read(
            &f.data,
            out,
            &format!("field_{i}"),
            &format!("(presence>>{j})&1!=0"),
        )? {
            continue;
        }
        ln(
            out,
            format!("let present=(presence>>{j})&1!=0;let field_{i}=if present {{"),
        );
        emit_unchecked_value(
            &f.data,
            out,
            "value",
            f.bits,
            false,
            schema.borrow_strings(),
        )?;
        ln(out, "Some(value)}else{None};");
    }
    ln(out, "Self{");
    for (i, f) in schema.fields.iter().enumerate() {
        ln(out, format!("{}:field_{i},", f.name));
    }
    ln(out, "} }");
    Ok(())
}
pub(super) fn validation(schema: &QslSchema, out: &mut String) -> QslResult<()> {
    let p = StaticOptionalPlan::analyze(schema)?;
    if let StaticPresence::Graph(ref g) = p.presence {
        return length_graph_codegen::validation(schema, g, out);
    }
    if let StaticPresence::Mask { bytes } = p.presence {
        ln(out, format!("let explicit_mask=c.uint({bytes})?;"));
        if p.fields < 64 {
            ln(
                out,
                format!(
                    "if explicit_mask>>{}!=0{{return Err(::qsl::QslError::InvalidValue);}}",
                    p.fields
                ),
            );
        }
    }
    for f in schema.fields.iter().filter(|f| !f.optional) {
        validation_codegen::field(&f.data, out, f.bits, false)?;
    }
    match p.presence {
        StaticPresence::Eof=>ln(out,"let presence=(c.position()<input.len()) as u64;"),
        StaticPresence::Length(_)=>ln(out,format!("let total=input.len()-c.position();let node=*{}_LENGTH_NODES.get(total).ok_or(::qsl::QslError::InvalidValue)? as usize;let point={}_CRITICAL_POINTS.get(node).ok_or(::qsl::QslError::InvalidValue)?;if point.0!=total{{return Err(::qsl::QslError::InvalidValue);}}let presence=point.1;",schema.name,schema.name)),
        StaticPresence::Mask{..}=>ln(out,"let presence=explicit_mask;"),
        StaticPresence::Graph(_)=>unreachable!(),
    }
    for (i, f) in schema.fields.iter().filter(|f| f.optional).enumerate() {
        ln(out, format!("if (presence>>{i})&1!=0{{"));
        validation_codegen::field(&f.data, out, f.bits, false)?;
        ln(out, "}");
    }
    validation_codegen::finish(schema, out)
}
