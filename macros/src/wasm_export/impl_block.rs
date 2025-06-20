use quote::quote;
use proc_macro2::TokenStream;
use super::{builder::*, attrs::*};
use syn::{Error, ImplItem, ItemImpl, ReturnType};

/// Parses an entire impl block methods and generates the wasm exported impl block with all the expected methods
pub fn parse(impl_block: &mut ItemImpl, top_attrs: WasmExportAttrs) -> Result<TokenStream, Error> {
    // bail early if invalid attributes were identified
    if let Some((_, span)) = top_attrs.unchecked_return_type {
        return Err(Error::new(
            span,
            "unexpected `unchecked_return_type` attribute, it can only be used for impl block methods or standalone functions",
        ));
    }
    if let Some(span) = top_attrs.preserve_js_class {
        return Err(Error::new(
            span,
            "unexpected `preserve_js_class` attribute, it can only be used for impl block methods or standalone functions",
        ));
    }
    if let Some((_, span)) = top_attrs.return_description {
        return Err(Error::new(
            span,
            "unexpected `return_description` attribute, it can only be used for impl block methods or standalone functions",
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
                let mut wasm_export_attrs = WasmExportAttrs::handle_method_attrs(method)?;

                // skip this method if skip attr was detected
                if wasm_export_attrs.should_skip.is_some() {
                    continue;
                }

                let return_type = wasm_export_attrs.handle_return_type(&method.sig.output);
                let WasmExportAttrs {
                    forward_attrs,
                    preserve_js_class,
                    ..
                } = wasm_export_attrs;

                // items included for exporting must all have Result<> return type
                if let Some(return_type) = return_type {
                    // create export method with the given configurations
                    let config = WasmExportFunctionBuilderConfig {
                        forward_attrs,
                        return_type,
                        preserve_js_class,
                    };
                    let export_method =
                        WasmExportFunctionBuilder::build_export_method(method, config);

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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    use proc_macro2::Span;

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

        // error for top preserve_js_class attr
        let mut method: ItemImpl = parse_quote!(
            impl SomeStrcut {
                #[wasm_export(some_forward_attr, preserve_js_class = "string")]
                pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                    Ok(SomeType::new())
                }
            }
        );
        let wasm_export_attr = WasmExportAttrs {
            preserve_js_class: Some(Span::call_site()),
            ..Default::default()
        };
        let err = parse(&mut method, wasm_export_attr).unwrap_err();
        assert_eq!(err.to_string(), "unexpected `preserve_js_class` attribute, it can only be used for impl block methods or standalone functions");

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
