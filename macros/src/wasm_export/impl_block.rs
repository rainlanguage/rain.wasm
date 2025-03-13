use quote::quote;
use proc_macro2::TokenStream;
use super::{tools::*, attrs::*};
use syn::{
    punctuated::Punctuated, Error, ImplItemFn, Meta, Token, Type, ImplItem, ItemImpl, ReturnType,
};

/// Parses an entire impl block methods and generates the wasm exported impl block with all the expected methods
pub fn parse(impl_block: &mut ItemImpl, top_attrs: WasmExportAttrs) -> Result<TokenStream, Error> {
    // bail early if invalid attribute was identified
    if let Some((_, span)) = top_attrs.unchecked_return_type {
        return Err(Error::new(
            span,
            "unexpected `unchecked_return_type` attribute, it can only be used for impl block methods or standalone functions",
        ));
    }

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
                        false,
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
    if !top_attrs.forward_attrs.is_empty() {
        let forward = &top_attrs.forward_attrs;
        export_impl_block.attrs = vec![syn::parse_quote!(
            #[wasm_bindgen(#(#forward),*)]
        )];
    } else {
        export_impl_block.attrs = vec![syn::parse_quote!(
            #[wasm_bindgen]
        )];
    }

    // Create two impl blocks, original and exporting one
    let output = quote! {
        #impl_block

        #export_impl_block
    };

    Ok(output)
}

/// Handles wasm_export macro attributes for a given impl method
fn handle_method_attrs(method: &mut ImplItemFn) -> Result<(Vec<Meta>, Option<Type>, bool), Error> {
    // start parsing nested attributes of this method
    let mut keep = Vec::new();
    let mut wasm_export_attrs = WasmExportAttrs::default();
    for attr in method.attrs.iter_mut() {
        if attr.path().is_ident(AttrKeys::WASM_EXPORT) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    use proc_macro2::Span;

    #[test]
    fn test_handle_method_attrs_happy() {
        let mut method: ImplItemFn = parse_quote!(
            #[some_external_macro]
            #[wasm_export(some_forward_attr, unchecked_return_type = "string", skip)]
            pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = handle_method_attrs(&mut method).unwrap();
        let expected = (
            vec![
                parse_quote!(some_forward_attr),
                parse_quote!(unchecked_return_type = "WasmEncodedResult<string>"),
            ],
            Some(parse_quote!(SomeType)),
            true,
        );
        assert_eq!(result, expected);
        assert_eq!(method.attrs, vec![parse_quote!(#[some_external_macro])]);

        let mut method: ImplItemFn = parse_quote!(
            #[wasm_export]
            pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = handle_method_attrs(&mut method).unwrap();
        let expected = (
            vec![parse_quote!(
                unchecked_return_type = "WasmEncodedResult<SomeType>"
            )],
            Some(parse_quote!(SomeType)),
            false,
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_handle_method_attrs_unhappy() {
        // bad delimiter
        let mut method: ImplItemFn = parse_quote!(
            #[wasm_export(some_forward_attr; skip)]
            pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let err = handle_method_attrs(&mut method).unwrap_err();
        assert_eq!(
            err.to_string(),
            "expected `,` as wasm_export attributes must be delimited by comma"
        );
    }

    #[test]
    fn test_parse_happy() {
        let mut method: ItemImpl = parse_quote!(
            impl SomeStrcut {
                #[wasm_export(some_forward_attr, unchecked_return_type = "string")]
                pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                    Ok(SomeType::new())
                }
                #[some_external_macro]
                #[wasm_export(skip)]
                pub fn some_skip_fn(arg1: String) -> SomeType {
                    SomeType::new()
                }
            }
        );
        let result = parse(&mut method, WasmExportAttrs::default()).unwrap();
        let expected: TokenStream = parse_quote!(
            impl SomeStrcut {
                pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                    Ok(SomeType::new())
                }
                #[some_external_macro]
                pub fn some_skip_fn(arg1: String) -> SomeType {
                    SomeType::new()
                }
            }
            #[wasm_bindgen]
            impl SomeStrcut {
                #[allow(non_snake_case)]
                #[wasm_bindgen(some_forward_attr, unchecked_return_type = "WasmEncodedResult<string>")]
                pub fn some_fn__wasm_export(arg1: String) -> WasmEncodedResult<SomeType> {
                    Self::some_fn(arg1).into()
                }
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_unhappy() {
        // error for top unchecked_return_type attr
        let mut method: ItemImpl = parse_quote!(
            impl SomeStrcut {
                #[wasm_export(some_forward_attr, unchecked_return_type = "string")]
                pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                    Ok(SomeType::new())
                }
            }
        );
        let wasm_export_attr = WasmExportAttrs {
            unchecked_return_type: Some(("string".to_string(), Span::call_site())),
            ..Default::default()
        };
        let err = parse(&mut method, wasm_export_attr).unwrap_err();
        assert_eq!(err.to_string(), "unexpected `unchecked_return_type` attribute, it can only be used for impl block methods or standalone functions");

        // error for method with non result return type
        let mut method: ItemImpl = parse_quote!(
            impl SomeStrcut {
                #[wasm_export(some_forward_attr, unchecked_return_type = "string")]
                pub fn some_fn(arg1: String) -> SomeType {
                    SomeType::new()
                }
            }
        );
        let err = parse(&mut method, WasmExportAttrs::default()).unwrap_err();
        assert_eq!(err.to_string(), "expected Result<T, E> return type");
    }
}
