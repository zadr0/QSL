use super::*;
use crate::length_graph::LengthGraph;
fn node(s: &QslSchema) -> String {
    format!(
        "{}_CRITICAL_POINTS.get_unchecked(Self::__qsl_event_index(i))",
        s.name
    )
}
pub(super) fn tables(s: &QslSchema, p: &LengthGraph, out: &mut String) -> QslResult<()> {
    let name = &s.name;
    let rows = p
        .nodes
        .iter()
        .map(|n| (n.start, n.end, n.forced, n.possible, n.valid))
        .collect::<Vec<_>>();
    ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_CRITICAL_POINTS:[(usize,usize,u64,u64,bool);{}]={rows:?};",rows.len()));
    let shared = p.nodes.len() > 128;
    let functions = p
        .nodes
        .iter()
        .enumerate()
        .map(|(j, _)| {
            if shared {
                format!("{name}::__qsl_event_shared")
            } else {
                format!("{name}::__qsl_event_{j}")
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_EVENT_TABLE:[unsafe fn(*const u8,usize)->u64;{}]=[{functions}];",p.nodes.len()));
    if p.nodes.len() > 33 {
        let capacity = p.nodes.len().next_power_of_two();
        let mut bounds = p
            .nodes
            .iter()
            .skip(1)
            .map(|n| n.start - 1)
            .collect::<Vec<_>>();
        bounds.resize(capacity, usize::MAX);
        ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_EVENT_BOUNDS:[usize;{capacity}]={bounds:?};"));
    }
    let recipes = p
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)|format!("::qsl::length_graph::OffsetRecipe{{prior_fields:{},min:{},max:{},length_slot:{:?}}}",if i==0{0}else{(1u64<<i)-1},f.min,f.max,f.slot))
        .collect::<Vec<_>>();
    ln(out,format!("#[allow(dead_code,non_upper_case_globals)] static {name}_OFFSET_TABLE:[::qsl::length_graph::OffsetRecipe;{}]=[{}];",recipes.len(),recipes.join(",")));
    qps_codegen::callback_table(s, out)
}
pub(super) fn methods(s: &QslSchema, p: &LengthGraph, out: &mut String) {
    ln(out,"/// Numeric boundary operands; verify branches in release assembly.\n#[inline] fn __qsl_event_index(len:usize)->usize{let _=len;");
    if p.nodes.len() <= 33 {
        let mut terms = vec!["0usize".into()];
        terms.extend(
            p.nodes
                .iter()
                .skip(1)
                .map(|n| format!("((len>{}) as usize)", n.start - 1)),
        );
        ln(out, terms.join("+"));
    } else {
        let mut step = p.nodes.len().next_power_of_two() / 2;
        ln(out, "let mut rank=0usize;");
        while step != 0 {
            ln(
                out,
                format!(
                    "rank+=((len>unsafe{{*{}_EVENT_BOUNDS.get_unchecked(rank+{})}})as usize)*{step};",
                    s.name,
                    step - 1
                ),
            );
            step >>= 1;
        }
        ln(out, "rank");
    }
    ln(out, "}");
    ln(out,format!("pub const OPT_TABLE_EVENT_COUNT:usize={};pub const OPT_TABLE_MAX_PAYLOAD:usize={};\n/// Numeric interval rank only; checked decoding separately validates payload length.\n#[inline] pub fn predict_index(payload_len:usize)->usize{{Self::__qsl_event_index(payload_len)}}",p.nodes.len(),p.max_payload));
    ln(out,format!("pub const OPT_TABLE_HEADER_BYTES:usize={};pub const OPT_TABLE_TAG_BYTES:usize={};\n#[inline] unsafe fn __qsl_optional_mask(i:usize,explicit:u64)->u64{{let point={};point.2|(explicit&(point.3^point.2))}}",p.metadata_bytes,p.tag_bytes,node(s)));
    if p.nodes.len() > 128 {
        ln(out,format!("#[inline] unsafe fn __qsl_event_shared(metadata:*const u8,event:usize)->u64{{let _=metadata;let point={}_CRITICAL_POINTS.get_unchecked(event);",s.name));
        emit_tag(out, p, "point.2!=point.3");
        ln(out, "point.2|(explicit&(point.3^point.2))}");
    } else {
        for (j, n) in p.nodes.iter().enumerate() {
            ln(out,format!("#[inline] unsafe fn __qsl_event_{j}(metadata:*const u8,_event:usize)->u64{{let _=metadata;"));
            if n.forced == n.possible {
                ln(out, format!("{}u64}}", n.forced));
            } else {
                emit_tag(out, p, "true");
                ln(
                    out,
                    format!("{}u64|(explicit&{}u64)}}", n.forced, n.possible ^ n.forced),
                );
            }
        }
    }
}
fn emit_tag(out: &mut String, p: &LengthGraph, needed: &str) {
    if p.tag_bytes == 0 {
        ln(out, "let explicit=0u64;");
        return;
    }
    let ty = format!("u{}", p.tag_bytes * 8);
    if needed == "true" {
        ln(
            out,
            format!(
                "let explicit=<{ty}>::from_le(metadata.cast::<{ty}>().read_unaligned())as u64;"
            ),
        );
    } else {
        ln(out,format!("let zero=0{ty};let source=::qsl::payload_quant::selected_address(std::hint::select_unpredictable({needed},metadata.cast::<{ty}>(),&zero));let explicit=<{ty}>::from_le(source.read_unaligned())as u64;"));
    }
}
pub(super) fn size(s: &QslSchema, p: &LengthGraph, out: &mut String) -> QslResult<()> {
    ln(
        out,
        format!(
            "#[inline] fn __qsl_size(&self)->usize{{let mut n={}usize;",
            p.metadata_bytes
        ),
    );
    for f in s.fields.iter().filter(|f| !f.optional) {
        ln(out, "let mut __qsl_bits=0usize;");
        emit_size_stored(f, out, &format!("&self.{}", f.name), f.bits, false)?;
        ln(out, "n+=(__qsl_bits+7)>>3;");
    }
    for (f, d) in s.fields.iter().filter(|f| f.optional).zip(&p.fields) {
        if d.variable {
            ln(
                out,
                format!("n+=self.{}.as_ref().map_or(0,|value|value.len());", f.name),
            );
        } else {
            ln(
                out,
                format!("n+={}*(self.{}.is_some() as usize);", d.max, f.name),
            );
        }
    }
    ln(out, "n}");
    Ok(())
}
pub(super) fn write(s: &QslSchema, p: &LengthGraph, out: &mut String) -> QslResult<()> {
    ln(out, "let mut writer=::qsl::QslByteWriter::from_raw(to);");
    if p.tag_bytes > 0 {
        let mut terms = vec!["0u64".into()];
        terms.extend(
            s.fields
                .iter()
                .filter(|f| f.optional)
                .enumerate()
                .map(|(i, f)| format!("((self.{}.is_some() as u64)<<{i})", f.name)),
        );
        ln(
            out,
            format!(
                "let mask={};writer.write_bytes_unchecked(&mask.to_le_bytes()[..{}]);",
                terms.join("|"),
                p.tag_bytes
            ),
        );
    }
    for (f, d) in s.fields.iter().filter(|f| f.optional).zip(&p.fields) {
        if let Some((_, bytes)) = d.slot {
            let ty = format!("u{}", bytes * 8);
            ln(out,format!("let length=self.{}.as_ref().map_or(0,|v|v.len()) as {ty};writer.write_bytes_unchecked(&length.to_le_bytes());",f.name));
        }
    }
    for f in s.fields.iter().filter(|f| !f.optional) {
        fast_stored_write(f, out, &format!("&self.{}", f.name), f.bits, false)?;
    }
    for (f, d) in s.fields.iter().filter(|f| f.optional).zip(&p.fields) {
        if optional_scalar_write(f, out)? {
            continue;
        }
        ln(
            out,
            format!("if let Some(value)=self.{}.as_ref(){{", f.name),
        );
        if d.variable {
            ln(out, "writer.write_bytes_unchecked(value.as_bytes());");
        } else {
            fast_stored_write(f, out, "value", f.bits, false)?;
        }
        ln(out, "}");
    }
    ln(out, "writer.as_mut_ptr().offset_from(to) as usize");
    Ok(())
}
fn fixed_tail(p: &LengthGraph, j: usize) -> String {
    let mut terms = vec!["0usize".into()];
    terms.extend(
        p.fields
            .iter()
            .enumerate()
            .skip(j + 1)
            .filter(|(_, f)| !f.variable)
            .map(|(i, f)| format!("{}*(((presence>>{i})&1)as usize)", f.max)),
    );
    terms.join("+")
}
pub(super) fn read(s: &QslSchema, p: &LengthGraph, out: &mut String) -> QslResult<()> {
    ln(out,format!("#[inline] unsafe fn __qsl_read_sized(bytes:*const u8,len:usize)->Self{{let mut reader=::qsl::QslByteReader::new(bytes.add({}));",p.metadata_bytes));
    emit_required_values(s, out)?;
    ln(out,"let payload=reader.as_ptr();let i=len-payload.offset_from(bytes)as usize;let event=Self::__qsl_event_index(i);");
    let powers = p.tag_bytes == 0
        && p.fields
            .iter()
            .all(|f| !f.variable && f.min.is_power_of_two());
    if powers {
        let terms = p
            .fields
            .iter()
            .enumerate()
            .map(|(j, f)| format!("(((i>>{})&1)as u64)<<{j}", f.min.trailing_zeros()))
            .collect::<Vec<_>>()
            .join("|");
        ln(out, format!("let presence={terms};let _=event;"));
    } else if s.flags & crate::FLAG_PREDICT_SEQ != 0 {
        ln(
            out,
            format!(
                "let point={}_CRITICAL_POINTS.get_unchecked(event);let metadata=bytes;",
                s.name
            ),
        );
        emit_tag(out, p, "point.2!=point.3");
        ln(out, "let presence=point.2|(explicit&(point.3^point.2));");
    } else {
        ln(
            out,
            format!(
                "let presence=(*{}_EVENT_TABLE.get_unchecked(event))(bytes,event);",
                s.name
            ),
        );
    }
    tagged_optional_codegen::result(s, out);
    for (j, (f, d)) in s
        .fields
        .iter()
        .filter(|f| f.optional)
        .zip(&p.fields)
        .enumerate()
    {
        ln(out, format!("if (presence>>{j})&1!=0{{"));
        if d.variable {
            ln(
                out,
                format!(
                    "let derived=i-reader.as_ptr().offset_from(payload)as usize-({});",
                    fixed_tail(p, j)
                ),
            );
            if let Some((offset, bytes)) = d.slot {
                let ty = format!("u{}", bytes * 8);
                let later = p
                    .fields
                    .iter()
                    .enumerate()
                    .skip(j + 1)
                    .filter(|(_, f)| f.variable)
                    .fold(0u64, |m, (k, _)| m | (1u64 << k));
                ln(out,format!("let fallback=(derived as {ty}).to_le();let source=::qsl::payload_quant::selected_address(std::hint::select_unpredictable(presence&{later}u64!=0,bytes.add({offset}).cast::<{ty}>(),&fallback));let length=<{ty}>::from_le(source.read_unaligned())as usize;"));
            } else {
                ln(out, "let length=derived;");
            }
        }
        tagged_optional_codegen::dispatch(
            s,
            out,
            &(f.tag - 1).to_string(),
            if d.variable { "length" } else { "0" },
        );
        ln(out, "}");
    }
    ln(out, "result}");
    Ok(())
}
pub(super) fn validation(
    s: &QslSchema,
    p: &LengthGraph,
    out: &mut String,
    capture: bool,
) -> QslResult<()> {
    ln(
        out,
        format!("let explicit=c.uint({})?;let _=explicit;", p.tag_bytes),
    );
    if p.tag_bytes > 0 && p.fields.len() < 64 {
        ln(
            out,
            format!(
                "if explicit>>{}!=0{{return Err(::qsl::QslError::InvalidValue);}}",
                p.fields.len()
            ),
        );
    }
    for (j, d) in p.fields.iter().enumerate() {
        if let Some((_, bytes)) = d.slot {
            ln(out, format!("let length_{j}=c.uint({bytes})? as usize;"));
        }
    }
    checked_codegen::required(s, out, capture)?;
    ln(out,format!("let i=input.len()-c.position();if i>{}{{return Err(::qsl::QslError::InvalidValue);}}let event=Self::__qsl_event_index(i);let point={}_CRITICAL_POINTS.get(event).ok_or(::qsl::QslError::InvalidValue)?;if !point.4{{return Err(::qsl::QslError::InvalidValue);}}",p.max_payload,s.name));
    // With no tag region, point.2 is the complete mask, not a wire value.
    if p.tag_bytes == 0 {
        ln(out, "let presence=point.2;");
    } else {
        ln(out,"if explicit&point.2!=point.2||explicit&!point.3!=0{return Err(::qsl::QslError::InvalidValue);}let presence=point.2|(explicit&(point.3^point.2));");
    }
    if capture {
        checked_codegen::begin_result(s, out);
    }
    for (j, (f, d)) in s
        .fields
        .iter()
        .filter(|f| f.optional)
        .zip(&p.fields)
        .enumerate()
    {
        if d.slot.is_some() {
            ln(out,format!("if presence>>{j}&1==0&&length_{j}!=0{{return Err(::qsl::QslError::InvalidValue);}}"));
        }
        ln(out, format!("if presence>>{j}&1!=0{{"));
        if d.variable {
            let length = if d.slot.is_some() {
                format!("length_{j}")
            } else {
                format!("(input.len()-c.position()).checked_sub({}).ok_or(::qsl::QslError::UnexpectedEof)?",fixed_tail(p,j))
            };
            if capture {
                checked_codegen::dispatch(out, &(f.tag - 1).to_string(), &length);
            } else {
                checked_codegen::field(
                    f,
                    out,
                    f.bits,
                    false,
                    s.borrow_strings(),
                    false,
                    Some(&length),
                )?;
            }
        } else {
            if capture {
                checked_codegen::dispatch(out, &(f.tag - 1).to_string(), "0");
            } else {
                checked_codegen::field(f, out, f.bits, false, s.borrow_strings(), false, None)?;
            }
        }
        ln(out, "}");
    }
    if capture {
        ln(out, "c.finish()?;Ok(result)}");
        Ok(())
    } else {
        checked_codegen::complete(s, out, false)
    }
}
