use std::{collections::BTreeMap, fmt::Write as _, fs, path::PathBuf};

const SHARED: &str = "../realistic/src/model.rs";
const BODIES: [&str; 5] = ["Reading", "Update", "Notice", "Heartbeat", "Event"];
const LAYOUTS: [(&str, &str); 5] = [
    ("Auto", "safety,auto"),
    ("Bitmap", "safety,auto,optionals=bitmap"),
    ("Compact", "safety,auto,compact,optionals=bitmap"),
    ("Bounded", "safety,auto,compact,optionals=bitmap"),
    ("Aligned", "safety,auto,compact,alignment,optionals=bitmap"),
];

#[derive(Clone)]
struct Field {
    name: String,
    ty: String,
    optional: bool,
}
type Model = BTreeMap<String, Vec<Field>>;

fn parse_model(source: &str) -> Model {
    let mut result = Model::new();
    let mut active = None::<String>;
    for line in source.lines().map(str::trim) {
        if let Some(line) = line.strip_prefix("pub struct ") {
            let name = line.strip_suffix(" {").expect("struct declaration");
            result.insert(name.to_string(), Vec::new());
            active = Some(name.to_string());
        } else if line == "}" {
            active = None;
        } else if let (Some(name), Some(field)) = (&active, line.strip_prefix("pub ")) {
            let (key, ty) = field.strip_suffix(',').unwrap().split_once(": ").unwrap();
            let option = ty.strip_prefix("Option<").and_then(|p| p.strip_suffix('>'));
            result.get_mut(name).unwrap().push(Field {
                name: key.into(),
                ty: option.unwrap_or(ty).into(),
                optional: option.is_some(),
            });
        }
    }
    result
}

#[derive(Clone)]
struct Leaf {
    name: String,
    ty: String,
    optional: bool,
    input: String,
    marker: bool,
}
fn schema_type(ty: &str) -> &str {
    match ty {
        "u32" => "u32",
        "u64" => "u64",
        "bool" => "u8(0..1)",
        "String" => "str(0..192)",
        "Vec<u32>" => "vec<u32>(0..32)",
        _ => panic!("unknown flat type {ty}"),
    }
}
fn accessor(ty: &str) -> bool {
    matches!(ty, "u32" | "u64" | "bool")
}

// Declared application domains for bounded candidates. The same input records
// are sent to every format. These are schema limits rather than sampled maxima.
fn bounded_type(leaf: &Leaf) -> &str {
    match leaf.name.as_str() {
        "revision" | "device_id" | "body_sequence" => "u32(0..65535)",
        "deadline_ms" | "body_lag_ms" | "body_retry_backoff_ms" => "u32(0..600000)",
        "body_topic"
        | "body_severity"
        | "body_scale"
        | "body_mask"
        | "body_patch_limits_low"
        | "body_retry_attempt"
        | "context_geo_zone" => "u32(0..255)",
        "context_gateway"
        | "context_geo_altitude_m"
        | "body_sensor_id"
        | "body_code"
        | "body_patch_limits_high" => "u32(0..65535)",
        "body_generation" | "body_stats_error_count" => "u32(0..65535)",
        "body_values" | "body_affected" | "body_related" => "vec<u32(0..65535)>(0..32)",
        _ => schema_type(&leaf.ty),
    }
}

