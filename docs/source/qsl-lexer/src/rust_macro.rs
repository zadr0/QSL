//! Explicit Rust-source imports for schema field expansion. Syn parses real
//! Rust syntax, so comments, raw identifiers and nested modules are not regexes.
use crate::{
    schema_macro::MacroError, QslField, QslSchema, QslType, FLAG_FIELD_BORROW, FLAG_MACRO,
};
use std::collections::{HashMap, HashSet};
use syn::{Fields, GenericArgument, GenericParam, Item, ItemStruct, PathArguments, Type};

fn marked(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "qsl_macro" || s.ident == "schem_macro")
    })
}
fn declarations(items: Vec<Item>, out: &mut Vec<ItemStruct>) -> Result<(), MacroError> {
    for item in items {
        match item {
            Item::Struct(s) if marked(&s.attrs) => out.push(s),
            Item::Mod(m) => {
                if let Some((_, items)) = m.content {
                    declarations(items, out)?;
                }
            }
            Item::Enum(e) if marked(&e.attrs) => {
                return Err(MacroError(
                    "Rust macro markers require a struct, not enum".into(),
                ))
            }
            _ => {}
        }
    }
    Ok(())
}
fn integer(expr: &syn::Expr) -> Result<usize, MacroError> {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(n),
        ..
    }) = expr
    {
        return n
            .base10_parse()
            .map_err(|_| MacroError("array/inline capacity must fit usize".into()));
    }
    Err(MacroError(
        "Rust macro arrays/inline strings require a literal capacity".into(),
    ))
}
/// Source imports can evaluate literal capacities and sealed built-in length
/// words. Generic aliases require typed Builder reflection instead of guessing.
fn bounded_capacity(
    capacity: &GenericArgument,
    length: Option<&GenericArgument>,
) -> Result<usize, MacroError> {
    let GenericArgument::Const(expr) = capacity else {
        return Err(MacroError(
            "bounded capacity must be an integer literal; use typed Builder for generic constants"
                .into(),
        ));
    };
    let n = integer(expr)?;
    let maximum = match length {
        None => u32::MAX as usize,
        Some(GenericArgument::Type(Type::Path(p))) => match p
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .as_deref()
        {
            Some("u32") => u32::MAX as usize,
            Some("NonZeroU8") => u8::MAX as usize - 1,
            Some("NonZeroU16") => u16::MAX as usize - 1,
            Some("NonZeroU32") => u32::MAX as usize - 1,
            _ => {
                return Err(MacroError(
                    "unknown bounded length word; use typed Builder for type aliases".into(),
                ))
            }
        },
        _ => {
            return Err(MacroError(
                "bounded length word must be u32 or NonZeroU8/U16/U32".into(),
            ))
        }
    };
    if n > maximum {
        return Err(MacroError(format!(
            "capacity {n} exceeds the selected length word's maximum {maximum}"
        )));
    }
    Ok(n)
}
fn field_type(
    ty: &Type,
    names: &HashSet<String>,
    optional: &mut bool,
    borrow: &mut bool,
    borrow_str: &mut bool,
    owned_str: &mut bool,
) -> Result<QslType, MacroError> {
    match ty {
        Type::Path(p) if p.qself.is_none() => {
            let leaf = p
                .path
                .segments
                .last()
                .ok_or_else(|| MacroError("empty Rust type path".into()))?;
            let name = leaf.ident.to_string();
            if name == "Option" {
                if *optional {
                    return Err(MacroError(
                        "nested Option cannot represent one optional field".into(),
                    ));
                }
                let PathArguments::AngleBracketed(args) = &leaf.arguments else {
                    return Err(MacroError("Option requires one type".into()));
                };
                let mut types = args.args.iter().filter_map(|a| {
                    if let GenericArgument::Type(t) = a {
                        Some(t)
                    } else {
                        None
                    }
                });
                let inner = types
                    .next()
                    .ok_or_else(|| MacroError("Option requires one type".into()))?;
                if types.next().is_some() {
                    return Err(MacroError("Option requires one type".into()));
                }
                *optional = true;
                return field_type(inner, names, optional, borrow, borrow_str, owned_str);
            }
            if matches!(name.as_str(), "Utf8Guard" | "Utf8View" | "Utf8CopyGuard") {
                return Err(MacroError(
                    "Guard strings are unsupported; use String, &str, StackString or InlineString"
                        .into(),
                ));
            }
            if name == "StackString" {
                let PathArguments::AngleBracketed(args) = &leaf.arguments else {
                    return Err(MacroError("StackString requires a literal capacity".into()));
                };
                if args.args.is_empty() || args.args.len() > 2 {
                    return Err(MacroError(
                        "StackString expects capacity and an optional length word".into(),
                    ));
                }
                let max = bounded_capacity(&args.args[0], args.args.get(1))?;
                return Ok(QslType::RangedString { min: 0, max });
            }
            if name == "InlineArray" {
                let PathArguments::AngleBracketed(args) = &leaf.arguments else {
                    return Err(MacroError(
                        "InlineArray requires element and literal capacity".into(),
                    ));
                };
                if !(2..=3).contains(&args.args.len()) {
                    return Err(MacroError(
                        "InlineArray expects element, capacity and an optional length word".into(),
                    ));
                }
                let GenericArgument::Type(element) = &args.args[0] else {
                    return Err(MacroError("InlineArray requires an element type".into()));
                };
                let max = bounded_capacity(&args.args[1], args.args.get(2))?;
                let (mut element_optional, mut element_borrow, mut element_str, mut element_owned) =
                    (false, false, false, false);
                let element = field_type(
                    element,
                    names,
                    &mut element_optional,
                    &mut element_borrow,
                    &mut element_str,
                    &mut element_owned,
                )?;
                if element_optional
                    || element_borrow
                    || element.primitive().ok().flatten().is_none()
                {
                    return Err(MacroError("source InlineArray imports require numeric elements; use typed Builder for other supported element types".into()));
                }
                // This newly supported container uses the typed Builder's byte
                // codec. Do not rewrite legacy source [u8;N] array semantics.
                let element = if matches!(
                    element,
                    QslType::Integer {
                        bits: 8,
                        signed: false,
                        min: 0
                    }
                ) {
                    QslType::Byte
                } else {
                    element
                };
                return Ok(QslType::RangedArray {
                    element: Box::new(element),
                    min: 0,
                    max,
                });
            }
            if name == "Box" {
                let PathArguments::AngleBracketed(args) = &leaf.arguments else {
                    return Err(MacroError("Box requires one type".into()));
                };
                if args.args.len() != 1 {
                    return Err(MacroError("Box requires one type".into()));
                }
                let GenericArgument::Type(inner) = &args.args[0] else {
                    return Err(MacroError("Box requires one type".into()));
                };
                let mut nested_optional = false;
                let data = field_type(
                    inner,
                    names,
                    &mut nested_optional,
                    borrow,
                    borrow_str,
                    owned_str,
                )?;
                if nested_optional || matches!(data, QslType::Custom { .. }) {
                    return Err(MacroError(
                        "source Box imports require a supported non-optional field type".into(),
                    ));
                }
                return Ok(data);
            }
            if name == "InlineString" {
                let PathArguments::AngleBracketed(args) = &leaf.arguments else {
                    return Err(MacroError(
                        "InlineString requires a literal capacity".into(),
                    ));
                };
                if args.args.is_empty() || args.args.len() > 2 {
                    return Err(MacroError(
                        "InlineString expects capacity and an optional length word".into(),
                    ));
                }
                let max = bounded_capacity(&args.args[0], args.args.get(1))?;
                if max > u16::MAX as usize {
                    return Err(MacroError(
                        "InlineString capacity exceeds the QSL-LZ limit".into(),
                    ));
                }
                return Ok(QslType::CompressedString { min: 0, max });
            }
            if names.contains(&name) {
                let lifetimes_only = match &leaf.arguments {
                    PathArguments::None => true,
                    PathArguments::AngleBracketed(a) => a
                        .args
                        .iter()
                        .all(|x| matches!(x, GenericArgument::Lifetime(_))),
                    _ => false,
                };
                if !lifetimes_only {
                    return Err(MacroError(format!(
                        "macro reference `{name}` only accepts lifetime arguments"
                    )));
                }
                return Ok(QslType::Custom {
                    rust_type: name,
                    fixed_wire_size: None,
                });
            }
            if !matches!(leaf.arguments, PathArguments::None) {
                return Err(MacroError(format!(
                    "generic Rust type `{name}` cannot be flattened"
                )));
            }
            if name == "String" {
                *owned_str = true;
                return Ok(QslType::String(0));
            }
            if name == "u128" {
                return Ok(QslType::DomainInteger(
                    crate::domain::IntegerDomain::unsigned(128, 0, u128::MAX)
                        .expect("full unsigned domain")
                        .into(),
                ));
            }
            if name == "i128" {
                return Ok(QslType::DomainInteger(
                    crate::domain::IntegerDomain::signed(128, i128::MIN, i128::MAX)
                        .expect("full signed domain")
                        .into(),
                ));
            }
            if let Some(bits) = name
                .get(1..)
                .and_then(|n| n.parse::<u8>().ok())
                .filter(|b| [8, 16, 32, 64].contains(b))
            {
                if name.starts_with('u') || name.starts_with('i') {
                    return Ok(QslType::Integer {
                        bits,
                        signed: name.starts_with('i'),
                        min: 0,
                    });
                }
            }
            if [
                "u128", "i128", "usize", "isize", "f32", "f64", "bool", "char",
            ]
            .contains(&name.as_str())
            {
                return Err(MacroError(format!("Rust primitive `{name}` has no matching QSL field codec; use an explicit supported wrapper")));
            }
            let rust_type = if names.contains(&name) {
                name
            } else {
                p.path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            };
            Ok(QslType::Custom {
                rust_type,
                fixed_wire_size: None,
            })
        }
        Type::Reference(r) if r.mutability.is_none() => match r.elem.as_ref() {
            Type::Path(p) if p.path.is_ident("str") => {
                *borrow = true;
                *borrow_str = true;
                Ok(QslType::BorrowedString { min: 0, max: None })
            }
            Type::Slice(_) => Err(MacroError(
                "byte slices are not UTF-8 string representations; use &str for text".into(),
            )),
            _ => Err(MacroError(
                "only shared str and [u8] references are supported in Rust schema macros".into(),
            )),
        },
        Type::Array(a) => {
            let mut element_optional = false;
            let mut element_borrow = false;
            let mut element_str = false;
            let mut element_owned = false;
            let element = field_type(
                &a.elem,
                names,
                &mut element_optional,
                &mut element_borrow,
                &mut element_str,
                &mut element_owned,
            )?;
            if element_optional
                || element_borrow
                || !matches!(
                    element,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::Flags(_)
                )
            {
                return Err(MacroError(
                    "Rust schema arrays require fixed primitive numeric elements".into(),
                ));
            }
            Ok(QslType::FixedArray(Box::new(element), integer(&a.len)?))
        }
        _ => Err(MacroError(
            "unsupported Rust field type in schema macro".into(),
        )),
    }
}
pub fn parse_rust_macros(sources: &[&str]) -> Result<Vec<QslSchema>, MacroError> {
    let mut defs = Vec::new();
    for source in sources {
        let file = syn::parse_file(source).map_err(|e| MacroError(format!("Rust source: {e}")))?;
        declarations(file.items, &mut defs)?;
    }
    let mut names = HashSet::new();
    for d in &defs {
        if !names.insert(d.ident.to_string()) {
            return Err(MacroError(format!(
                "duplicate Rust schema macro `{}`",
                d.ident
            )));
        }
    }
    let mut result = Vec::new();
    for def in defs {
        let mut native = false;
        let mut native_alignment: Option<usize> = None;
        for attr in def.attrs.iter().filter(|a| marked(std::slice::from_ref(a))) {
            if !matches!(attr.meta, syn::Meta::Path(_)) {
                let mode = attr
                    .parse_args::<syn::Ident>()
                    .map_err(|_| MacroError("expected aggressive_transmutate".into()))?;
                if mode != "aggressive_transmutate" {
                    return Err(MacroError("expected aggressive_transmutate".into()));
                }
                native = true;
            }
        }
        if native {
            for attr in def.attrs.iter().filter(|a| a.path().is_ident("repr")) {
                attr.parse_nested_meta(|meta|{if meta.path.is_ident("C"){Ok(())}else if meta.path.is_ident("align"){let content;syn::parenthesized!(content in meta.input);let value:syn::LitInt=content.parse()?;let n=value.base10_parse::<usize>()?;if !n.is_power_of_two(){return Err(meta.error("native alignment must be a power of two"))}native_alignment=Some(native_alignment.unwrap_or(1).max(n));Ok(())}else{Err(meta.error("aggressive_transmutate requires repr(C), optionally with align(N); packed/Rust/transparent are incompatible"))}}).map_err(|e|MacroError(format!("{}: {e}",def.ident)))?;
            }
        }
        if def
            .attrs
            .iter()
            .any(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        {
            return Err(MacroError(format!(
                "{}: conditional Rust macro declarations require an explicit resolved source input",
                def.ident
            )));
        }
        if def
            .generics
            .params
            .iter()
            .any(|p| !matches!(p, GenericParam::Lifetime(_)))
        {
            return Err(MacroError(format!(
                "{}: only lifetime parameters are supported",
                def.ident
            )));
        }
        let Fields::Named(fields) = def.fields else {
            return Err(MacroError(format!(
                "{}: schema macros require named fields",
                def.ident
            )));
        };
        let mut schema = QslSchema {
            name: def.ident.to_string(),
            flags: FLAG_MACRO,
            ..Default::default()
        };
        let (mut borrow, mut borrowed_str, mut owned_str) = (false, false, false);
        for f in fields.named {
            if f.attrs
                .iter()
                .any(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
            {
                return Err(MacroError(format!(
                    "{}: conditional macro fields are not supported",
                    schema.name
                )));
            }
            if native
                && matches!(&f.ty,Type::Path(p) if p.path.segments.last().is_some_and(|s|s.ident=="Box"))
            {
                return Err(MacroError(
                    "native regions cannot contain Box ownership descriptors".into(),
                ));
            }
            let name = f.ident.expect("named field").to_string();
            let mut optional = false;
            let mut data = field_type(
                &f.ty,
                &names,
                &mut optional,
                &mut borrow,
                &mut borrowed_str,
                &mut owned_str,
            )
            .map_err(|e| MacroError(format!("{}::{name}: {e}", schema.name)))?;
            if native {
                if let QslType::InlineRawString { max, .. }
                | QslType::CompressedString { max, .. } = data
                {
                    data = QslType::FixedString(max);
                }
            }
            let bits = data.default_compact_bits()?;
            schema.fields.push(QslField {
                shifted_length: false,
                storage: Default::default(),
                name,
                optional,
                bits,
                tag: schema.fields.len() + 1,
                data,
                default: None,
            });
        }
        if borrow {
            schema.flags |= FLAG_FIELD_BORROW;
        }
        if native {
            schema.options.regions.push(crate::layout::NativeRegion {
                alignment: native_alignment,
                external: true,
                field_prefix: String::new(),
                rust_type: schema.name.clone(),
                start: 0,
                end: schema.fields.len(),
            });
        }
        schema.max_tag = schema.fields.len();
        result.push(schema);
    }
    Ok(result)
}
impl From<crate::QslError> for MacroError {
    fn from(e: crate::QslError) -> Self {
        Self(e.to_string())
    }
}

/// Imported Rust declarations supply fields but are not emitted as duplicate
/// Rust types. Their original source declarations remain compiled normally.
pub fn expand_with_rust(
    schemas: &[QslSchema],
    sources: &[&str],
) -> Result<Vec<QslSchema>, MacroError> {
    let mut all = schemas.to_vec();
    let imported = parse_rust_macros(sources)?;
    let external_names = imported
        .iter()
        .map(|s| s.name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    // Already lowered native schemas keep the explicit QSL field contract.
    // The imported Rust name supplies the definition, not a second schema.
    all.extend(imported.into_iter().filter(|m| {
        !schemas
            .iter()
            .any(|s| s.options.generated_region && s.name == m.name)
    }));
    let mut expanded = crate::schema_macro::expand_macros(&all)?;
    expanded.truncate(schemas.len());
    for schema in &mut expanded {
        schema
            .options
            .externally_defined_types
            .extend(external_names.iter().cloned());
        if schema.options.generated_region && external_names.contains(&schema.name) {
            schema.options.external_definition = true;
        }
    }
    Ok(expanded)
}
pub fn parse_qsl_with_rust(
    source: &str,
    constants: &HashMap<String, i128>,
    rust_sources: &[&str],
) -> Result<Vec<QslSchema>, MacroError> {
    let schemas = crate::parser::parse_qsl_raw_with_constants(source, constants)
        .map_err(|e| MacroError(e.to_string()))?;
    expand_with_rust(&schemas, rust_sources)
}

pub fn parse_qsl_with_rust_wide(
    source: &str,
    constants: &HashMap<String, crate::WideNumber>,
    rust_sources: &[&str],
) -> Result<Vec<QslSchema>, MacroError> {
    let schemas =
        crate::parser::parse_qsl_raw_with_wide_primitives(source, constants, &Default::default())
            .map_err(|e| MacroError(e.to_string()))?;
    expand_with_rust(&schemas, rust_sources)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(qsl: &str, rust: &str) -> Result<Vec<QslSchema>, MacroError> {
        parse_qsl_with_rust(qsl, &HashMap::new(), &[rust])
    }
    #[test]
    fn rust_markers_expand_nested_fields_without_duplicate_types() {
        let defs="#[qsl::qsl_macro] pub struct Meta {pub base:Base,pub code:u16} #[qsl::schem_macro] pub struct Base {pub id:u64}";
        let s = parse("&Packet[safety]{Meta first;Meta second;u8? extra;}", defs).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(
            s[0].fields
                .iter()
                .map(|f| (f.name.as_str(), f.tag))
                .collect::<Vec<_>>(),
            [
                ("first_base_id", 1),
                ("first_code", 2),
                ("second_base_id", 3),
                ("second_code", 4),
                ("extra", 5)
            ]
        );
        assert!(crate::parse_qsl("&Meta[macro]{u64 id;}").is_err());
    }
    #[test]
    fn borrowed_rust_fields_keep_ownership_and_lifetime() {
        let defs="#[qsl_macro] struct Base<'a>{text:&'a str} #[schem_macro] struct Meta<'a>{base:Base<'a>}";
        let s = parse("&P{Meta meta;}", defs).unwrap();
        assert!(s[0].lifetime());
        assert!(!s[0].borrow_strings());
        assert_eq!(s[0].fields[0].data.rust_type(false).unwrap(), "&'a str");
        let mixed = parse("&P{str owned;Meta meta;}", defs).unwrap();
        assert_eq!(mixed[0].fields[0].data.rust_type(false).unwrap(), "String");
        assert_eq!(mixed[0].fields[1].data.rust_type(false).unwrap(), "&'a str");
        let s = parse(
            "&P{Meta meta;}",
            "#[qsl_macro] struct Meta<'a>{text:&'a str,owned:String}",
        )
        .unwrap();
        assert!(s[0].lifetime());
        assert!(!s[0].borrow_strings());
        assert!(matches!(s[0].fields[1].data, QslType::String(_)));
    }
    #[test]
    fn shared_rust_dag_has_bounded_expansion_work() {
        let mut source = String::from("#[qsl_macro] struct M0{}");
        for i in 1..32 {
            source.push_str(&format!(
                "#[qsl_macro] struct M{i}{{a:M{},b:M{}}}",
                i - 1,
                i - 1
            ));
        }
        let s = parse("&P{M31 empty;}", &source).unwrap();
        assert!(s[0].fields.is_empty());
    }
    #[test]
    fn cycles_collisions_group_presence_and_field_order_are_errors() {
        for (q, r) in [
            ("&P{A a;}", "#[qsl_macro]struct A{a:A}"),
            (
                "&P{A a;}",
                "#[qsl_macro]struct A{b:B}#[qsl_macro]struct B{a:A}",
            ),
            ("&P{M m;u8 m_id;}", "#[qsl_macro]struct M{id:u8}"),
            ("&P{M? m;}", "#[qsl_macro]struct M{id:u8}"),
            ("&P{M m;u8 required;}", "#[qsl_macro]struct M{x:Option<u8>}"),
        ] {
            assert!(parse(q, r).is_err(), "{q}: {r}");
        }
    }
    #[test]
    fn source_macro_import_preserves_full_width_integer_metadata() {
        let s = parse("&P{M m;}", "#[qsl_macro]struct M{a:u128,b:i128}").unwrap();
        assert_eq!(s[0].fields[0].data.primitive().unwrap(), Some("u128"));
        assert_eq!(s[0].fields[1].data.primitive().unwrap(), Some("i128"));
        assert_eq!(s[0].fields[0].data.fixed_wire_size().unwrap(), Some(16));
    }
    #[test]
    fn source_constraints_are_explicit() {
        for r in [
            "#[qsl_macro]enum M{A}",
            "#[qsl_macro]struct M(u8);",
            "#[qsl_macro]struct M<T>{value:T}",
            "#[qsl_macro]#[cfg(unix)]struct M{id:u8}",
            "#[qsl_macro]struct M{x:Option<Option<u8>>}",
            "#[qsl_macro]struct M{bytes:[u8;SIZE]}",
        ] {
            assert!(parse("&P{M m;}", r).is_err(), "{r}");
        }
    }
    #[test]
    fn source_bounded_containers_preserve_numeric_domains_and_reject_invalid_length_words() {
        let s=parse("&P[safety,aggressive[shift=true]]{M m;}","#[qsl_macro]struct M{name:qsl::StackString<32,core::num::NonZeroU8>,values:qsl::InlineArray<u16,8,core::num::NonZeroU8>,extra:Option<qsl::InlineArray<u8,4>>}").unwrap();
        assert!(matches!(
            s[0].fields[0].data,
            QslType::RangedString { min: 0, max: 32 }
        ));
        assert!(matches!(
            s[0].fields[1].data,
            QslType::RangedArray { min: 0, max: 8, .. }
        ));
        assert!(s[0].fields[2].optional);
        for ty in [
            "StackString<255,NonZeroU8>",
            "StackString<8,u8>",
            "InlineArray<u16,255,NonZeroU8>",
            "InlineArray<Option<u8>,8>",
            "InlineArray<u8,8,UnknownLength>",
            "InlineArray<u8,8,u32,u32>",
        ] {
            let rust = format!("#[qsl_macro]struct M{{value:{ty}}}");
            assert!(parse("&P{M m;}", &rust).is_err(), "{ty}");
        }
    }
    #[test]
    fn inline_string_is_lz_while_explicit_native_rewrite_remains_fixed_utf8() {
        let normal = parse("&P{M meta;}", "#[qsl_macro]struct M{text:InlineString<64>}").unwrap();
        let expected = crate::parse_qsl("&P{str(0..64) meta_text|decompress=true|;}").unwrap();
        assert!(matches!(
            normal[0].fields[0].data,
            QslType::CompressedString { min: 0, max: 64 }
        ));
        for algorithm in [
            crate::SchemaHashAlgorithm::Xxh3_64,
            crate::SchemaHashAlgorithm::Xxh3_128,
        ] {
            assert_eq!(
                normal[0].hashes(algorithm).unwrap(),
                expected[0].hashes(algorithm).unwrap()
            );
            let mut old = normal[0].clone();
            old.fields[0].data = QslType::InlineRawString { min: 0, max: 64 };
            assert_ne!(
                normal[0].hashes(algorithm).unwrap(),
                old.hashes(algorithm).unwrap()
            );
        }
        let native = parse(
            "&P[safety]{M meta;}",
            "#[qsl_macro(aggressive_transmutate)]struct M{text:InlineString<4>}",
        )
        .unwrap();
        assert!(native.iter().any(|s| s
            .fields
            .iter()
            .any(|f| matches!(f.data, QslType::FixedString(4)))));
        assert!(parse(
            "&P[safety]{M meta;}",
            "#[qsl_macro(aggressive_transmutate)]struct M{text:Box<InlineString<4>>}"
        )
        .is_err());
    }
    #[test]
    fn explicit_native_region_reuses_a_supplied_rust_declaration_in_either_preparation_order() {
        let qsl="&P[safety]{#region(aggressive_transmutate=\"ArrayMeta\");array<u16>(3) values;#end_region;u8 tail;}";
        let rust = "#[schem_macro(aggressive_transmutate)]struct ArrayMeta{values:[u16;3]}";
        let raw = crate::parser::parse_qsl_raw_with_constants(qsl, &Default::default()).unwrap();
        let parsed = crate::parse_qsl(qsl).unwrap();
        for schemas in [raw, parsed] {
            let imported = expand_with_rust(&schemas, &[rust]).unwrap();
            let lowered = crate::frontend::expand_schemas(&imported).unwrap();
            let definitions = lowered
                .iter()
                .filter(|s| s.name == "ArrayMeta")
                .collect::<Vec<_>>();
            assert_eq!(definitions.len(), 1);
            assert!(definitions[0].options.external_definition);
            let repeated = crate::frontend::expand_schemas(&lowered).unwrap();
            assert!(
                repeated
                    .iter()
                    .find(|s| s.name == "ArrayMeta")
                    .unwrap()
                    .options
                    .external_definition
            );
        }
    }
}
