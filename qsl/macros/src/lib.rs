//! Mark ordinary Rust definitions; optional native representations are explicit.
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Fields, GenericArgument, ItemStruct, PathArguments, Type,
};
fn marker(args: TokenStream, item: TokenStream) -> TokenStream {
    if args.is_empty() {
        if syn::parse::<ItemStruct>(item.clone()).is_err() {
            return "compile_error!(\"qsl_macro/schem_macro require a Rust struct\");"
                .parse()
                .unwrap();
        }
        let structure = syn::parse::<ItemStruct>(item).unwrap();
        return template(structure);
    }
    if args.to_string() != "aggressive_transmutate" {
        return "compile_error!(\"qsl_macro/schem_macro accept only aggressive_transmutate\");"
            .parse()
            .unwrap();
    }
    let mut structure = parse_macro_input!(item as ItemStruct);
    if !structure.generics.params.is_empty() {
        return "compile_error!(\"aggressive_transmutate requires a fixed non-generic struct\");"
            .parse()
            .unwrap();
    }
    let Fields::Named(fields) = &mut structure.fields else {
        return "compile_error!(\"aggressive_transmutate requires named fields\");"
            .parse()
            .unwrap();
    };
    let mut checks = Vec::new();
    let mut layouts = Vec::new();
    for field in &mut fields.named {
        let name = field.ident.as_ref().unwrap();
        let mut utf8 = false;
        if let Type::Path(p) = &field.ty {
            if let Some(leaf) = p.path.segments.last() {
                if leaf.ident == "InlineString" {
                    if let PathArguments::AngleBracketed(args) = &leaf.arguments {
                        if let Some(GenericArgument::Const(n)) = args.args.first() {
                            field.ty = parse_quote!([u8;#n]);
                            utf8 = true;
                        }
                    }
                }
            }
        }
        let ty = &field.ty;
        if utf8 {
            layouts.push(quote!{(::core::mem::offset_of!(Self,#name),::qsl::storage::NativeLayout::leaf(::core::mem::size_of::<#ty>(),200,::core::mem::size_of::<#ty>()))});
            checks.push(quote!{let bytes=::core::slice::from_raw_parts(address.add(::core::mem::offset_of!(Self,#name)),::core::mem::size_of::<#ty>());::core::str::from_utf8(bytes).map_err(|_|::qsl::QslError::InvalidUtf8)?;});
        } else {
            layouts.push(quote!{(::core::mem::offset_of!(Self,#name),<#ty as ::qsl::storage::NativeField>::NATIVE_LAYOUT)});
            checks.push(quote!{<#ty as ::qsl::storage::NativeField>::validate_native_fields(address.add(::core::mem::offset_of!(Self,#name)))?;});
        }
    }
    let mut repr_c = false;
    for attr in structure.attrs.iter().filter(|a| a.path().is_ident("repr")) {
        let result=attr.parse_nested_meta(|meta|{if meta.path.is_ident("C"){repr_c=true;Ok(())}else if meta.path.is_ident("align"){let content;syn::parenthesized!(content in meta.input);let _:syn::LitInt=content.parse()?;Ok(())}else{Err(meta.error("aggressive_transmutate requires repr(C), optionally with align(N); packed/Rust/transparent are incompatible"))}});
        if let Err(error) = result {
            return error.to_compile_error().into();
        }
    }
    if !repr_c {
        structure.attrs.push(parse_quote!(#[repr(C)]));
    }
    let mut derives = Vec::new();
    for attr in &structure.attrs {
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|meta| {
                if let Some(id) = meta.path.get_ident() {
                    derives.push(id.to_string());
                }
                Ok(())
            });
        }
    }
    if !derives.iter().any(|d| d == "Clone") {
        structure.attrs.push(parse_quote!(#[derive(Clone)]));
    }
    if !derives.iter().any(|d| d == "Copy") {
        structure.attrs.push(parse_quote!(#[derive(Copy)]));
    }
    let name = &structure.ident;
    quote!{#structure
 unsafe impl ::qsl::storage::NativeField for #name{const NATIVE_LAYOUT: ::qsl::storage::NativeLayout=::qsl::storage::NativeLayout::fields(&[#(#layouts),*]);unsafe fn validate_native_fields(address:*const u8)->::qsl::QslResult<()>{#(#checks)*Ok(())}}
 }.into()
}
#[proc_macro_attribute]
pub fn qsl_macro(args: TokenStream, item: TokenStream) -> TokenStream {
    marker(args, item)
}
#[proc_macro_attribute]
pub fn schem_macro(args: TokenStream, item: TokenStream) -> TokenStream {
    marker(args, item)
}

// Reflection is build-time schema metadata, separate from any unsafe ABI promise.
fn template(structure: ItemStruct) -> TokenStream {
    let name = &structure.ident;
    let Fields::Named(fields) = &structure.fields else {
        return syn::Error::new_spanned(&structure, "QSL templates require named fields")
            .to_compile_error()
            .into();
    };
    let mut generics = structure.generics.clone();
    let scalar_parameters=structure.generics.type_params().filter(|p|!p.bounds.iter().any(|b|matches!(b,syn::TypeParamBound::Trait(t) if t.path.segments.last().is_some_and(|s|s.ident=="QslMacro")))).map(|p|p.ident.to_string()).collect::<std::collections::HashSet<_>>();
    fn atom(ty: &Type, parameters: &std::collections::HashSet<String>) -> bool {
        match ty {
            Type::Array(a) => atom(&a.elem, parameters),
            Type::Reference(r) => matches!(r.elem.as_ref(),Type::Path(p) if p.path.is_ident("str")),
            Type::Path(p) => {
                p.path
                    .segments
                    .last()
                    .is_some_and(|leaf| match leaf.ident.to_string().as_str() {
                        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128"
                        | "i128" | "String" | "StackString" | "InlineString" | "InlineArray" => {
                            true
                        }
                        "Vec" | "Option" | "Box" => {
                            if let PathArguments::AngleBracketed(a) = &leaf.arguments {
                                a.args
                                    .iter()
                                    .filter_map(|g| {
                                        if let GenericArgument::Type(t) = g {
                                            Some(t)
                                        } else {
                                            None
                                        }
                                    })
                                    .all(|t| atom(t, parameters))
                            } else {
                                false
                            }
                        }
                        n if parameters.contains(n) => true,
                        _ => false,
                    })
            }
            _ => false,
        }
    }
    let mut append = Vec::new();
    for f in &fields.named {
        let field = f.ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        if atom(ty, &scalar_parameters) {
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: ::qsl::QslFieldType));
            append.push(quote!{if !<#ty as ::qsl::QslFieldType>::VALID {return Err(::qsl::QslError::InvalidSchema);}let data=<#ty as ::qsl::QslFieldType>::qsl_type();let bits=data.default_compact_bits()?;fields.push(::qsl::QslField{name:#field.into(),optional:<#ty as ::qsl::QslFieldType>::OPTIONAL,data,bits,tag:fields.len()+1,..Default::default()});});
        } else {
            let mut inner = ty;
            let mut optional = false;
            if let Type::Path(p) = ty {
                if let Some(leaf) = p.path.segments.last() {
                    if leaf.ident == "Option" {
                        if let PathArguments::AngleBracketed(a) = &leaf.arguments {
                            if let Some(GenericArgument::Type(t)) = a.args.first() {
                                inner = t;
                                optional = true;
                            }
                        }
                    }
                }
            }
            if optional {
                append.push(quote! {return Err(::qsl::QslError::InvalidSchema);});
                continue;
            }
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#inner: ::qsl::QslMacro));
            append.push(quote!{for mut field in <#inner as ::qsl::QslMacro>::qsl_fields()? {field.name=format!("{}_{}",#field,field.name);field.optional|=#optional;field.tag=fields.len()+1;fields.push(field);}});
        }
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote!{#structure impl #impl_generics ::qsl::QslMacro for #name #type_generics #where_clause {fn qsl_fields()->::qsl::QslResult<Vec<::qsl::QslField>>{let mut fields=Vec::new();#(#append)*Ok(fields)}}}.into()
}