fn flatten(
    model: &Model,
    ty: &str,
    prefix: &str,
    access: &str,
    parent_optional: bool,
    leaves: &mut Vec<Leaf>,
) {
    for field in &model[ty] {
        if field.name == "body" {
            continue;
        }
        let key = format!("{prefix}{}", field.name);
        let name = &field.name;
        if model.contains_key(&field.ty) {
            assert!(field.optional);
            let child = if parent_optional {
                format!("{access}.and_then(|p| p.{name}.as_mut())")
            } else {
                format!("{access}.{name}.as_mut()")
            };
            leaves.push(Leaf {
                name: format!("{key}_present"),
                ty: "bool".into(),
                optional: true,
                input: format!("{child}.map(|_| 1)"),
                marker: true,
            });
            flatten(model, &field.ty, &format!("{key}_"), &child, true, leaves);
            continue;
        }
        let input = match (parent_optional, field.optional, accessor(&field.ty)) {
            (false, false, true) => format!("{access}.{name}"),
            (false, false, false) => format!("std::mem::take(&mut {access}.{name})"),
            (false, true, _) => format!("{access}.{name}.take()"),
            (true, false, true) => format!("{access}.map(|p| p.{name})"),
            (true, false, false) => format!("{access}.map(|p| std::mem::take(&mut p.{name}))"),
            (true, true, _) => format!("{access}.and_then(|p| p.{name}.take())"),
        };
        let optional = parent_optional || field.optional;
        let input = if field.ty == "bool" {
            if optional {
                format!("{input}.map(u8::from)")
            } else {
                format!("u8::from({input})")
            }
        } else {
            input
        };
        leaves.push(Leaf {
            name: key,
            ty: field.ty.clone(),
            optional,
            input,
            marker: false,
        });
    }
}

