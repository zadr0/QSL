//! Target-source adaptation only; no runtime implementation lives here.
use crate::{QslError, QslResult};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};
#[derive(Default)]
struct AllocUse(bool);
impl<'a> Visit<'a> for AllocUse {
    fn visit_path(&mut self, p: &'a syn::Path) {
        if p.segments.iter().any(|s| {
            matches!(
                s.ident.to_string().as_str(),
                "Vec" | "String" | "Box" | "SerOut" | "QuantizedOut"
            )
        }) {
            self.0 = true;
        }
        visit::visit_path(self, p);
    }
    fn visit_expr_method_call(&mut self, e: &'a syn::ExprMethodCall) {
        if matches!(
            e.method.to_string().as_str(),
            "to_vec" | "to_owned" | "repeat"
        ) {
            self.0 = true;
        }
        visit::visit_expr_method_call(self, e);
    }
    fn visit_macro(&mut self, m: &'a syn::Macro) {
        if m.path.is_ident("vec") {
            self.0 = true;
        }
        visit::visit_macro(self, m);
    }
}
struct Edits<'a> {
    lines: Vec<usize>,
    source: &'a str,
    edits: Vec<(usize, usize, String)>,
}
impl Edits<'_> {
    fn offset(&self, p: proc_macro2::LineColumn) -> usize {
        self.lines[p.line - 1] + p.column
    }
    fn replace(&mut self, span: proc_macro2::Span, text: &str) {
        self.edits.push((
            self.offset(span.start()),
            self.offset(span.end()),
            text.into(),
        ));
    }
    fn wrap(&mut self, span: proc_macro2::Span) {
        let start = self.offset(span.start());
        let end = self.offset(span.end());
        self.edits
            .push((start, start, "::qsl::with_alloc!{\n".into()));
        self.edits.push((end, end, "\n}".into()));
    }
}
impl<'a> Visit<'a> for Edits<'_> {
    fn visit_path(&mut self, p: &'a syn::Path) {
        if p.leading_colon.is_none() {
            if let Some(first) = p.segments.first() {
                let name = first.ident.to_string();
                let replacement = match name.as_str() {
                    "Vec" => Some("::qsl::alloc::vec::Vec"),
                    "String" => Some("::qsl::alloc::string::String"),
                    "Box" => Some("::qsl::alloc::boxed::Box"),
                    "vec" => Some("::qsl::alloc::vec"),
                    _ => None,
                };
                if let Some(r) = replacement {
                    self.replace(first.ident.span(), r);
                }
            }
        }
        visit::visit_path(self, p);
    }
}
pub fn lower(source: &str) -> QslResult<String> {
    let source = source
        .replace("std::vec::", "::qsl::alloc::vec::")
        .replace("std::string::", "::qsl::alloc::string::")
        .replace("std::boxed::", "::qsl::alloc::boxed::")
        .replace("std::", "core::");
    let file = syn::parse_file(&source).map_err(|_| QslError::InvalidSchema)?;
    let mut lines = vec![0];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            lines.push(i + 1)
        }
    }
    let mut edits = Edits {
        lines,
        source: &source,
        edits: Vec::new(),
    };
    for item in &file.items {
        match item {
            syn::Item::Impl(block) => {
                let mut whole = AllocUse::default();
                whole.visit_item_impl(block);
                if block.trait_.is_some() && whole.0 {
                    edits.wrap(block.span());
                } else {
                    for member in &block.items {
                        if let syn::ImplItem::Fn(f) = member {
                            let mut used = AllocUse::default();
                            used.visit_impl_item_fn(f);
                            if used.0 {
                                edits.wrap(f.span());
                            }
                        }
                    }
                }
            }
            syn::Item::Fn(f) => {
                let mut used = AllocUse::default();
                used.visit_item_fn(f);
                if used.0 {
                    edits.wrap(f.span());
                }
            }
            _ => {}
        }
    }
    edits.visit_file(&file);
    let mut result = edits.source.to_string();
    edits.edits.sort_by_key(|(a, b, _)| (*a, *b));
    for (a, b, text) in edits.edits.into_iter().rev() {
        result.replace_range(a..b, &text)
    }
    Ok(result)
}
/// Select owned output from a proven maximum during generation, never from a runtime branch.
pub(crate) fn select_output(
    source: &str,
    schema: &crate::QslSchema,
    config: crate::GenerationConfig,
) -> QslResult<String> {
    let maximum = crate::schema_maximum_size(schema)?;
    // STACK_LIMIT bounds wire bytes inclusively; StackOut metadata is not part of the frame.
    let stack = maximum.filter(|&n| n <= config.stack_limit && n <= u32::MAX as usize);
    let output = stack.map_or_else(
        || "Vec<u8>".to_string(),
        |n| format!("::qsl::StackOut<{n}>"),
    );
    let file = syn::parse_file(source).map_err(|_| crate::QslError::InvalidSchema)?;
    let mut lines = vec![0];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            lines.push(i + 1)
        }
    }
    let mut edits = Edits {
        lines,
        source,
        edits: Vec::new(),
    };
    let mut has_serialize = false;
    let target = if schema.lifetime() {
        "::qsl::BorrowSerializable<'a>"
    } else {
        "::qsl::Serializable"
    };
    let lt = if schema.lifetime() { "<'a>" } else { "" };
    let cap_limit = if schema
        .fields
        .iter()
        .any(|f| super::interpreter::output_may_shrink(&f.data))
    {
        "usize::MAX"
    } else {
        "Self::MAX_DATA_SIZE"
    };
    for item in &file.items {
        let syn::Item::Impl(block) = item else {
            continue;
        };
        if block.trait_.is_some() {
            continue;
        }
        let syn::Type::Path(ty) = block.self_ty.as_ref() else {
            continue;
        };
        if !ty
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == schema.name)
        {
            continue;
        }
        for member in &block.items {
            let syn::ImplItem::Fn(f) = member else {
                continue;
            };
            let method = f.sig.ident.to_string();
            if method == "serialize" {
                has_serialize = true;
                if let Some(n) = stack {
                    let original =
                        &source[edits.offset(f.span().start())..edits.offset(f.span().end())];
                    if original.contains("QuantizedOut") {
                        let quant = qsl_lexer::payload_quant::PayloadQuantPlan::analyze(schema)?;
                        let cap = n
                            .checked_add(quant.padding_bytes)
                            .ok_or(crate::QslError::LengthOverflow)?;
                        // Private restore padding is not transmitted and does not change the frame limit.
                        if cap <= u32::MAX as usize {
                            edits.replace(f.span(),&format!("pub fn serialize(&self)->::qsl::QslResult<::qsl::payload_quant::QuantizedLease<Self,::qsl::StackOut<{cap}>>>{{::qsl::payload_quant::QuantizedLease::encode_owned(self)}}"));
                        }
                    }
                }
                continue;
            }
            if method == "comp_write" {
                edits.replace(f.span(),&format!("pub fn comp_write(&self)->{output}{{self.serialize_checked().expect(\"QSL compact encoding failed\")}}"));
                continue;
            }
            if method != "serialize_checked" {
                continue;
            }
            let cap = if schema.options.dynamic.is_some() {
                "self.__qsl_dyn_checked_size()?".to_string()
            } else {
                format!("<Self as {target}>::serialized_size(self)")
            };
            let write = if schema.options.dynamic.is_some() {
                "self.serialize_dyn_unchecked(out.as_mut_ptr())".to_string()
            } else {
                format!("<Self as {target}>::serialize_into_unsafe(self,out.as_mut_ptr())")
            };
            let construct = if let Some(n) = stack {
                format!("if cap>{n}{{return Err(::qsl::QslError::BufferTooSmall)}}let mut out=::qsl::StackOut::<{n}>::new();")
            } else {
                "let mut out=Vec::new();out.try_reserve_exact(cap).map_err(|_|::qsl::QslError::AllocationFailed)?;".into()
            };
            edits.replace(f.span(),&format!("pub fn serialize_checked(&self)->::qsl::QslResult<{output}>{{self.validate_values()?;let cap={cap};::qsl::validation::check_size(cap,{cap_limit})?;{construct}let n=unsafe{{{write}}};::qsl::validation::check_size(n,Self::MAX_DATA_SIZE)?;unsafe{{out.set_len(n)}}Ok(out)}}"));
        }
    }
    let mut result = source.to_string();
    edits.edits.sort_by_key(|(a, b, _)| (*a, *b));
    for (a, b, text) in edits.edits.into_iter().rev() {
        result.replace_range(a..b, &text)
    }
    result.push_str(&format!(
        "\nimpl{lt} ::qsl::SerializeOutput for {}{lt}{{type Output={output};}}\n",
        schema.name
    ));
    if !has_serialize {
        result.push_str(&format!("impl{lt} {}{lt}{{pub fn serialize(&self)->::qsl::QslResult<{output}>{{self.serialize_checked()}}}}\n",schema.name));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configured_stack_limit_selects_output_without_changing_wire_identity() {
        use crate::InterpreterBuilderExt;
        let mut identity = None;
        for limit in [0, 63, 64, 65, 128, 256] {
            let config = crate::parse_deq(&format!("!config STACK_LIMIT = {limit};"))
                .unwrap()
                .generation;
            let source = crate::QslSchemaBuilder::from_source_with_config(
                "&Sized[safety]{array<u8>(64) bytes;}",
                config,
            )
            .unwrap();
            let hashes = source.schemas()[0].hashes(Default::default()).unwrap();
            if let Some(old) = identity {
                assert_eq!(hashes, old)
            } else {
                identity = Some(hashes)
            }
            let generated = String::from_utf8(source.use_interpreter().unwrap()).unwrap();
            let normalized: String = generated.split_whitespace().collect();
            let expected = if limit >= 64 {
                "typeOutput=::qsl::StackOut<64>;"
            } else {
                "typeOutput=::qsl::alloc::vec::Vec<u8>;"
            };
            assert!(
                normalized.contains(expected),
                "wrong output for STACK_LIMIT={limit}"
            );
        }
    }
    #[test]
    fn alloc_types_are_qualified_without_moving_runtime_to_the_compiler() {
        let code=lower("pub struct S {pub bytes:Vec<u8>} impl S {pub fn a()->Vec<u8>{Vec::new()} pub fn n()->usize{std::mem::size_of::<Self>()}} ").unwrap();
        assert!(code.contains("::qsl::alloc::vec::Vec"));
        assert!(code.contains("core::mem::size_of"));
        assert!(code.contains("::qsl::with_alloc!"));
        assert!(!code.contains("std::"));
        syn::parse_file(&code).unwrap();
    }
}
