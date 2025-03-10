use quote::quote;
use proc_macro::TokenStream;
use super::{WASM_EXPORT_ATTR, tools::*, attrs::*};
use syn::{
    punctuated::Punctuated, Error, ImplItemFn, Meta, Token, Type, ImplItem, ItemImpl, ReturnType,
};

/// Parses an entire impl block methods and generates the wasm exported impl block with all the expected methods
pub fn parse(impl_block: &mut ItemImpl, top_attrs: Vec<Meta>) -> Result<TokenStream, Error> {
    // create vector to store exported items
    // and loop over items inside of the impl block and process each method
    let mut export_items = Vec::new();
    for item in impl_block.items.iter_mut() {
        if let ImplItem::Fn(method) = item {
            // process the method only if its visibility is pub
            if let syn::Visibility::Public(_) = method.vis {
                // process method attributes
                let (forward_attrs, return_type, should_skip) = handle_method_attrs(method)?;
                if should_skip {
                    continue;
                }

                // items included for exporting must all have Result<> return type
                if let Some(return_type) = return_type {
                    let org_fn_ident = &method.sig.ident;

                    // create exported method and apply all the modifications
                    let mut export_method = method.clone();

                    // set exported method name, it is appended with __wasm_export
                    export_method.sig.ident = populate_name(org_fn_ident);

                    // forward attributes for exported method
                    export_method.attrs = vec![syn::parse_quote!(#[allow(non_snake_case)])];
                    if !forward_attrs.is_empty() {
                        export_method.attrs.push(syn::parse_quote!(
                            #[wasm_bindgen(#(#forward_attrs),*)]
                        ));
                    }

                    // set exported method return type as WasmEncodedResult
                    export_method.sig.output =
                        syn::parse_quote!(-> WasmEncodedResult<#return_type>);

                    // call the original method as body of the exported method
                    export_method.block = create_function_call(
                        org_fn_ident,
                        &method.sig.inputs,
                        method.sig.asyncness.is_some(),
                    );

                    export_items.push(ImplItem::Fn(export_method));
                } else {
                    let msg = "expected Result<T, E> return type";
                    return match &method.sig.output {
                        ReturnType::Default => Err(Error::new_spanned(&method.sig, msg)),
                        ReturnType::Type(_, _) => Err(Error::new_spanned(&method.sig.output, msg)),
                    };
                }
            }
        }
    }

    let mut export_impl_block = impl_block.clone();
    export_impl_block.items = export_items;
    export_impl_block.attrs = vec![];
    if option_env!("TEST_WASM_BINDGEN_UTILS_MACROS").is_none() {
        // we only need exports on wasm target so we apply cfg, but for easy
        // testing we need to not apply it using env so we can test the expansion
        export_impl_block.attrs.push(syn::parse_quote!(
            #[cfg(target_family = "wasm")]
        ));
    }
    if !top_attrs.is_empty() {
        export_impl_block.attrs.push(syn::parse_quote!(
            #[wasm_bindgen(#(#top_attrs),*)]
        ));
    } else {
        export_impl_block.attrs.push(syn::parse_quote!(
            #[wasm_bindgen]
        ));
    }

    // Create two impl blocks, original and exporting one
    let output = quote! {
        #impl_block

        #export_impl_block
    };

    Ok(output.into())
}

/// Handles wasm_export macro attributes for a given impl method
fn handle_method_attrs(method: &mut ImplItemFn) -> Result<(Vec<Meta>, Option<Type>, bool), Error> {
    // start parsing nested attributes of this method
    let mut keep = Vec::new();
    let mut wasm_export_attrs = WasmExportAttrs::default();
    for attr in method.attrs.iter_mut() {
        if attr.path().is_ident(WASM_EXPORT_ATTR) {
            // skip parsing by delimited comma if there are no nested attrs
            if !matches!(attr.meta, Meta::Path(_)) {
                let nested_seq = attr
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .map_err(extend_err_msg(
                        " as wasm_export attributes must be delimited by comma",
                    ))?;
                handle_attrs_sequence(nested_seq, &mut wasm_export_attrs)?;
            }
            keep.push(false);
        } else {
            keep.push(true);
        }
    }

    // extract wasm_export attrs from this method input
    let mut keep = keep.into_iter();
    method.attrs.retain(|_| keep.next().unwrap_or(true));

    // handle return type
    let ret_type = wasm_export_attrs.handle_return_type(&method.sig.output);

    Ok((
        wasm_export_attrs.forward_attrs,
        ret_type,
        wasm_export_attrs.should_skip.is_some(),
    ))
}
