use quote::ToTokens;
use super::{try_extract_result_inner_type, SKIP_PARAM};
use crate::wasm_export::{UNCHECKED_RETURN_TYPE_PARAM, WASM_EXPORT_ATTR};
use syn::{punctuated::Punctuated, Attribute, ImplItemFn, Meta, Token, Type, Error};

/// Handles wasm_export macro attributes for a given method
pub fn handle_attrs(
    method: &mut ImplItemFn,
) -> Result<(Vec<Attribute>, Option<Type>, bool), Error> {
    // Forward the wasm_bindgen attributes to the new function
    let mut keep = Vec::new();
    let mut should_skip = false;
    let mut unchecked_ret_type = None;
    let mut wasm_bindgen_attrs: Vec<Attribute> = Vec::new();
    for attr in &method.attrs {
        if attr.path().is_ident(WASM_EXPORT_ATTR) {
            keep.push(false);
            let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for meta in nested {
                if meta.path().is_ident(UNCHECKED_RETURN_TYPE_PARAM) {
                    if unchecked_ret_type.is_some() {
                        return Err(Error::new_spanned(
                            meta,
                            "duplicate unchecked_return_type attribute",
                        ));
                    } else if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(str),
                        ..
                    }) = &meta.require_name_value()?.value
                    {
                        unchecked_ret_type = Some(str.value());
                    } else {
                        return Err(Error::new_spanned(meta, "expected string literal"));
                    }
                } else if meta.path().is_ident(SKIP_PARAM) {
                    if should_skip {
                        return Err(Error::new_spanned(meta, "duplicate skip attribute"));
                    }
                    meta.require_path_only()?;
                    should_skip = true;
                } else {
                    // include unchanged
                    wasm_bindgen_attrs.push(syn::parse_quote!(
                        #[wasm_bindgen(#meta)]
                    ));
                }
            }
        } else {
            keep.push(true);
        }
    }

    // extract #[wasm_export] attrs from input
    let mut keep = keep.into_iter();
    method.attrs.retain(|_| keep.next().unwrap());

    // Create the modified return type and add the modified unchecked_return_type
    // Falls back to original return inner type if not provided by unchecked_return_type
    let inner_ret_type = try_extract_result_inner_type(method).cloned();
    if let Some(v) = unchecked_ret_type.or(inner_ret_type
        .as_ref()
        .map(|v| format!("{}", v.to_token_stream())))
    {
        let return_type = format!("WasmEncodedResult<{}>", v);
        wasm_bindgen_attrs.push(syn::parse_quote!(
            #[wasm_bindgen(unchecked_return_type = #return_type)]
        ));
    }

    Ok((wasm_bindgen_attrs, inner_ret_type, should_skip))
}