fn rebuild(model: &Model, ty: &str, prefix: &str, parent_optional: bool, body: &str) -> String {
    let mut output = format!("{ty}{{");
    for field in &model[ty] {
        let name = &field.name;
        let key = format!("leaf_{prefix}{name}");
        let value = if name == "body" {
            format!(
                "Some(Body::{body}({}))",
                rebuild(model, body, "body_", false, body)
            )
        } else if model.contains_key(&field.ty) {
            let child = rebuild(model, &field.ty, &format!("{prefix}{name}_"), true, body);
            format!("if {key}_present == Some(1) {{ Some({child}) }} else {{ None }}")
        } else if parent_optional && !field.optional {
            format!("{key}.ok_or(\"missing required nested field\")?")
        } else if field.ty == "bool" && field.optional {
            format!("{key}.map(|flag| flag != 0)")
        } else {
            key
        };
        write!(output, "{name}:{value},").unwrap();
    }
    output.push('}');
    output
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={SHARED}");
    println!("cargo:rerun-if-changed=schemas/settings.deq");
    let model = parse_model(&fs::read_to_string(SHARED)?);
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let mut schema = String::new();
    let mut bindings = String::new();
    let mut variants = String::new();
    let mut create = String::new();
    let mut write_cases = String::new();
    let mut alloc_cases = String::new();
    let mut read_cases = String::new();
    let mut sizes = String::new();
    for (layout, attributes) in LAYOUTS {
        for (i, body) in BODIES.into_iter().enumerate() {
            let tag = i + 1;
            let name = format!("{layout}{body}");
            let mut leaves = Vec::new();
            flatten(&model, "Envelope", "", "value", false, &mut leaves);
            flatten(&model, body, "body_", "body", false, &mut leaves);
            leaves.sort_by_key(|leaf| leaf.optional);
            writeln!(schema, "&{name}[{attributes}] {{").unwrap();
            for leaf in &leaves {
                let spec = if leaf.marker {
                    "u8(1..1)"
                } else {
                    if layout == "Bounded" || layout == "Aligned" {
                        bounded_type(leaf)
                    } else {
                        schema_type(&leaf.ty)
                    }
                };
                let spec = if leaf.optional {
                    if let Some(pos) = spec.find('(') {
                        format!("{}?{}", &spec[..pos], &spec[pos..])
                    } else {
                        format!("{spec}?")
                    }
                } else {
                    spec.into()
                };
                writeln!(schema, "    {spec} {};", leaf.name).unwrap();
            }
            writeln!(schema, "}}").unwrap();
            writeln!(variants, "{name}(generated::{name}),").unwrap();
            let args = leaves
                .iter()
                .map(|f| f.input.as_str())
                .collect::<Vec<_>>()
                .join(",");
            writeln!(create, "(Layout::{layout},Body::{body}(mut body)) => Self::{name}(generated::{name}::try_new({args})?),").unwrap();
            writeln!(write_cases, "Self::{name}(value) => {{ output[0]={tag}; 1 + if checked {{ value.serialize_checked_to(&mut output[1..])? }} else {{ unsafe {{ value.serialize_into_unsafe(output.as_mut_ptr().add(1)) }} }} }},").unwrap();
            writeln!(alloc_cases, "Self::{name}(value) => {{ let capacity=1+value.serialized_size(); if checked {{ let mut out=vec![0;capacity]; out[0]={tag}; let n=value.serialize_checked_to(&mut out[1..])?;out.truncate(1+n);out }} else {{ let mut out:Vec<u8>=Vec::with_capacity(capacity); unsafe {{ out.as_mut_ptr().write({tag}); let n=value.serialize_into_unsafe(out.as_mut_ptr().add(1));out.set_len(1+n); }} out }} }},").unwrap();
            writeln!(read_cases, "(Layout::{layout},{tag}) => rebuild_{name}(if checked {{ generated::{name}::deserialize_checked(bytes)? }} else {{ unsafe {{ generated::{name}::deserialize_from_raw_parts(bytes.as_ptr(),bytes.len()) }} }}),").unwrap();
            writeln!(
                sizes,
                "Self::{name}(_) => std::mem::size_of::<generated::{name}>(),"
            )
            .unwrap();
            writeln!(
                bindings,
                "fn rebuild_{name}(record:generated::{name}) -> Result<Envelope>{{"
            )
            .unwrap();
            for leaf in leaves.iter().filter(|f| f.optional && accessor(&f.ty)) {
                writeln!(bindings, "let leaf_{}=record.{}();", leaf.name, leaf.name).unwrap();
            }
            for leaf in leaves.iter().filter(|f| !(f.optional && accessor(&f.ty))) {
                writeln!(bindings, "let leaf_{}=record.{};", leaf.name, leaf.name).unwrap();
            }
            writeln!(
                bindings,
                "Ok({})\n}}",
                rebuild(&model, "Envelope", "", false, body)
            )
            .unwrap();
        }
    }
    schema.push_str(include_str!("schemas/small.qsl"));
    let schema_path = output.join("flat.qsl");
    fs::write(&schema_path, schema)?;
    qsl::build_qsl_files(
        &[schema_path.to_str().ok_or("non-UTF8 schema path")?],
        &["schemas/settings.deq"],
        &output,
    )?;
    let generated = format!(
        r#"
        use crate::{{model::*, Layout, Result}};
        use qsl::Serializable;
        #[allow(dead_code)]
        mod generated {{ include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs")); }}
        {bindings}
        #[allow(clippy::large_enum_variant)]
        pub(crate) enum Flat {{ {variants} }}
        impl Flat {{
            #[allow(unused_mut)]
            pub(crate) fn prepare(mut value:Envelope, layout:Layout)->Result<Self> {{
                let body=value.body.take().ok_or("missing payload")?;
                Ok(match (layout,body){{{create}}})
            }}
            /// A trusted write requires an immutable previously validated object and sufficient output.
            pub(crate) unsafe fn write(&self, output:&mut [u8], checked:bool)->Result<usize>{{
                Ok(match self {{ {write_cases} }})
            }}
            pub(crate) fn size(&self)->usize {{ match self {{ {sizes} }} }}
            pub(crate) unsafe fn allocate(&self, checked:bool)->Result<Vec<u8>> {{
                Ok(match self {{ {alloc_cases} }})
            }}
        }}
        /// A trusted read requires the exact immutable checked frame for this layout.
        pub(crate) unsafe fn read(input:&[u8], layout:Layout, checked:bool)->Result<Envelope>{{
            let (&tag,bytes)=input.split_first().ok_or("empty QSL frame")?;
            match (layout,tag){{{read_cases} _=>Err("unknown QSL payload tag".into())}}
        }}
    "#
    );
    fs::write(output.join("flat_adapter.rs"), generated)?;
    capnpc::CompilerCommand::new()
        .capnp_executable(std::env::var_os("CAPNP").unwrap_or_else(|| "capnp".into()))
        .src_prefix("schemas")
        .file("schemas/small.capnp")
        .run()?;
    println!("cargo:rerun-if-env-changed=CAPNP");
    println!("cargo:rerun-if-changed=schemas/small.capnp");
    println!("cargo:rerun-if-changed=schemas/small.qsl");
    Ok(())
}
