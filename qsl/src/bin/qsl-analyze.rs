//! Native editor bridge: the exact parser and analysis used by code generation.
#[path = "analyzer/layout.rs"]
mod editor_layout;
use std::{
    collections::HashMap,
    io::{self, Read},
};
fn json(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn main() {
    if std::env::args().any(|a| a == "--capabilities") {
        println!(
            "{{\"editorBundle\":1,\"wireLayout\":1,\"generatedRust\":1,\"qpsCodecRevision\":2,\"qpsByteCodecRevision\":3,\"rustMacros\":{}}}",
            cfg!(feature = "rust-macros")
        );
        return;
    }
    let mut source = String::new();
    let mut rust_sources = Vec::new();
    let input = if std::env::args().any(|a| a == "--editor-bundle") {
        read_bundle(&mut source, &mut rust_sources)
    } else {
        io::stdin().read_to_string(&mut source).map(|_| ())
    };
    if input.is_err() {
        println!("{{\"ok\":false,\"message\":\"invalid UTF-8 input\",\"offset\":0}}");
        return;
    }
    if std::env::args().any(|a| a == "--deq-stdin") {
        match qsl::parse_deq(&source) {
            Ok(_) => println!("{{\"ok\":true,\"schemas\":[]}}"),
            Err(e) => println!(
                "{{\"ok\":false,\"offset\":{},\"message\":{}}}",
                e.offset,
                json(&e.message)
            ),
        }
        return;
    }
    let mut constants = HashMap::new();
    let mut rt = false;
    let mut slots = 16;
    let mut args = std::env::args().skip(1);
    while let Some(path) = args.next() {
        if path == "--editor-bundle" || path == "--generate" {
            continue;
        }
        if path == "--rust-macro-text" {
            if let Some(text) = args.next() {
                rust_sources.push(text);
            }
            continue;
        }
        let text = if path == "--deq-text" {
            args.next()
        } else {
            std::fs::read_to_string(path).ok()
        };
        if let Some(text) = text {
            let d = match qsl::parse_deq(&text) {
                Ok(d) => d,
                Err(e) => {
                    println!(
                        "{{\"ok\":false,\"offset\":0,\"message\":{}}}",
                        json(&format!("DEQ: {e}"))
                    );
                    return;
                }
            };
            constants.extend(d.integer_values());
            if d.configured.contains("QSL_RT") {
                rt = d.generation.qsl_rt;
            }
            if d.configured.contains("QSL_RT_DEFAULT") {
                slots = d.generation.qsl_rt_default;
            }
        }
    }
    #[cfg(feature = "rust-macros")]
    let parsed = if rust_sources.is_empty() {
        qsl::parse_qsl_with_constants(&source, &constants)
    } else {
        qsl::rust_macro::parse_qsl_with_rust(
            &source,
            &constants,
            &rust_sources.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .map_err(|e| qsl::ParseError {
            offset: 0,
            message: e.to_string(),
        })
    };
    #[cfg(not(feature = "rust-macros"))]
    let parsed = if rust_sources.is_empty() {
        qsl::parse_qsl_with_constants(&source, &constants)
    } else {
        Err(qsl::ParseError {
            offset: 0,
            message: "build qsl-analyze with --features rust-macros to import Rust declarations"
                .into(),
        })
    };
    let schemas = match parsed {
        Ok(s) => s,
        Err(e) => {
            println!(
                "{{\"ok\":false,\"offset\":{},\"message\":{}}}",
                e.offset,
                json(&e.message)
            );
            return;
        }
    };
    let generated = if std::env::args().any(|a| a == "--generate") {
        let mut g = qsl::QslInterpreter::new();
        g.config.qsl_rt = rt;
        g.config.qsl_rt_default = slots;
        g.generate(&schemas)
            .ok()
            .and_then(|_| String::from_utf8(g.output).ok())
    } else {
        None
    };
    let mut rows = Vec::new();
    #[cfg(feature = "rust-macros")]
    let rust_names = qsl::rust_macro::parse_rust_macros(
        &rust_sources.iter().map(String::as_str).collect::<Vec<_>>(),
    )
    .unwrap_or_default()
    .into_iter()
    .map(|s| s.name)
    .collect::<Vec<_>>();
    #[cfg(not(feature = "rust-macros"))]
    let rust_names = Vec::<String>::new();
    for s in schemas {
        let quant =
            qsl::payload_quant::PayloadQuantPlan::analyze(&s).expect("validated payload plan");
        let padding = if quant.active { quant.padding_bytes } else { 0 };
        let maximum = qsl::interpreter::schema_maximum_size(&s).ok().flatten();
        let eligible = maximum.is_some_and(|n| {
            n.checked_add(padding)
                .and_then(|n| n.checked_mul(slots))
                .is_some_and(|x| x <= 64 * 1024 * 1024)
        });
        let (bits, index, plain, reason) = if s.state_bits() || s.predict_table() {
            match qsl::static_index::PredictPlan::analyze(&s) {
                Ok(p) => (
                    p.used_bits,
                    p.packed_index().map_or(0, |t| t.len() * 4),
                    p.index.len() * 4,
                    p.reason.unwrap_or("exact static subset-sum lookup"),
                ),
                Err(_) => (0, 0, 0, "unsupported plan"),
            }
        } else {
            (0, 0, 0, "legacy codec")
        };
        let inline = if s.flags & qsl::FLAG_INLINE_ALWAYS != 0 {
            "always"
        } else if s.flags & qsl::FLAG_INLINE_NEVER != 0 {
            "never"
        } else if s.flags & qsl::FLAG_INLINE != 0 {
            "hint"
        } else {
            "compiler default"
        };
        let mut variable_seen = false;
        let mut suggest = false;
        for f in s.fields.iter().filter(|f| !f.optional) {
            if f.data.fixed_wire_size().ok().flatten().is_none() {
                variable_seen = true;
            } else if variable_seen {
                suggest = true;
            }
        }
        let rt_reason = if maximum.is_none() {
            "unbounded field or unknown custom width"
        } else if !eligible {
            "static region payload budget exceeds 64 MiB per schema"
        } else {
            "finite wire upper bound"
        };
        let safety = s.flags & qsl::FLAG_SAFETY != 0;
        let optional_tail = true;
        let structure_macro = s.flags & qsl::FLAG_MACRO != 0;
        let strings = s
            .fields
            .iter()
            .filter_map(|f| {
                let (min, max, compressed) = match f.data {
                    qsl::QslType::CompressedString { min, max } => (min, Some(max), true),
                    qsl::QslType::FixedCompressedString(n) => (n, Some(n), true),
                    qsl::QslType::InlineRawString { min, max }
                    | qsl::QslType::RangedString { min, max } => (min, Some(max), false),
                    qsl::QslType::FixedString(n) => (n, Some(n), false),
                    qsl::QslType::String(min) => (min, None, false),
                    qsl::QslType::GuardString { min, max }
                    | qsl::QslType::CopyGuardString { min, max }
                    | qsl::QslType::BinaryString { min, max } => (min, max, false),
                    _ => return None,
                };
                Some(json(&format!(
                    "{}: UTF-8 {}..{} B; {}",
                    f.name,
                    min,
                    max.map_or("unbounded".into(), |n| n.to_string()),
                    if compressed {
                        "QSL-LZv1, raw fallback; compressed frame must be smaller"
                    } else if s.state_bits() && f.optional {
                        "raw bytes; exact presence/length in D"
                    } else {
                        "raw UTF-8"
                    }
                )))
            })
            .collect::<Vec<_>>()
            .join(",");
        let static_mode = if quant.active && s.opt_table() {
            "packed bit-prefix Boolean graph"
        } else if s.state_bits() && s.opt_table() {
            "legacy descriptor graph driven by D"
        } else if s.opt_table() {
            match qsl::static_optional::StaticOptionalPlan::analyze(&s)
                .expect("validated optional plan")
                .presence
            {
                qsl::static_optional::StaticPresence::Eof => "EOF Boolean predicate",
                qsl::static_optional::StaticPresence::Length(_) => {
                    "static critical lengths; no wire directory"
                }
                qsl::static_optional::StaticPresence::Mask { .. } => {
                    "static prefix graph with presence bits; no wire directory"
                }
                qsl::static_optional::StaticPresence::Graph(_)=>"payload-length event graph; conditional tag reads; native lengths outside payload",
            }
        } else if s.flags & qsl::FLAG_OPT_TABLE_LEGACY != 0 {
            "legacy wire offset directory"
        } else {
            "disabled"
        };
        let length_events = if s.opt_table() && !s.state_bits() && !quant.active {
            match qsl::static_optional::StaticOptionalPlan::analyze(&s).expect("validated length plan").presence{
            qsl::static_optional::StaticPresence::Graph(g)=>format!("{{\"metadataBytes\":{},\"tagBytes\":{},\"maxPayload\":{},\"events\":[{}]}}",g.metadata_bytes,g.tag_bytes,g.max_payload,g.nodes.iter().map(|n|format!("{{\"start\":{},\"end\":{},\"forced\":{},\"possible\":{},\"valid\":{}}}",n.start,n.end,mask_json(n.forced),mask_json(n.possible),n.valid)).collect::<Vec<_>>().join(",")),
            _=>"null".into()
        }
        } else {
            "null".into()
        };
        let wire_layout = editor_layout::describe(&s);
        let hint = s.opt_table() && s.flags & qsl::FLAG_PREDICT_SEQ != 0;
        let byte_maximum = maximum
            .and_then(|n| n.checked_add(qsl::qps_bytes::BYTE_FRAME_HEADER))
            .map_or("null".to_string(), |n| n.to_string());
        let payload=format!("\"qpsByteCodecRevision\":3,\"qpsByteHeader\":16,\"qpsByteMaximum\":{byte_maximum},\"qpsByteAPI\":{safety},\"stateBits\":{},\"stateBitsQuantum\":{},\"qpsCodecRevision\":2,\"payloadQuantRequested\":{},\"payloadQuantActive\":{},\"payloadQuantum\":{},\"payloadPadding\":{padding},\"payloadMaximum\":{},\"rawMaximum\":{},\"payloadReason\":{},\"staticOptional\":{},\"lengthGraph\":{length_events},\"predictSequenceHint\":{hint}",s.state_bits(),s.state_bits_quantum(),s.quant_requested(),quant.active,quant.quantum,quant.max_bytes,quant.raw_max_bytes.map_or("null".into(),|x|x.to_string()),json(quant.reason.unwrap_or("exact quotient and remainder; separate native restoration")),json(static_mode));
        rows.push(format!("{{\"wireLayout\":{wire_layout},\"name\":{},\"maximum\":{},\"rtEligible\":{eligible},\"rtReason\":{},\"strings\":[{strings}],\"rtEnabled\":{},\"slots\":{slots},\"qps\":{},\"quantum\":{},\"dBits\":{bits},\"indexBytes\":{index},\"plainIndexBytes\":{plain},\"reason\":{},\"inline\":{},\"optionalCount\":{},\"fieldCount\":{},\"structureMacro\":{structure_macro},\"safety\":{safety},\"optionalTailRequired\":{optional_tail},\"suggestFixedPrefix\":{suggest},{payload}}}",json(&s.name),maximum.map_or("null".into(),|x|x.to_string()),json(rt_reason),rt&&eligible,s.qps(),if s.qps(){s.quantum()}else{s.state_bits_quantum()},json(reason),json(inline),s.optional_count(),s.fields.len()));
    }
    println!(
        "{{\"ok\":true,\"generated\":{},\"rustMacros\":[{}],\"schemas\":[{}]}}",
        generated.as_ref().map_or("null".into(), |s| json(s)),
        rust_names
            .iter()
            .map(|s| json(s))
            .collect::<Vec<_>>()
            .join(","),
        rows.join(",")
    );
}
fn mask_json(value: u64) -> String {
    if value < (1u64 << 53) {
        value.to_string()
    } else {
        json(&format!("0x{value:016x}"))
    }
}

/// Internal editor transport, not QSL wire format. Length-prefixing source
/// texts avoids Windows' command-line limit for large Rust declarations.
fn read_bundle(source: &mut String, rust: &mut Vec<String>) -> io::Result<()> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut at = 0;
    fn word(bytes: &[u8], at: &mut usize) -> io::Result<usize> {
        let chunk = bytes
            .get(*at..*at + 4)
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        *at += 4;
        Ok(u32::from_le_bytes(chunk.try_into().unwrap()) as usize)
    }
    fn text(bytes: &[u8], at: &mut usize) -> io::Result<String> {
        let len = word(bytes, at)?;
        let chunk = bytes
            .get(*at..at.checked_add(len).ok_or(io::ErrorKind::InvalidData)?)
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        *at += len;
        String::from_utf8(chunk.to_vec()).map_err(|_| io::ErrorKind::InvalidData.into())
    }
    *source = text(&bytes, &mut at)?;
    let count = word(&bytes, &mut at)?;
    if count > 256 {
        return Err(io::ErrorKind::InvalidData.into());
    }
    for _ in 0..count {
        rust.push(text(&bytes, &mut at)?);
    }
    if at != bytes.len() {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(())
}
