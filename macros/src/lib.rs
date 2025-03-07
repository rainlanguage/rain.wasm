use proc_macro::TokenStream;

mod wasm_export;

#[proc_macro_attribute]
pub fn wasm_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    match wasm_export::expand(attr, item) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error().into(),
    }
}
