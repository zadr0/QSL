use qsl_interpreter::{
    layout::StorageKind, optimizer::registers::CpuProfile, parse_qsl, prepare_schemas,
    GenerationConfig, RustGenerator,
};
use serde_json::json;
use std::{env, fmt::Write as _, fs, path::PathBuf};

const FIXED: &str = "u32(0..4095) a;u32(0..4095) b;u32(0..4095) c;u32(0..4095) d;";
const PRESENCE: &str = "u8? a;u16? b;u32? c;";
const TEXT: &str = "u32 id;str?(0..1024) text;";
const NATIVE: &str = "u64 stamp;u32 id;u16(0..4095) zone;u8 code;u8 flags;";
const MIXED: &str = "u8 flags;u64 stamp;u16 zone;";
const DOMAINS: &str = "u32(0..4092) a|step=4|;u8 b|values=[1,3,7,15]|;";
const SAFETY_TEXT: &str = "u32 id;str(0..8192) text;";

struct Candidate {
    group: &'static str,
    profile: String,
    attrs: String,
    fields: String,
    stack: usize,
    cpu: CpuProfile,
}

fn candidates() -> Vec<Candidate> {
    let mut result = Vec::new();
    let mut add =
        |group: &'static str, profile: String, attrs: String, fields: &str, stack, cpu| {
            result.push(Candidate {
                group,
                profile,
                attrs,
                fields: fields.into(),
                stack,
                cpu,
            });
        };
    let portable = CpuProfile::Portable;
    let native = CpuProfile::KabyLake;
    for (group, fields) in [
        ("fixed", FIXED),
        ("presence", PRESENCE),
        ("optional_text", TEXT),
    ] {
        for (label, attrs) in [
            ("auto", "auto"),
            ("compact", "auto,compact"),
            ("aligned", "auto,compact,alignment"),
        ] {
            add(group, label.into(), attrs.into(), fields, 4096, native);
        }
        for shift in [false, true] {
            for mask in 0..8 {
                let options = format!(
                    "shift={shift},bitmap={},pointing={},group={}",
                    mask & 1 != 0,
                    mask & 2 != 0,
                    mask & 4 != 0
                );
                add(
                    group,
                    format!(
                        "aggressive_shift{}_bm{}_point{}_group{}",
                        shift as u8,
                        mask & 1,
                        (mask >> 1) & 1,
                        (mask >> 2) & 1
                    ),
                    format!("auto,aggressive[{options}]"),
                    fields,
                    4096,
                    native,
                );
            }
        }
        add(
            group,
            "aggressive_default".into(),
            "auto,aggressive".into(),
            fields,
            4096,
            native,
        );
    }
    for (profile, attr, cpu) in [
        (
            "compact_group_native",
            "auto,compact,aggressive[shift=false,group=true]",
            native,
        ),
        (
            "compact_group_portable",
            "auto,compact,aggressive[shift=false,group=true]",
            portable,
        ),
        ("inline_hint", "auto,compact,inline=hint", native),
        ("inline_always", "auto,compact,inline=always", native),
        ("inline_never", "auto,compact,inline=never", native),
        ("loyal_cache_32", "auto,loyal_cache,cache=32", native),
        ("rt_slots_1", "auto,slots=1", native),
        ("rt_slots_16", "auto,slots=16", native),
    ] {
        add("fixed", profile.into(), attr.into(), FIXED, 4096, cpu);
    }
    for (profile, attr) in [
        ("ps", "predict_seq"),
        ("ot_static", "opt_table=static"),
        ("ot_legacy", "opt_table=legacy"),
        ("ps_ot", "predict_seq,opt_table=static"),
        ("bitmap", "optionals=bitmap"),
        ("bitmap_ot", "optionals=bitmap,opt_table=static"),
        ("compact_bitmap", "compact,optionals=bitmap"),
        ("ps_bitmap", "predict_seq,optionals=bitmap"),
        (
            "ps_ot_object_bitmap",
            "predict_seq,opt_table=static,aggressive[bitmap=true]",
        ),
        (
            "wire_and_object_bitmap",
            "optionals=bitmap,aggressive[bitmap=true]",
        ),
        ("compact_object_bitmap", "compact,aggressive[bitmap=true]"),
    ] {
        add(
            "presence",
            profile.into(),
            format!("auto,{attr}"),
            PRESENCE,
            4096,
            native,
        );
    }
    for (mode, widths) in [
        ("line", &[16, 32, 64][..]),
        ("index(dynamic)", &[32, 64]),
        ("index(strict)", &[32, 64]),
    ] {
        for width in widths {
            add(
                "dyn",
                format!(
                    "{}_{}",
                    mode.replace(['(', ')'], "_").trim_end_matches('_'),
                    width
                ),
                format!("auto,dyn[type={mode},width={width},revision=1]"),
                "u8? a=1;u16? b=2;u32? c=3;",
                4096,
                native,
            );
        }
    }
    for q in [1, 2, 4, 8, 16] {
        add(
            "presence",
            format!("state_bits_q{q}"),
            format!("auto,state_bits=Q{q}"),
            PRESENCE,
            4096,
            native,
        );
        if q >= 2 {
            add(
                "quantized",
                format!("qps_q{q}"),
                format!("auto,qps=Q{q}"),
                FIXED,
                4096,
                native,
            );
            add(
                "quantized",
                format!("qps_q{q}_ot"),
                format!("auto,qps=Q{q},opt_table=static"),
                PRESENCE,
                4096,
                native,
            );
        }
    }
    for (profile, attrs) in [
        ("declared", ""),
        ("auto", "auto"),
        ("auto_wire_v1", "auto[wire=1]"),
    ] {
        add("layout", profile.into(), attrs.into(), MIXED, 4096, native);
    }
    for (profile, attrs) in [
        ("fields", "auto"),
        ("transmute_copy", "auto,transmutate"),
        ("transmute_overlay", "auto,transmutate,lifetime"),
    ] {
        add("native", profile.into(), attrs.into(), NATIVE, 4096, native);
    }
    add("native_region", "named_region".into(), "auto".into(),
        "#region(aggressive_transmutate=\"Prefix\");u64 stamp;u32 id;u16(0..4095) zone;u8 code;u8 flags;#end_region;u64 suffix;", 4096, native);
    add(
        "native_region",
        "fields".into(),
        "auto".into(),
        &format!("{NATIVE}u64 suffix;"),
        4096,
        native,
    );
    for (profile, attrs, fields, stack) in [
        ("heap_string", "auto", TEXT, 0),
        ("stack_string", "auto,aggressive[shift=false]", TEXT, 4096),
        (
            "shifted_stack_string",
            "auto,aggressive[shift=true]",
            TEXT,
            4096,
        ),
        (
            "qsl_lz_inline",
            "auto",
            "u32 id;str?(0..1024) text|decompress=true|;",
            4096,
        ),
        (
            "qsl_lz_heap",
            "auto",
            "u32 id;str?(0..1024) text|decompress=true|;",
            0,
        ),
    ] {
        add(
            "string_storage",
            profile.into(),
            attrs.into(),
            fields,
            stack,
            native,
        );
    }
    for (profile, attrs, stack) in [
        ("heap", "auto", 0),
        ("inline", "auto,aggressive[shift=false]", 4096),
        ("inline_shifted", "auto,aggressive[shift=true]", 4096),
        (
            "inline_shifted_bitmap",
            "auto,aggressive[shift=true,bitmap=true]",
            4096,
        ),
    ] {
        add(
            "arrays",
            profile.into(),
            attrs.into(),
            "u32 id;vec<u16>?(0..32) items;",
            stack,
            native,
        );
    }
    for (profile, attrs) in [
        ("auto", "auto"),
        ("compact", "auto,compact"),
        ("aligned", "auto,compact,alignment"),
    ] {
        add(
            "domain",
            profile.into(),
            attrs.into(),
            DOMAINS,
            4096,
            native,
        );
    }
    for (profile, attrs) in [
        ("owned", "auto"),
        ("borrowed", "auto,lifetime"),
        ("borrowed_aligned", "auto,lifetime,compact,alignment"),
    ] {
        add(
            "safety_text",
            profile.into(),
            attrs.into(),
            SAFETY_TEXT,
            0,
            native,
        );
    }
    add(
        "domain",
        "defaults".into(),
        "auto".into(),
        "u32(0..4092) a|step=4||default=4|;u8 b|values=[1,3,7,15]||default=3|;",
        4096,
        native,
    );
    add(
        "safety_text",
        "borrowed_field".into(),
        "auto".into(),
        "u32 id;str(0..8192) text|borrow=true|;",
        0,
        native,
    );
    result
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut code = String::new();
    let mut registry = String::from(
        "fn register(bench: &mut Runner) {\nlet mut cases: Vec<fn(&mut Runner)> = vec![\n",
    );
    let mut manifest = Vec::new();
    let mut schemas = String::new();
    for (i, candidate) in candidates().into_iter().enumerate() {
        let name = format!("P{i}");
        let attrs = format!(
            "safety{}{}",
            if candidate.attrs.is_empty() { "" } else { "," },
            candidate.attrs
        );
        let schema = format!("&{name}[{attrs}] {{ {} }}", candidate.fields);
        let config = GenerationConfig {
            max_data_size: 32768,
            stack_limit: candidate.stack,
            qsl_rt: candidate.profile.starts_with("rt_slots_"),
            cpu_profile: candidate.cpu,
            ..Default::default()
        };
        let outcome = (|| -> Result<_, String> {
            let parsed = parse_qsl(&schema).map_err(|e| e.to_string())?;
            let prepared =
                prepare_schemas(&parsed, config).map_err(|e| format!("analysis: {e:?}"))?;
            let mut gen = RustGenerator::new();
            gen.generate_prepared(&prepared)
                .map_err(|e| format!("Rust generation: {e:?}"))?;
            let source = String::from_utf8(gen.output).unwrap();
            let lowered = prepared.schemas()[0].schema();
            let fields = lowered.fields.iter().map(|f|json!({"name":f.name,"storage":format!("{:?}",f.storage),"shifted_length":f.shifted_length})).collect::<Vec<_>>();
            let mut args = Vec::new();
            for field in &lowered.fields {
                let val = match (candidate.group, field.name.as_str()) {
                    ("fixed" | "quantized", "a" | "b" | "c" | "d") if candidate.fields == FIXED => {
                        let salt = field.name.as_bytes()[0] - b'a' + 1;
                        format!("number(i,{salt})")
                    }
                    ("presence" | "quantized" | "dyn", "a") => "(i&1!=0).then_some(i as u8)".into(),
                    ("presence" | "quantized" | "dyn", "b") => {
                        "(i&2!=0).then_some(i as u16)".into()
                    }
                    ("presence" | "quantized" | "dyn", "c") => {
                        "(i&4!=0).then_some(number(i,7))".into()
                    }
                    ("domain", "a") => "(number(i,1)&4092)".into(),
                    ("domain", "b") => "[1,3,7,15][i&3]".into(),
                    (_, "stamp" | "suffix") => "i as u64 * 1027".into(),
                    (_, "id") => "i as u32".into(),
                    (_, "flags" | "code") => "i as u8".into(),
                    (_, "zone") => "(i as u16)&4095".into(),
                    ("arrays", "items") => match field.storage {
                        StorageKind::StackArray(_) => "(i%4!=0).then(||::qsl::InlineArray::try_from_iter((0..i%33).map(|item|item as u16 ^ i as u16)).unwrap())".into(),
                        _ => "(i%4!=0).then(||(0..i%33).map(|item|item as u16 ^ i as u16).collect())".into(),
                    },
                    (_, "text") => {
                        let text = match field.storage {
                            StorageKind::StrView => "data",
                            StorageKind::StackString(_) => {
                                "::qsl::StackString::try_from_str(data).unwrap()"
                            }
                            StorageKind::HeapInlineString(_) => {
                                "Box::new(::qsl::InlineString::try_from_str(data).unwrap())"
                            }
                            _ if candidate.fields.contains("decompress") => {
                                "::qsl::InlineString::try_from_str(data).unwrap()"
                            }
                            _ if lowered.lifetime() => "data",
                            _ => "data.to_owned()",
                        };
                        if field.optional {
                            format!("(i%4!=0).then(||{text})")
                        } else {
                            text.into()
                        }
                    }
                    _ => return Ok((source, None)),
                };
                args.push((field.name.clone(), val));
            }
            let construct = if candidate.profile == "named_region" {
                format!("{name}{{Prefix:Prefix{{stamp:i as u64*1027,id:i as u32,zone:(i as u16)&4095,code:i as u8,flags:i as u8}},suffix:i as u64*1027}}")
            } else if source.contains("pub fn try_new(") {
                let names = args
                    .iter()
                    .map(|(_, v)| v.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{name}::try_new({names}).unwrap()")
            } else {
                let names = args
                    .iter()
                    .map(|(field, v)| format!("{field}:{v}"))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{name}{{{names}}}")
            };
            Ok((
                source,
                Some((
                    construct,
                    lowered.lifetime(),
                    prepared.schemas()[0].native_overlay(),
                    fields,
                )),
            ))
        })();
        let mut evidence = json!({"group":candidate.group,"profile":candidate.profile,"schema":schema,
            "stack_limit":candidate.stack,"cpu":format!("{:?}",candidate.cpu),"rt":config.qsl_rt});
        match outcome {
            Ok((source, Some((construct, borrowed, overlay, fields)))) => {
                let route = if overlay {
                    "overlay"
                } else if candidate.group == "dyn" {
                    "dynamic"
                } else if source.contains("QUANT_ACTIVE:bool=true") {
                    "quantized"
                } else if borrowed {
                    "borrowed"
                } else {
                    "owned"
                };
                let extra = if source.contains("pub fn serialize_cached") {
                    "cache"
                } else if config.qsl_rt && source.contains("pub fn serialize(") {
                    "slots"
                } else if overlay {
                    "overlay"
                } else if candidate.profile == "transmute_copy"
                    && source.contains("pub fn deserialize_transmutate_copy")
                {
                    "copy"
                } else {
                    "none"
                };
                let adapters = if overlay {
                    format!("impl {name} {{fn bench_checked(input:&[u8])->&Self{{Self::deserialize_overlay(input).unwrap()}} fn bench_trusted(input:&[u8])->&Self{{unsafe{{Self::deserialize_overlay_unchecked(input)}}}}}}")
                } else if borrowed {
                    format!("impl<'a> {name}<'a>{{fn bench_checked(input:&'a[u8])->Self{{Self::deserialize_checked(input).unwrap()}} fn bench_trusted(input:&'a[u8])->Self{{unsafe{{Self::deserialize_borrowed_from_raw_parts(input.as_ptr(),input.len())}}}}}}")
                } else {
                    String::new()
                };
                writeln!(code, "#[allow(dead_code)] mod c{i}{{ {source}\nuse super::*; {adapters} pub(super) fn add(bench:&mut Runner){{{route}!({name},bench,{:?},{:?},|i:usize,data:&'static str|{{let _=data;{construct}}},{extra});}} }}",candidate.group,candidate.profile).unwrap();
                writeln!(registry, "c{i}::add,").unwrap();
                writeln!(schemas, "{schema}").unwrap();
                evidence["status"] = json!("measured");
                evidence["route"] = json!(route);
                evidence["extra"] = json!(extra);
                evidence["quant_active"] = json!(route == "quantized");
                evidence["emits_pext"] = json!(source.contains("pack4_bmi2"));
                evidence["fields"] = json!(fields);
            }
            Ok(_) => {
                evidence["status"] = json!("compile_only");
                writeln!(schemas, "{schema}").unwrap();
            }
            Err(error) => {
                evidence["status"] = json!("rejected");
                evidence["reason"] = json!(error);
            }
        }
        manifest.push(evidence);
    }
    registry
        .push_str("]; if bench.reverse { cases.reverse() } for case in cases { case(bench); } }\n");
    fs::write(out.join("matrix.rs"), code + &registry).unwrap();
    fs::write(out.join("schemas.qsl"), schemas).unwrap();
    fs::write(
        out.join("profiles.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
