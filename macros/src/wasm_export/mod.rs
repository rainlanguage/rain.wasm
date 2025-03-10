use attrs::WasmExportAttrs;
use syn::{Error, Item};
use proc_macro::TokenStream;

mod attrs;
mod tools;
mod impl_block;

// wasm_export attribute keys
pub const SKIP_ATTR: &str = "skip";
pub const WASM_EXPORT_ATTR: &str = "wasm_export";
pub const UNCHECKED_RETURN_TYPE_ATTR: &str = "unchecked_return_type";

/// Starts macro parsing and expansion process by routing the parse towards corresponding
/// parse logic based on input type
pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let input = syn::parse(item)?;
    let top_attrs = syn::parse::<WasmExportAttrs>(attr)?;

    // parse the input as an impl block, this will result in an error as intended
    // if the macro was used elsewhere, but this can change as more features and
    // use cases may arrive in future
    match input {
        Item::Impl(mut impl_item) => {
            if let Some((_, span)) = top_attrs.unchecked_return_type {
                return Err(Error::new(
                    span,
                    "unexpected `unchecked_return_type` attributes, it can only be used for impl block methods",
                ));
            }
            impl_block::parse(&mut impl_item, top_attrs.forward_attrs)
        }
        _ => Err(Error::new_spanned(
            &input,
            "unexpected input, wasm_export macro is only applicable to impl blocks",
        )),
    }
}
