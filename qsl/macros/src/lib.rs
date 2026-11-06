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
        return item;
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
            layouts.push(quote!{(::std::mem::offset_of!(Self,#name),::qsl::modernization::NativeLayout::leaf(::std::mem::size_of::<#ty>(),200,::std::mem::size_of::<#ty>()))});
            checks.push(quote!{let bytes=::std::slice::from_raw_parts(address.add(::std::mem::offset_of!(Self,#name)),::std::mem::size_of::<#ty>());::std::str::from_utf8(bytes).map_err(|_|::qsl::QslError::InvalidUtf8)?;});
        } else {
            layouts.push(quote!{(::std::mem::offset_of!(Self,#name),<#ty as ::qsl::modernization::NativeField>::NATIVE_LAYOUT)});
            checks.push(quote!{<#ty as ::qsl::modernization::NativeField>::validate_native_fields(address.add(::std::mem::offset_of!(Self,#name)))?;});
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
 unsafe impl ::qsl::modernization::NativeField for #name{const NATIVE_LAYOUT: ::qsl::modernization::NativeLayout=::qsl::modernization::NativeLayout::fields(&[#(#layouts),*]);unsafe fn validate_native_fields(address:*const u8)->::qsl::QslResult<()>{#(#checks)*Ok(())}}
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
