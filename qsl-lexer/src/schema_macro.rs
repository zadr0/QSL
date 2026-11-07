//! Build-time field expansion. Macro schemas still produce standalone Rust
//! structures; references to them contribute fields, not nested Rust members.
use crate::{QslField, QslSchema, QslType, FLAG_FIELD_BORROW, FLAG_MACRO};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MacroError(pub String);
impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for MacroError {}

pub fn expand_macros(schemas: &[QslSchema]) -> Result<Vec<QslSchema>, MacroError> {
    let mut names = HashMap::new();
    for (i, s) in schemas.iter().enumerate() {
        if names.insert(s.name.as_str(), i).is_some() {
            return Err(MacroError(format!("duplicate schema `{}`", s.name)));
        }
    }
    fn fields(
        index: usize,
        schemas: &[QslSchema],
        names: &HashMap<&str, usize>,
        stack: &mut Vec<usize>,
        cache: &mut HashMap<usize, (Vec<QslField>, bool, usize)>,
    ) -> Result<(Vec<QslField>, bool, usize), MacroError> {
        if let Some(result) = cache.get(&index) {
            return Ok(result.clone());
        }
        if stack.contains(&index) {
            return Err(MacroError(format!(
                "cyclic structure-macro reference: {} -> {}",
                stack
                    .iter()
                    .map(|&i| schemas[i].name.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> "),
                schemas[index].name
            )));
        }
        if stack.len() >= 64 {
            return Err(MacroError("structure-macro nesting exceeds 64".into()));
        }
        stack.push(index);
        let schema = &schemas[index];
        let mut result = Vec::new();
        let mut expanded = false;
        let mut height = 1usize;
        for f in &schema.fields {
            let target = if let QslType::Custom { rust_type, .. } = &f.data {
                names
                    .get(rust_type.as_str())
                    .copied()
                    .filter(|&i| schemas[i].flags & FLAG_MACRO != 0)
            } else {
                None
            };
            if let Some(target) = target {
                if f.optional || f.default.is_some() {
                    return Err(MacroError(format!("{}::{}: optional/default on a macro reference is ambiguous; declare optionality/defaults on its fields",schema.name,f.name)));
                }
                let definition = &schemas[target];
                let (children, _, depth) = fields(target, schemas, names, stack, cache)?;
                if definition.borrow_strings() != schema.borrow_strings()
                    && children.iter().any(|f| {
                        matches!(
                            f.data,
                            QslType::String(_)
                                | QslType::RangedString { .. }
                                | QslType::Array(_, _)
                                | QslType::RangedArray { .. }
                                | QslType::FixedArray(_, _)
                        )
                    })
                {
                    return Err(MacroError(format!("{}::{}: macro-level lifetime differs; use matching lifetime flags or explicit per-field binary_opt",schema.name,f.name)));
                }
                height = height.max(depth + 1);
                if height > 64 {
                    return Err(MacroError("structure-macro nesting exceeds 64".into()));
                }
                for mut child in children {
                    child.name = format!("{}_{}", f.name, child.name);
                    result.push(child);
                }
                expanded = true;
            } else {
                result.push(f.clone());
            }
            if result.len() > if schema.bitmap() { 65535 } else { 64 } {
                return Err(MacroError(format!(
                    "{}: expanded schema exceeds the codec field capacity",
                    schema.name
                )));
            }
        }
        stack.pop();
        cache.insert(index, (result.clone(), expanded, height));
        Ok((result, expanded, height))
    }
    fn regions(
        index: usize,
        schemas: &[QslSchema],
        names: &HashMap<&str, usize>,
        cache: &HashMap<usize, (Vec<QslField>, bool, usize)>,
        memo: &mut HashMap<usize, Vec<crate::modernization::NativeRegion>>,
    ) -> Result<Vec<crate::modernization::NativeRegion>, MacroError> {
        if let Some(r) = memo.get(&index) {
            return Ok(r.clone());
        }
        let source = &schemas[index];
        let mut positions = vec![0usize];
        let mut children = Vec::new();
        for f in &source.fields {
            let target = if let QslType::Custom { rust_type, .. } = &f.data {
                names
                    .get(rust_type.as_str())
                    .copied()
                    .filter(|&i| schemas[i].flags & FLAG_MACRO != 0)
            } else {
                None
            };
            let count = target.map_or(1, |i| cache[&i].0.len());
            children.push(target);
            positions.push(positions.last().unwrap() + count);
        }
        let mut out = Vec::new();
        for r in &source.options.regions {
            let start = *positions
                .get(r.start)
                .ok_or_else(|| MacroError("native region start outside schema".into()))?;
            let end = *positions
                .get(r.end)
                .ok_or_else(|| MacroError("native region end outside schema".into()))?;
            out.push(crate::modernization::NativeRegion {
                alignment: r.alignment,
                external: r.external,
                field_prefix: r.field_prefix.clone(),
                rust_type: r.rust_type.clone(),
                start,
                end,
            });
        }
        for (i, target) in children.into_iter().enumerate() {
            if let Some(target) = target {
                for mut r in regions(target, schemas, names, cache, memo)? {
                    if r.external {
                        r.field_prefix = format!("{}_{}", source.fields[i].name, r.field_prefix);
                    }
                    r.start += positions[i];
                    r.end += positions[i];
                    out.push(r);
                }
            }
        }
        memo.insert(index, out.clone());
        Ok(out)
    }
    let mut result = Vec::new();
    let mut cache = HashMap::new();
    let mut region_cache = HashMap::new();
    for (i, source) in schemas.iter().enumerate() {
        let (mut fields, expanded, _) = fields(i, schemas, &names, &mut Vec::new(), &mut cache)?;
        if expanded {
            for (i, f) in fields.iter_mut().enumerate() {
                f.tag = i + 1;
            }
        }
        let mut schema = source.clone();
        schema.fields = fields;
        schema.options.regions = regions(i, schemas, &names, &cache, &mut region_cache)?;
        if expanded {
            schema.max_tag = schema.fields.len();
        }
        if schema
            .fields
            .iter()
            .any(|f| matches!(f.data, QslType::BorrowedString { .. }))
        {
            schema.flags |= FLAG_FIELD_BORROW;
        }
        let mut optional = false;
        for f in &schema.fields {
            if optional && !f.optional {
                return Err(MacroError(format!(
                    "{}::{}: required field follows optional fields after macro expansion",
                    schema.name, f.name
                )));
            }
            optional |= f.optional;
        }
        for region in &schema.options.regions {
            let members = schema
                .fields
                .get(region.start..region.end)
                .ok_or_else(|| MacroError("native region outside expanded schema".into()))?;
            if members.is_empty()
                || members.iter().any(|f| {
                    f.optional
                        || f.data.fixed_wire_size().ok().flatten().is_none()
                        || matches!(f.data, QslType::FixedCompressedString(_))
                })
            {
                return Err(MacroError(format!(
                    "{}: native regions require fixed required fields",
                    schema.name
                )));
            }
        }
        schema
            .validate()
            .map_err(|e| MacroError(format!("{} after macro expansion: {e}", schema.name)))?;
        result.push(schema);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::*;
    // These cases exercise the internal expansion AST, not public QSL syntax.
    // Preserve their graph/lifetime coverage after moving markers into Rust.
    fn internal_macro_ast(source: &str) -> Result<Vec<QslSchema>, Box<dyn std::error::Error>> {
        let compact = source
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        let cleaned = source
            .replace("[macro,lifetime]", "[lifetime]")
            .replace("[macro]", "");
        let mut schemas =
            crate::parser::parse_qsl_raw_with_constants(&cleaned, &Default::default())?;
        for s in &mut schemas {
            if compact.contains(&format!("&{}[macro", s.name)) {
                s.flags |= FLAG_MACRO;
            }
        }
        Ok(super::expand_macros(&schemas)?)
    }
    #[test]
    fn nested_lifetime_modes_are_checked_after_expansion() {
        let defs = "&Base[macro,lifetime]{str value;}&Meta[macro,lifetime]{Base base;}";
        assert!(internal_macro_ast(&format!("{defs}&P{{Meta meta;}}")).is_err());
        let s = internal_macro_ast(&format!("{defs}&P[lifetime]{{Meta meta;}}")).unwrap();
        assert_eq!(s[2].fields[0].data.rust_type(true).unwrap(), "&'a str");
        let s = internal_macro_ast(
            "&Base[macro]{str value|binary_opt=view|;}&Meta[macro]{Base base;}&P{Meta meta;}",
        )
        .unwrap();
        assert!(s[2].lifetime());
        assert!(matches!(
            s[2].fields[0].data,
            QslType::BorrowedString { .. }
        ));
    }
    #[test]
    fn shared_empty_macro_dag_is_expanded_once_per_definition() {
        let mut source = String::from("&M0[macro]{}");
        for i in 1..32 {
            source.push_str(&format!("&M{i}[macro]{{M{} a;M{} b;}}", i - 1, i - 1));
        }
        let schemas = internal_macro_ast(&source).unwrap();
        assert!(schemas.iter().all(|s| s.fields.is_empty()));
    }
    #[test]
    fn nested_forward_and_repeated_macros_flatten_with_fresh_tags() {
        let schemas=internal_macro_ast("&Packet[safety]{Meta first;Meta second;u8? extra;} &Meta[macro]{Base base;u16 code;} &Base[macro]{u64 id;}").unwrap();
        assert_eq!(schemas.len(), 3);
        assert_eq!(
            schemas[0]
                .fields
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
    }
    #[test]
    fn cycles_collisions_group_presence_and_expanded_order_are_errors() {
        for source in [
            "&A[macro]{A a;}",
            "&A[macro]{B b;}&B[macro]{A a;}",
            "&M[macro]{u8 id;}&P{M m;u8 m_id;}",
            "&M[macro]{u8 id;}&P{M? m;}",
            "&M[macro]{u8? x;}&P{M m;u8 required;}",
        ] {
            assert!(internal_macro_ast(source).is_err(), "{source}");
        }
    }
}
