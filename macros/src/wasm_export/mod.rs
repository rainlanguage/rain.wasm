use syn::{Error, Item};
use proc_macro2::TokenStream;

mod attrs;
mod error;
mod builder;
mod impl_block;
mod standalone_fn;

/// Starts macro parsing and expansion process by routing the parse towards corresponding
/// parse logic based on input type
pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let input = syn::parse2(item)?;
    let top_attrs = syn::parse2(attr)?;

    // parse the input based on its type
    match input {
        Item::Impl(mut impl_block) => impl_block::parse(&mut impl_block, top_attrs),
        Item::Fn(mut func) => standalone_fn::parse(&mut func, top_attrs),
        _ => Err(Error::new_spanned(
            &input,
            "unexpected input, wasm_export macro is only applicable to impl blocks or functions",
        )),
    }
}
