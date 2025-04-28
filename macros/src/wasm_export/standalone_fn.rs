use quote::quote;
use proc_macro2::{Span, TokenStream};
use syn::{punctuated::Punctuated, Error, ItemFn, Meta, ReturnType, Token};
use super::{
    attrs::{handle_attrs_sequence, AttrKeys, WasmExportAttrs},
    tools::{create_standalone_function_call, extend_err_msg, populate_name},
};

/// 1. Attributes to forward to wasm_bindgen.
/// 2. Optional unchecked return type override (Type name, Span).
/// 3. Boolean indicating if the function should be skipped.
type FnAttrsResult = (Vec<Meta>, Option<(String, Span)>, bool);

/// Parses a standalone function and generates the wasm exported function
pub fn parse(func: &mut ItemFn, mut top_attrs: WasmExportAttrs) -> Result<TokenStream, Error> {
    // 1. Handle function-level attributes
    let (fn_forward_attrs, return_type_override, should_skip) = handle_fn_attrs(func)?;

    if should_skip {
        // If skipped, just return the original function definition (with wasm_export attribute removed)
        return Ok(quote!(#func));
    }

    // Combine top-level and function-level forward attributes
    top_attrs.forward_attrs.extend(fn_forward_attrs);
    top_attrs.unchecked_return_type = return_type_override; // Use fn-level override if present

    // 2. Validate return type and determine the inner type T for Result<T, E>
    let original_return_type = match top_attrs.handle_return_type(&func.sig.output) {
        Some(ty) => ty,
        None => {
            let msg = "expected Result<T, E> return type";
            return match &func.sig.output {
                ReturnType::Default => Err(Error::new_spanned(&func.sig, msg)),
                ReturnType::Type(_, _) => Err(Error::new_spanned(&func.sig.output, msg)),
            };
        }
    };

    // 3. Create the export function
    let original_fn_ident = &func.sig.ident;
    let mut export_fn = func.clone();

    // Set export function name (e.g., original_name__wasm_export)
    export_fn.sig.ident = populate_name(original_fn_ident);

    // Remove non-wasm_bindgen attributes (they stay on the original function)
    export_fn.attrs.clear();

    // Add #[wasm_bindgen(...)] attribute
    let combined_forward_attrs = &top_attrs.forward_attrs;
    if !combined_forward_attrs.is_empty() {
        export_fn.attrs.push(syn::parse_quote!(
            #[wasm_bindgen(#(#combined_forward_attrs),*)]
        ));
    } else {
        // Add wasm_bindgen even if no specific attrs were forwarded
        export_fn.attrs.push(syn::parse_quote!(#[wasm_bindgen]));
    }
    // Allow non_snake_case for the generated function name
    export_fn
        .attrs
        .push(syn::parse_quote!(#[allow(non_snake_case)]));

    // Set export function return type to WasmEncodedResult<T>
    export_fn.sig.output = syn::parse_quote!(-> WasmEncodedResult<#original_return_type>);

    // Set export function body to call the original function
    export_fn.block = Box::new(create_standalone_function_call(
        original_fn_ident,
        &func.sig.inputs,
        func.sig.asyncness.is_some(), // Pass true if original fn is async
    ));

    // 4. Combine original and exported function tokens
    let output = quote! {
        #func // The original function (with wasm_export attr removed)

        #export_fn
    };

    Ok(output)
}

/// Handles wasm_export macro attributes for a standalone function.
/// This is similar to `handle_method_attrs` but adapted for ItemFn.
fn handle_fn_attrs(func: &mut ItemFn) -> Result<FnAttrsResult, Error> {
    let mut keep_indices = Vec::new();
    let mut wasm_export_attrs = WasmExportAttrs::default();

    for attr in func.attrs.iter() {
        if attr.path().is_ident(AttrKeys::WASM_EXPORT) {
            // Skip parsing if there are no nested attributes (e.g., #[wasm_export])
            if !matches!(attr.meta, Meta::Path(_)) {
                let nested_seq = attr
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .map_err(extend_err_msg(
                        " as wasm_export attributes must be delimited by comma",
                    ))?;
                handle_attrs_sequence(nested_seq, &mut wasm_export_attrs)?;
            }
            // Mark this attribute for removal
            keep_indices.push(false);
        } else {
            // Mark other attributes to be kept on the original function
            keep_indices.push(true);
        }
    }

    // Remove wasm_export attributes from the original function
    let mut keep_iter = keep_indices.into_iter();
    func.attrs.retain(|_| keep_iter.next().unwrap_or(true));

    // Cannot use skip and unchecked_return_type together on the same function
    if let (Some(skip_span), Some((_, urt_span))) = (
        wasm_export_attrs.should_skip,
        wasm_export_attrs.unchecked_return_type.as_ref(),
    ) {
        let mut err = Error::new(
            skip_span,
            "`skip` attribute cannot be used together with `unchecked_return_type`",
        );
        err.combine(Error::new(
            *urt_span,
            "`unchecked_return_type` attribute cannot be used together with `skip`",
        ));
        return Err(err);
    }

    Ok((
        wasm_export_attrs.forward_attrs,
        wasm_export_attrs.unchecked_return_type,
        wasm_export_attrs.should_skip.is_some(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_standalone_fn_basic() {
        let mut func: ItemFn = parse_quote!(
            #[wasm_export(js_name = "exportedName")]
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

            #[wasm_bindgen(
                js_name = "exportedName",
                unchecked_return_type = "WasmEncodedResult<u32>"
            )]
            #[allow(non_snake_case)]
            pub async fn my_async_func__wasm_export(a: String) -> WasmEncodedResult<u32> {
                my_async_func(a).await.into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_standalone_fn_with_top_attrs() {
        let mut func: ItemFn = parse_quote!(
            #[wasm_export(js_name = "specificName")]
            fn my_sync_func() -> Result<(), JsValue> {
                Ok(())
            }
        );
        // Simulate #[wasm_export(catch)] on top
        let top_attrs: WasmExportAttrs = syn::parse_quote!(catch);
        let result = parse(&mut func, top_attrs).unwrap();

        let expected: TokenStream = parse_quote!(
            fn my_sync_func() -> Result<(), JsValue> {
                Ok(())
            }

            #[wasm_bindgen(
                catch,
                js_name = "specificName",
                unchecked_return_type = "WasmEncodedResult<()>"
            )]
            #[allow(non_snake_case)]
            fn my_sync_func__wasm_export() -> WasmEncodedResult<()> {
                my_sync_func().into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_standalone_fn_skip() {
        let mut func: ItemFn = parse_quote!(
            #[other_attr]
            #[wasm_export(skip)]
            pub fn skipped_func() -> Result<String, JsValue> {
                Ok("hello".to_string())
            }
        );
        let top_attrs = WasmExportAttrs::default();
        let result = parse(&mut func, top_attrs).unwrap();

        let expected: TokenStream = parse_quote!(
            #[other_attr]
            pub fn skipped_func() -> Result<String, JsValue> {
                Ok("hello".to_string())
            }
        );
        // The skipped function should just be the original function, with wasm_export removed
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_standalone_fn_return_override() {
        let mut func: ItemFn = parse_quote!(
            #[wasm_export(unchecked_return_type = "MyJsType")]
            pub fn override_func() -> Result<MyRustType, JsValue> {
                Ok(MyRustType)
            }
        );
        let top_attrs = WasmExportAttrs::default();
        let result = parse(&mut func, top_attrs).unwrap();

        let expected: TokenStream = parse_quote!(
            pub fn override_func() -> Result<MyRustType, JsValue> {
                Ok(MyRustType)
            }

            #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<MyJsType>")]
            #[allow(non_snake_case)]
            pub fn override_func__wasm_export() -> WasmEncodedResult<MyRustType> {
                override_func().into()
            }
        );
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_standalone_fn_no_result_error() {
        let mut func: ItemFn = parse_quote!(
            #[wasm_export]
            pub fn not_a_result() -> String {
                "hello".to_string()
            }
        );
        let top_attrs = WasmExportAttrs::default();
        let err = parse(&mut func, top_attrs).unwrap_err();
        assert_eq!(err.to_string(), "expected Result<T, E> return type");
    }

    #[test]
    fn test_parse_standalone_fn_skip_and_override_error() {
        let mut func: ItemFn = parse_quote!(
            #[wasm_export(skip, unchecked_return_type = "string")]
            pub fn invalid_attrs() -> Result<(), JsValue> {
                Ok(())
            }
        );
        let top_attrs = WasmExportAttrs::default();
        let err = parse(&mut func, top_attrs).unwrap_err();
        assert!(err
            .to_string()
            .contains("`skip` attribute cannot be used together with `unchecked_return_type`"));
    }
}
