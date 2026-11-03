//! Rust declarations remain ordinary Rust types. The QSL build reads marked
//! source declarations from the explicit input list; no runtime reflection.
use proc_macro::{TokenStream, TokenTree};
fn marker(args: TokenStream, item: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return "compile_error!(\"qsl_macro/schem_macro take no arguments\");"
            .parse()
            .unwrap();
    }
    if !item
        .clone()
        .into_iter()
        .any(|t| matches!(t,TokenTree::Ident(id) if id.to_string()=="struct"))
    {
        return "compile_error!(\"qsl_macro/schem_macro require a Rust struct\");"
            .parse()
            .unwrap();
    }
    item
}
#[proc_macro_attribute]
pub fn qsl_macro(args: TokenStream, item: TokenStream) -> TokenStream {
    marker(args, item)
}
#[proc_macro_attribute]
pub fn schem_macro(args: TokenStream, item: TokenStream) -> TokenStream {
    marker(args, item)
}
