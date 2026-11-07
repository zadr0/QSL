//! Native editor bridge: the exact parser and analysis used by code generation.
#[path = "analyzer/layout.rs"]
mod editor_layout;
#[path = "analyzer/memory.rs"]
mod editor_memory;
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
            "{{\"editorBundle\":1,\"bitmapRevision\":2,\"bitmapMaxOptionals\":128,\"wireLayout\":1,\"schemaHashVersion\":1,\"checkedSchemas\":1,\"optionalModes\":2,\"generatedRust\":1,\"memoryLayout\":1,\"safetyOptIn\":1,\"qpsCodecRevision\":2,\"qpsByteCodecRevision\":3,\"rustMacros\":{}}}",
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
    let mut use_hash = qsl::SchemaHashAlgorithm::default();
    let mut configured = std::collections::HashSet::new();
    let mut maximum_data = qsl::validation::DEFAULT_MAX_DATA_SIZE;
    let mut rt = false;
    let mut slots = 16;
    let mut stack_limit = 256;
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
            for key in &d.configured {
                if !configured.insert(key.clone()) {
                    println!(
                        "{{\"ok\":false,\"offset\":0,\"message\":{}}}",
                        json(&format!("DEQ config `{key}` is defined more than once"))
                    );
                    return;
                }
            }
            constants.extend(d.integer_values());
            if d.configured.contains("STACK_LIMIT") {
                stack_limit = d.generation.stack_limit;
            }
            if d.configured.contains("MAX_DATA_SIZE") {
                maximum_data = d.generation.max_data_size;
            }
            if d.configured.contains("USE_HASH") {
                use_hash = d.generation.use_hash;
            }
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
    let mut diagnostics = Vec::<qsl::ParseError>::new();
    let schemas = match parsed {
        Ok(s) => s,
        Err(e) => {
            if std::env::args().any(|a| a == "--editor-bundle") {
                let mut recovered = Vec::new();
                let mut names = std::collections::HashSet::new();
                for (start, end) in editor_spans(&source) {
                    #[cfg(feature = "rust-macros")]
                    let result = if rust_sources.is_empty() {
                        qsl::parse_qsl_with_constants(&source[start..end], &constants)
                    } else {
                        qsl::rust_macro::parse_qsl_with_rust(
                            &source[start..end],
                            &constants,
                            &rust_sources.iter().map(String::as_str).collect::<Vec<_>>(),
                        )
                        .map_err(|e| qsl::ParseError {
                            offset: 0,
                            message: e.to_string(),
                        })
                    };
                    #[cfg(not(feature = "rust-macros"))]
                    let result = qsl::parse_qsl_with_constants(&source[start..end], &constants);
                    match result {
                        Ok(schemas) => {
                            for s in schemas {
                                if names.insert(s.name.clone()) {
                                    recovered.push(s)
                                } else {
                                    diagnostics.push(qsl::ParseError {
                                        offset: start,
                                        message: format!("duplicate schema `{}`", s.name),
                                    })
                                }
                            }
                        }
                        Err(mut error) => {
                            error.offset = error.offset.saturating_add(start);
                            diagnostics.push(error)
                        }
                    }
                }
                if !diagnostics.iter().any(|d| d.message == e.message) {
                    diagnostics.push(e)
                }
                recovered
            } else {
                println!(
                    "{{\"ok\":false,\"offset\":{},\"message\":{}}}",
                    e.offset,
                    json(&e.message)
                );
                return;
            }
        }
    };
    let generated = if std::env::args().any(|a| a == "--generate") {
        let mut g = qsl::QslInterpreter::new();
        g.config.use_hash = use_hash;
        g.config.max_data_size = maximum_data;
        g.config.qsl_rt = rt;
        g.config.qsl_rt_default = slots;
        g.config.stack_limit = stack_limit;
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
        let mut s = s;
        if let Err(message) = qsl::modernization::check_storage_limit(&s, stack_limit) {
            diagnostics.push(qsl::ParseError {
                offset: 0,
                message: format!("{}: {message}", s.name),
            });
            continue;
        }
        if s.options.transmutate && s.borrow_strings() {
            s.flags &= !(qsl::FLAG_LIFETIME | qsl::FLAG_FIELD_BORROW);
        }
        let s = match qsl::modernization::lower_storage(&s, stack_limit) {
            Ok(s) => s,
            Err(e) => {
                diagnostics.push(qsl::ParseError {
                    offset: 0,
                    message: format!("{}: {e}", s.name),
                });
                continue;
            }
        };
        let slots = s.options.slots.unwrap_or(slots);
        let hashes = match s.hashes(use_hash) {
            Ok(hashes) => hashes,
            Err(error) => {
                println!(
                    "{{\"ok\":false,\"offset\":0,\"message\":{}}}",
                    json(&format!("schema identity: {error}"))
                );
                return;
            }
        };
        let hash_json = format!("\"hashes\":{{\"version\":{},\"algorithm\":{},\"bits\":{},\"strict\":{},\"light\":{},\"complete\":{}}}", hashes.version, json(hashes.algorithm.name()), hashes.algorithm.bits(), json(&hashes.strict_hex()), json(&hashes.light_hex()), hashes.complete);
        let quant =
            qsl::payload_quant::PayloadQuantPlan::analyze(&s).expect("validated payload plan");
        let hybrid = qsl::hybrid::HybridPlan::analyze(&s).expect("validated scalar prefix");
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
                    qsl::QslType::BorrowedString { min, max } => (min, max, false),
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
        let static_mode = if s.bitmap() && s.opt_table() {
            "optional-only bitmap; dense reader table; descending markers"
        } else if s.bitmap() {
            "optional-only bitmap; match dispatch; descending markers"
        } else if quant.active && s.opt_table() {
            "packed numeric kernel; no wire tags"
        } else if s.state_bits() && s.opt_table() {
            "descriptor presence with tag-indexed reader dispatch"
        } else if s.predicted_opt_table() {
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
                qsl::static_optional::StaticPresence::Graph(_) => {
                    "payload-length event graph plus tag-indexed reader dispatch"
                }
            }
        } else if s.opt_table() {
            "tag-indexed reader dispatch; default wire format"
        } else if s.flags & qsl::FLAG_OPT_TABLE_LEGACY != 0 {
            "legacy wire offset directory"
        } else {
            "disabled"
        };
        let length_events = if s.predicted_opt_table() && !quant.active {
            match qsl::static_optional::StaticOptionalPlan::analyze(&s).expect("validated length plan").presence{
            qsl::static_optional::StaticPresence::Graph(g)=>format!("{{\"metadataBytes\":{},\"tagBytes\":{},\"maxPayload\":{},\"events\":[{}]}}",g.metadata_bytes,g.tag_bytes,g.max_payload,g.nodes.iter().map(|n|format!("{{\"start\":{},\"end\":{},\"forced\":{},\"possible\":{},\"valid\":{}}}",n.start,n.end,mask_json(n.forced),mask_json(n.possible),n.valid)).collect::<Vec<_>>().join(",")),
            _=>"null".into()
        }
        } else {
            "null".into()
        };
        let wire_layout = editor_layout::describe(&s);
        let hybrid_json=hybrid.as_ref().map_or("null".into(),|p|format!("{{\"minimumBytes\":{},\"maximumBytes\":{},\"lengthHeaderBytes\":{},\"fieldCount\":{}}}",p.prefix_minimum,p.prefix_bytes,p.length_header,p.prefix.fields.len()));
        let byte_api = safety
            && s.fields.iter().all(|f| {
                matches!(
                    f.data,
                    qsl::QslType::Byte
                        | qsl::QslType::Integer { .. }
                        | qsl::QslType::RangedInteger { .. }
                        | qsl::QslType::Flags(_)
                )
            });
        let memory = editor_memory::describe(&s);
        let groups=qsl::modernization::register_groups(&s).unwrap_or_default().iter().map(|g|format!("{{\"firstField\":{},\"fieldCount\":{},\"wireOffset\":{},\"readBytes\":{},\"registerBits\":{},\"metadataBytes\":{}}}",g.first,g.fields,g.offset.map_or("null".into(),|n|n.to_string()),g.bytes,g.register_bits,g.metadata_bytes)).collect::<Vec<_>>().join(",");
        let regions = s
            .options
            .regions
            .iter()
            .map(|r| {
                format!(
                    "{{\"type\":{},\"firstField\":{},\"endField\":{}}}",
                    json(&r.rust_type),
                    r.start,
                    r.end
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let prediction = s.predict_seq();
        let dispatch = s.opt_table() && !quant.active;
        let hint = false;
        let byte_maximum = maximum
            .and_then(|n| n.checked_add(qsl::qps_bytes::BYTE_FRAME_HEADER))
            .map_or("null".to_string(), |n| n.to_string());
        let payload=format!("\"qpsByteCodecRevision\":3,\"qpsByteHeader\":16,\"qpsByteMaximum\":{byte_maximum},\"qpsByteAPI\":{byte_api},\"scalarPrefix\":{hybrid_json},\"stateBits\":{},\"stateBitsQuantum\":{},\"qpsCodecRevision\":2,\"payloadQuantRequested\":{},\"payloadQuantActive\":{},\"payloadQuantum\":{},\"payloadPadding\":{padding},\"payloadMaximum\":{},\"rawMaximum\":{},\"payloadReason\":{},\"staticOptional\":{},\"lengthGraph\":{length_events},\"predictSequenceHint\":{hint},\"predictSequence\":{prediction},\"tagDispatch\":{dispatch}",s.state_bits(),s.state_bits_quantum(),s.quant_requested(),quant.active,quant.quantum,quant.max_bytes,quant.raw_max_bytes.map_or("null".into(),|x|x.to_string()),json(quant.reason.unwrap_or("exact quotient and remainder; separate native restoration")),json(static_mode));
        rows.push(format!("{{\"maxDataSize\":{maximum_data},{hash_json},\"wireLayout\":{wire_layout},\"memory\":{memory},\"registerGroups\":[{groups}],\"nativeRegions\":[{regions}],\"stackLimit\":{stack_limit},\"name\":{},\"maximum\":{},\"rtEligible\":{eligible},\"rtReason\":{},\"strings\":[{strings}],\"rtEnabled\":{},\"slots\":{slots},\"qps\":{},\"quantum\":{},\"dBits\":{bits},\"indexBytes\":{index},\"plainIndexBytes\":{plain},\"reason\":{},\"inline\":{},\"optionalCount\":{},\"fieldCount\":{},\"structureMacro\":{structure_macro},\"safety\":{safety},\"optionalTailRequired\":{optional_tail},\"suggestFixedPrefix\":{suggest},{payload}}}",json(&s.name),maximum.map_or("null".into(),|x|x.to_string()),json(rt_reason),rt&&eligible,s.qps(),if s.qps(){s.quantum()}else{s.state_bits_quantum()},json(reason),json(inline),s.optional_count(),s.fields.len()));
    }
    let diagnostic_json = diagnostics
        .iter()
        .map(|e| {
            format!(
                "{{\"offset\":{},\"message\":{}}}",
                e.offset,
                json(&e.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"ok\":true,\"diagnostics\":[{diagnostic_json}],\"generated\":{},\"rustMacros\":[{}],\"schemas\":[{}]}}",
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

/// Recovery is editor-only; normal builds continue to reject the entire source.
fn editor_spans(source: &str) -> Vec<(usize, usize)> {
    let b = source.as_bytes();
    let mut out = Vec::new();
    let mut start = None;
    let mut i = 0;
    let mut depth = 0;
    while i < b.len() {
        if b[i..].starts_with(b"//") {
            while i < b.len() && b[i] != b'\n' {
                i += 1
            }
            continue;
        }
        if b[i..].starts_with(b"/*") {
            i += 2;
            while i + 1 < b.len() && !b[i..].starts_with(b"*/") {
                i += 1
            }
            i = (i + 2).min(b.len());
            continue;
        }
        if b[i] == b'"' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i = (i + 2).min(b.len())
                } else if b[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1
                }
            }
            continue;
        }
        if b[i] == b'&' {
            if let Some(s) = start.take() {
                out.push((s, i))
            }
            start = Some(i);
            depth = 0;
        } else if b[i] == b'{' {
            depth += 1;
        } else if b[i] == b'}' && depth > 0 {
            depth -= 1;
            if depth == 0 {
                if let Some(s) = start.take() {
                    out.push((s, i + 1))
                }
            }
        }
        i += 1;
    }
    if let Some(s) = start {
        out.push((s, b.len()))
    }
    out
}
