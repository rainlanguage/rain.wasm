use quote::quote;
use proc_macro2::TokenStream;
use syn::{Error, ItemFn, ReturnType, Visibility};
use super::{
    attrs::WasmExportAttrs,
    builder::{WasmExportFunctionBuilder, WasmExportFunctionBuilderConfig},
};

/// Parses a standalone function and generates the wasm exported function
pub fn parse(func: &mut ItemFn, mut top_attrs: WasmExportAttrs) -> Result<TokenStream, Error> {
    // process the func only if its visibility is pub
    if !matches!(func.vis, syn::Visibility::Public(_)) {
        let msg = "expected pub visibility";
        return match &func.vis {
            Visibility::Inherited => Err(Error::new_spanned(func.sig.fn_token, msg)),
            _ => Err(Error::new_spanned(&func.vis, msg)),
        };
    }

    // Validate return type and determine the inner type T for Result<T, E>
    let return_type = match top_attrs.handle_return_type(&func.sig.output) {
        Some(ty) => ty,
        None => {
            let msg = "expected Result<T, E> return type";
            return match &func.sig.output {
                ReturnType::Default => Err(Error::new_spanned(&func.sig, msg)),
                ReturnType::Type(_, _) => Err(Error::new_spanned(&func.sig.output, msg)),
            };
        }
    };

    // Top attrs (WasmExportAttrs) parsing logic already handles 'skip'
    // which is not valid for standalone functions, also standalone
    // functions cannot have function level attrs like impl blocks do,
    // they only have top level attrs which comes from entry point macro
    // and they are not available as part of function level attrs
    let WasmExportAttrs {
        forward_attrs,
        preserve_js_class,
        ..
    } = top_attrs;

    // create export function with the given configurations
    let config = WasmExportFunctionBuilderConfig {
        forward_attrs,
        return_type,
        preserve_js_class,
    };
    let export_fn = WasmExportFunctionBuilder::build_export_function(func, config)?;

    // Clean wasm_export attributes from original function parameters
    WasmExportFunctionBuilder::clean_parameter_attributes(&mut func.sig.inputs);

    // Combine original and exported function tokens
    let output = quote! {
        #func // The original function (with wasm_export attr removed)

        #export_fn
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    use proc_macro2::Span;

    #[test]
    fn test_parse_standalone_fn_basic() {
        let mut func: ItemFn = parse_quote!(
            pub async fn my_async_func(a: String) -> Result<u32, JsValue> {
                Ok(a.len() as u32)
            }
        );
        let top_attrs = WasmExportAttrs::default(); // No top-level attrs
        let result = parse(&mut func, top_attrs).unwrap();

        let expected: TokenStream = parse_quote!(
            pub async fn my_async_func(a: String) -> Result<u32, JsValue> {
                Ok(a.len() as u32)
            }

            #[allow(non_snake_case)]
            #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<u32>")]
            pub async fn my_async_func__wasm_export(a: String) -> WasmEncodedResult<u32> {
                my_async_func(a).await.into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_standalone_fn_with_top_attrs() {
        let mut func: ItemFn = parse_quote!(
            #[something_else]
            pub fn my_sync_func() -> Result<(), JsValue> {
                Ok(())
            }
        );
        // Simulate #[wasm_export(catch)] on top
        let top_attrs: WasmExportAttrs = syn::parse_quote!(catch, js_name = "specificName");
        let result = parse(&mut func, top_attrs).unwrap();

        let expected: TokenStream = parse_quote!(
            #[something_else]
            pub fn my_sync_func() -> Result<(), JsValue> {
                Ok(())
            }

            #[allow(non_snake_case)]
            #[wasm_bindgen(
                catch,
                js_name = "specificName",
                unchecked_return_type = "WasmEncodedResult<()>"
            )]
            pub fn my_sync_func__wasm_export() -> WasmEncodedResult<()> {
                my_sync_func().into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_standalone_fn_return_override() {
        let mut func: ItemFn = parse_quote!(
            pub fn override_func() -> Result<MyRustType, JsValue> {
                Ok(MyRustType)
            }
        );
        let top_attrs: WasmExportAttrs = syn::parse_quote!(unchecked_return_type = "MyJsType");
        let result = parse(&mut func, top_attrs).unwrap();

        let expected: TokenStream = parse_quote!(
            pub fn override_func() -> Result<MyRustType, JsValue> {
                Ok(MyRustType)
            }

            #[allow(non_snake_case)]
            #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<MyJsType>")]
            pub fn override_func__wasm_export() -> WasmEncodedResult<MyRustType> {
                override_func().into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_standalone_fn_no_result_error() {
        let mut func: ItemFn = parse_quote!(
            pub fn not_a_result() -> String {
                "hello".to_string()
            }
        );
        let top_attrs = WasmExportAttrs::default();
        let err = parse(&mut func, top_attrs).unwrap_err();
        assert_eq!(err.to_string(), "expected Result<T, E> return type");
    }

    #[test]
    fn test_parse_happy() {
        let mut method: ItemFn = parse_quote!(
            pub async fn some_fn(arg1: String) -> Result<SomeType, Error> {}
        );
        let wasm_export_attrs = WasmExportAttrs {
            should_skip: None,
            forward_attrs: vec![parse_quote!(some_forward_attr)],
            unchecked_return_type: Some(("string".to_string(), Span::call_site())),
            preserve_js_class: None,
            return_description: None,
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

    #[test]
    fn test_parse_standalone_fn_with_return_description() {
        let mut func: ItemFn = parse_quote!(
            pub fn add(a: u32, b: u32) -> Result<u32, Error> {
                Ok(a + b)
            }
        );
        let top_attrs: WasmExportAttrs =
            syn::parse_quote!(return_description = "sum of the inputs");
        let result = parse(&mut func, top_attrs).unwrap();

        let expected: TokenStream = parse_quote!(
            pub fn add(a: u32, b: u32) -> Result<u32, Error> {
                Ok(a + b)
            }

            #[allow(non_snake_case)]
            #[wasm_bindgen(
                unchecked_return_type = "WasmEncodedResult<u32>",
                return_description = "sum of the inputs"
            )]
            pub fn add__wasm_export(a: u32, b: u32) -> WasmEncodedResult<u32> {
                add(a, b).into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_standalone_fn_with_return_description_and_other_attrs() {
        let mut func: ItemFn = parse_quote!(
            pub async fn complex_calc(input: String) -> Result<i64, Error> {
                Ok(42)
            }
        );
        let top_attrs: WasmExportAttrs = syn::parse_quote!(
            js_name = "complexCalculation",
            return_description = "complex calculation result",
            catch
        );
        let result = parse(&mut func, top_attrs).unwrap();

        let expected: TokenStream = parse_quote!(
            pub async fn complex_calc(input: String) -> Result<i64, Error> {
                Ok(42)
            }

            #[allow(non_snake_case)]
            #[wasm_bindgen(
                js_name = "complexCalculation",
                catch,
                unchecked_return_type = "WasmEncodedResult<i64>",
                return_description = "complex calculation result"
            )]
            pub async fn complex_calc__wasm_export(input: String) -> WasmEncodedResult<i64> {
                complex_calc(input).await.into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }
}
