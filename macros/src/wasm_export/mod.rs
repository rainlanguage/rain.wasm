use quote::quote;
use proc_macro::TokenStream;
use syn::{Error, ImplItem, ItemImpl, ReturnType};

mod attrs;
mod fn_tools;

pub use fn_tools::*;
pub use attrs::handle_attrs;

// Constants used throughout the module
pub const SKIP_ATTR: &str = "skip";
pub const WASM_EXPORT_ATTR: &str = "wasm_export";
pub const UNCHECKED_RETURN_TYPE_ATTR: &str = "unchecked_return_type";

/// Starts macro parsing and expansion process. This is the main logic of this macro
/// by processing and handling of the attributes and generating the final output
pub fn expand(_attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    // parse the input as an impl block, this will result in an error as intended if the
    // macro was used on a non impl block since we have restricted its usage only for impl
    // blocks, but this can change as more features and use cases may arrive in future
    let mut input = syn::parse::<ItemImpl>(item)?;

    // create vector to store exported items
    let mut export_items = Vec::new();

    for item in input.items.iter_mut() {
        if let ImplItem::Fn(method) = item {
            // process the method only if its visibility is pub
            if let syn::Visibility::Public(_) = method.vis {
                // process method attributes
                let (forward_attrs, inner_ret_type, should_skip) = handle_attrs(method)?;

                // exclude from exported methods if skip
                // attr was specified for this method
                if should_skip {
                    continue;
                }

                // items included for exporting must all have Result<> return type
                if let Some(inner_ret_type) = inner_ret_type {
                    let org_fn_ident = &method.sig.ident;

                    // create exported method and apply all the modifications
                    let mut export_method = method.clone();

                    // set exported method name, it is appended with __wasm_export
                    export_method.sig.ident = syn::Ident::new(
                        &format!("{}__{}", org_fn_ident, WASM_EXPORT_ATTR),
                        org_fn_ident.span(),
                    );

                    // forward attributes for exported method
                    export_method
                        .attrs
                        .push(syn::parse_quote!(#[allow(non_snake_case)]));
                    export_method.attrs.extend(forward_attrs);

                    // set exported method return type as WasmEncodedResult
                    export_method.sig.output =
                        syn::parse_quote!(-> WasmEncodedResult<#inner_ret_type>);

                    // call the original method as body of the exported method
                    let (has_self_receiver, args) = collect_function_arguments(&method.sig.inputs);
                    let call_expr = create_function_call(org_fn_ident, has_self_receiver, &args);
                    if method.sig.asyncness.is_some() {
                        export_method.block = syn::parse_quote!({
                            #call_expr.await.into()
                        });
                    } else {
                        export_method.block = syn::parse_quote!({
                            #call_expr.into()
                        });
                    }

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

    let mut export_impl = input.clone();
    export_impl.items = export_items;

    // Create two impl blocks, original and exporting one
    // Generate the output with wasm_bindgen only for the export impl block
    let output = quote! {
        #input

        #[wasm_bindgen]
        #export_impl
    };

    Ok(output.into())
}
