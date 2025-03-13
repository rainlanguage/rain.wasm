use syn::{Error, Item};
use proc_macro2::TokenStream;

mod attrs;
mod tools;
mod func;
mod impl_block;

/// Starts macro parsing and expansion process by routing the parse towards corresponding
/// parse logic based on input type
pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let input = syn::parse2(item)?;
    let top_attrs = syn::parse2(attr)?;

    // parse the input as an impl block or standalone function, this will result in an
    // error as intended if the macro was used elsewhere, but this can change as more
    // features and use cases may arrive in future
    match input {
        Item::Fn(mut func) => func::parse(&mut func, top_attrs),
        Item::Impl(mut impl_block) => impl_block::parse(&mut impl_block, top_attrs),
        _ => Err(Error::new_spanned(
            &input,
            "unexpected input, wasm_export macro is only applicable to impl blocks or standalone functions",
        )),
    }
}
