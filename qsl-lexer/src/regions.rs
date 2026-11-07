//! Structured lowering of native regions. Flat source ranges become named typed fields.
use crate::modernization::{native_type_layout, NativeRegion, NativeTypeLayout};
use crate::{QslField, QslSchema, QslType, FLAG_SAFETY};
use std::collections::{BTreeMap, HashMap, HashSet};
#[derive(Debug)]
pub struct RegionError(pub String);
impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for RegionError {}
fn error(s: impl Into<String>) -> RegionError {
    RegionError(s.into())
}
fn rust_region_name(name: &str) -> bool {
    let mut chars = name.chars();
    let lexical = chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric());
    lexical
        && name != "_"
        && !name.starts_with("__qsl_")
        && ![
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
            "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
            "trait", "true", "type", "unsafe", "use", "where", "while", "abstract", "become",
            "box", "do", "final", "gen", "macro", "override", "priv", "try", "typeof", "unsized",
            "virtual", "yield", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "u128",
            "i128", "usize", "isize", "bool", "char", "str", "f32", "f64",
        ]
        .contains(&name)
}

fn layout(
    s: &QslSchema,
    types: &BTreeMap<String, NativeTypeLayout>,
) -> Result<NativeTypeLayout, RegionError> {
    let (mut native, mut wire, mut align, mut values, mut leaves) =
        (0usize, 0usize, 1usize, 0usize, 0usize);
    for f in &s.fields {
        if f.optional {
            return Err(error("native regions cannot contain optional fields"));
        }
        let l = native_type_layout(&f.data, types).ok_or_else(|| {
            error(format!(
                "{}::{}: native region requires a fixed, registered representation",
                s.name, f.name
            ))
        })?;
        if l.alignment == 0 || !l.alignment.is_power_of_two() {
            return Err(error("invalid native alignment"));
        }
        align = align.max(l.alignment);
        native = native
            .checked_add(l.alignment - 1)
            .map(|n| n & !(l.alignment - 1))
            .and_then(|n| n.checked_add(l.native_bytes))
            .ok_or_else(|| error("native layout overflow"))?;
        wire = wire
            .checked_add(l.wire_bytes)
            .ok_or_else(|| error("wire layout overflow"))?;
        values = values
            .checked_add(l.value_bytes)
            .ok_or_else(|| error("value size overflow"))?;
        leaves = leaves
            .checked_add(l.leaves)
            .ok_or_else(|| error("leaf count overflow"))?;
    }
    if let Some(forced) = s.options.native_alignment {
        if !forced.is_power_of_two() {
            return Err(error("invalid explicit native alignment"));
        }
        align = align.max(forced);
    }
    native = native
        .checked_add(align - 1)
        .map(|n| n & !(align - 1))
        .ok_or_else(|| error("native layout overflow"))?;
    if leaves > 64 {
        return Err(error(
            "native layout supports at most 64 initialized leaves",
        ));
    }
    Ok(NativeTypeLayout {
        is_region: true,
        native_bytes: native,
        alignment: align,
        wire_bytes: wire,
        value_bytes: values,
        leaves,
    })
}
struct Context<'a> {
    names: HashSet<&'a str>,
    types: BTreeMap<String, NativeTypeLayout>,
    defs: Vec<QslSchema>,
    fingerprints: HashMap<String, String>,
    stack: Vec<String>,
}
impl Context<'_> {
    fn fields(
        &mut self,
        source: &QslSchema,
        start: usize,
        end: usize,
        regions: &[NativeRegion],
    ) -> Result<Vec<QslField>, RegionError> {
        let mut result = Vec::new();
        let mut at = start;
        while at < end {
            let next = regions
                .iter()
                .enumerate()
                .filter(|(_, r)| r.start >= at && r.end <= end)
                .min_by_key(|(i, r)| (r.start, usize::MAX - r.end, usize::MAX - i))
                .map(|(_, r)| r);
            let Some(region) = next else {
                result.extend_from_slice(&source.fields[at..end]);
                break;
            };
            result.extend_from_slice(&source.fields[at..region.start]);
            if self.stack.contains(&region.rust_type) {
                return Err(error(format!(
                    "recursive native region {}",
                    region.rust_type
                )));
            }
            if !rust_region_name(&region.rust_type) {
                return Err(error(
                    "native region requires an unreserved Rust identifier",
                ));
            }
            if self.types.contains_key(&region.rust_type)
                && !self.fingerprints.contains_key(&region.rust_type)
            {
                return Err(error("native region conflicts with a registered type"));
            }
            if self.names.contains(region.rust_type.as_str()) {
                return Err(error(format!(
                    "native region type {} conflicts with a declared schema",
                    region.rust_type
                )));
            }
            if self.stack.len() >= 64 {
                return Err(error("native regions exceed maximum nesting depth 64"));
            }
            self.stack.push(region.rust_type.clone());
            let children = regions
                .iter()
                .filter(|r| {
                    !std::ptr::eq(*r, region) && r.start >= region.start && r.end <= region.end
                })
                .cloned()
                .collect::<Vec<_>>();
            let mut fields = self.fields(source, region.start, region.end, &children)?;
            self.stack.pop();
            for (i, f) in fields.iter_mut().enumerate() {
                f.tag = i + 1;
                if region.external {
                    f.name = f
                        .name
                        .strip_prefix(&region.field_prefix)
                        .ok_or_else(|| error("external region prefix mismatch"))?
                        .to_owned();
                }
            }
            let mut child = QslSchema {
                name: region.rust_type.clone(),
                max_tag: fields.len(),
                fields,
                flags: FLAG_SAFETY,
                ..Default::default()
            };
            child.options.transmutate = true;
            child.options.generated_region = true;
            child.options.external_definition = region.external;
            child.options.native_alignment = region.alignment;
            let fingerprint = format!(
                "{:?}|{}|{:?}",
                child.fields, child.options.external_definition, child.options.native_alignment
            );
            if let Some(old) = self.fingerprints.get(&child.name) {
                if old != &fingerprint {
                    return Err(error(format!(
                        "conflicting native region definitions named {}",
                        child.name
                    )));
                }
            } else {
                let l = layout(&child, &self.types)?;
                self.types.insert(child.name.clone(), l);
                self.fingerprints.insert(child.name.clone(), fingerprint);
                self.defs.push(child.clone());
            }
            let l = &self.types[&child.name];
            result.push(QslField {
                name: if region.external {
                    region
                        .field_prefix
                        .strip_suffix('_')
                        .unwrap_or(&region.field_prefix)
                        .to_owned()
                } else {
                    region.rust_type.clone()
                },
                tag: source.fields[region.start].tag,
                data: QslType::Custom {
                    rust_type: child.name,
                    fixed_wire_size: Some(l.wire_bytes),
                },
                ..Default::default()
            });
            at = region.end;
        }
        Ok(result)
    }
}
pub fn lower_regions(schemas: &[QslSchema]) -> Result<Vec<QslSchema>, RegionError> {
    let mut context = Context {
        names: schemas
            .iter()
            .filter(|s| !s.options.generated_region)
            .map(|s| s.name.as_str())
            .collect(),
        types: schemas
            .iter()
            .flat_map(|s| s.options.native_types.clone())
            .collect(),
        defs: Vec::new(),
        fingerprints: schemas
            .iter()
            .filter(|s| s.options.generated_region)
            .map(|s| {
                (
                    s.name.clone(),
                    format!(
                        "{:?}|{}|{:?}",
                        s.fields, s.options.external_definition, s.options.native_alignment
                    ),
                )
            })
            .collect(),
        stack: Vec::new(),
    };
    let mut parents = Vec::new();
    for source in schemas {
        if source.options.regions.is_empty() {
            parents.push(source.clone());
            continue;
        }
        if source.compact() || source.qps() {
            return Err(error("native regions require ordinary byte encoding"));
        }
        for r in &source.options.regions {
            if r.start >= r.end || r.end > source.fields.len() {
                return Err(error("native region outside schema"));
            }
        }
        for (a, i) in source.options.regions.iter().enumerate() {
            for j in &source.options.regions[a + 1..] {
                if i.start < j.start && j.start < i.end && i.end < j.end
                    || j.start < i.start && i.start < j.end && j.end < i.end
                {
                    return Err(error("crossing native regions are invalid"));
                }
            }
        }
        let mut parent = source.clone();
        parent.fields = context.fields(source, 0, source.fields.len(), &source.options.regions)?;
        parent.options.regions.clear();
        parents.push(parent);
    }
    context.defs.extend(parents);
    for schema in &mut context.defs {
        schema.options.native_types.extend(context.types.clone());
    }
    Ok(context.defs)
}

