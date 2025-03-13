use quote::quote;
use proc_macro2::TokenStream;
use super::{tools::*, attrs::WasmExportAttrs};
use syn::{Error, ItemFn, ReturnType, Visibility};

/// Parses a standalone function and generates the wasm exported function
pub fn parse(func: &mut ItemFn, mut attrs: WasmExportAttrs) -> Result<TokenStream, Error> {
    // process the func only if its visibility is pub
    if let syn::Visibility::Public(_) = func.vis {
        // create exported func and apply all the modifications
        let mut export_func = func.clone();

        let return_type = attrs.handle_return_type(&func.sig.output);
        let WasmExportAttrs { forward_attrs, .. } = attrs;

        // must have Result<> return type
        if let Some(return_type) = return_type {
            let org_fn_ident = &func.sig.ident;

            // set exported func name, it is appended with __wasm_export
            export_func.sig.ident = populate_name(org_fn_ident);

            // forward attributes for exported func
            export_func.attrs = vec![syn::parse_quote!(#[allow(non_snake_case)])];
            if !forward_attrs.is_empty() {
                export_func.attrs.push(syn::parse_quote!(
                    #[wasm_bindgen(#(#forward_attrs),*)]
                ));
            }

            // set exported func return type as WasmEncodedResult
            export_func.sig.output = syn::parse_quote!(-> WasmEncodedResult<#return_type>);

            // call the original func as body of the exported func
            export_func.block = Box::new(create_function_call(
                org_fn_ident,
                &func.sig.inputs,
                func.sig.asyncness.is_some(),
                true,
            ));

            // Create two functions, original and exporting one
            let output = quote! {
                #func

                #export_func
            };

            Ok(output)
        } else {
            let msg = "expected Result<T, E> return type";
            match &func.sig.output {
                ReturnType::Default => Err(Error::new_spanned(&func.sig, msg)),
                ReturnType::Type(_, _) => Err(Error::new_spanned(&func.sig.output, msg)),
            }
        }
    } else {
        let msg = "expected pub visibility";
        match &func.vis {
            Visibility::Inherited => Err(Error::new_spanned(func.sig.fn_token, msg)),
            _ => Err(Error::new_spanned(&func.vis, msg)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    use proc_macro2::Span;

    #[test]
    fn test_parse_happy() {
        let mut method: ItemFn = parse_quote!(
            pub async fn some_fn(arg1: String) -> Result<SomeType, Error> {}
        );
        let wasm_export_attrs = WasmExportAttrs {
            should_skip: None,
            forward_attrs: vec![parse_quote!(some_forward_attr)],
            unchecked_return_type: Some(("string".to_string(), Span::call_site())),
        };
        let result = parse(&mut method, wasm_export_attrs).unwrap();
        let expected: TokenStream = parse_quote!(
            pub async fn some_fn(arg1: String) -> Result<SomeType, Error> {}
            #[allow(non_snake_case)]
            #[wasm_bindgen(some_forward_attr, unchecked_return_type = "WasmEncodedResult<string>")]
            pub async fn some_fn__wasm_export(arg1: String) -> WasmEncodedResult<SomeType> {
                some_fn(arg1).await.into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_unhappy() {
        // error for pub visibility
        let mut method: ItemFn = parse_quote!(
            #[wasm_export(some_forward_attr, unchecked_return_type = "string")]
            fn some_fn(arg1: String) -> Result<SomeType, Error> {}
        );
        let err = parse(&mut method, WasmExportAttrs::default()).unwrap_err();
        assert_eq!(err.to_string(), "expected pub visibility");

        // error for method with non result return type
        let mut method: ItemFn = parse_quote!(
            #[wasm_export(some_forward_attr, unchecked_return_type = "string")]
            pub fn some_fn(arg1: String) -> SomeType {}
        );
        let err = parse(&mut method, WasmExportAttrs::default()).unwrap_err();
        assert_eq!(err.to_string(), "expected Result<T, E> return type");
    }
}