#[cfg(test)]
mod tests {
    use crate::{parse_qsl, QslType};
    #[test]
    fn explicit_regions_remain_nested_and_idempotent() {
        let source="&P{#region(aggressive_transmutate=\"Outer\");#region(aggressive_transmutate=\"Inner\");u64 id;#end_region;#end_region;u8 tail;}";
        let schemas = parse_qsl(source).unwrap();
        let parent = schemas.iter().find(|s| s.name == "P").unwrap();
        assert_eq!(parent.fields.len(), 2);
        assert_eq!(parent.fields[0].name, "Outer");
        let outer = schemas.iter().find(|s| s.name == "Outer").unwrap();
        assert_eq!(outer.fields[0].name, "Inner");
        assert!(matches!(outer.fields[0].data, QslType::Custom { .. }));
        assert_eq!(super::lower_regions(&schemas).unwrap().len(), schemas.len());
    }
    #[test]
    fn incompatible_recursive_and_conflicting_regions_are_rejected() {
        for source in [
   "&P{#region(aggressive_transmutate=\"type\");u64 x;#end_region;}",
   "&P{#region(aggressive_transmutate=\"R\");str text;#end_region;}",
   "&P{#region(aggressive_transmutate=\"R\");u8? x;#end_region;}",
   "&R{u64 x;}&P{#region(aggressive_transmutate=\"R\");u64 x;#end_region;}",
   "&P{#region(aggressive_transmutate=\"R\");#region(aggressive_transmutate=\"R\");u64 x;#end_region;#end_region;}",
   "&P{#region(aggressive_transmutate=\"R\");u64 x;#end_region;}&Q{#region(aggressive_transmutate=\"R\");u32 y;#end_region;}"
  ]{assert!(parse_qsl(source).is_err(),"{source}");}
    }
}
