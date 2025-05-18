use quote::quote;
use std::ops::Deref;
use proc_macro2::TokenStream;
use syn::{
    punctuated::Punctuated, token::Comma, Block, FnArg, Ident, ImplItemFn, ItemFn, Path,
    PathSegment, ReturnType, Type, TypePath,
};

/// Enum to specify the context of the function call
pub enum FunctionType<'a> {
    /// Function is a method within an impl block (can be instance or static)
    Method(&'a ImplItemFn),
    /// Function is a standalone (outside any impl block)
    Standalone(&'a ItemFn),
}

/// Holds the required context for building the export method/function body
pub struct BuildExportFunctionBodyContext<'a> {
    pub function_type: FunctionType<'a>,
    pub preserve_js_class: bool,
}

/// Creates a function call expression based on the given context (method or standalone)
pub fn build_export_function_body(context: BuildExportFunctionBodyContext) -> Block {
    // destructure the method's/function's name, args and asyncness
    let (fn_name, fn_args, is_async) = match context.function_type {
        FunctionType::Method(method) => (
            &method.sig.ident,
            &method.sig.inputs,
            method.sig.asyncness.is_some(),
        ),
        FunctionType::Standalone(function) => (
            &function.sig.ident,
            &function.sig.inputs,
            function.sig.asyncness.is_some(),
        ),
    };

    // collect arguments
    let (has_self_receiver, args) = collect_function_arguments(fn_args);

    // create call expression from the original
    let call_expr = match context.function_type {
        FunctionType::Method(_) => {
            if has_self_receiver {
                // Instance method call: self.method_name(...)
                quote! { self.#fn_name(#(#args),*) }
            } else {
                // Static method call: Self::method_name(...)
                quote! { Self::#fn_name(#(#args),*) }
            }
        }
        FunctionType::Standalone(_) => {
            quote! { #fn_name(#(#args),*) }
        }
    };

    // append .await if the function is async, and .into() in
    // all cases to convert rust Result to WasmEncodedResult
    let expression = if is_async {
        quote!( #call_expr.await.into() )
    } else {
        quote!( #call_expr.into() )
    };

    // manually build a js obj that resembles the WasmEncodedResult to preserve
    // the class if preserve_js_class attr was detected and return it as JsValue
    // otherwise return the call expression unchanged
    if context.preserve_js_class {
        syn::parse_quote!({
            // bring necessary items in scope
            use std::str::FromStr;
            use js_sys::{Reflect, Object};

            // create empty js obj
            let obj = Object::new();

            // call the expression and proceed based on its result
            //
            // populate "value" field with class instance and "error" field with undefined if
            // result is Ok and vice versa if result is Err, this js obj will resemble the
            // WasmEncodedResult (that normally is serialized through serde_wasm_bindgen which
            // results in plain js objects for nested types) type in js/ts with preserving the
            // class instance for value field
            //
            // "Reflect::set" can only fail if the obj is sealed or frozen which is not the case
            // here, so it is safe to use unwrap, "Reflect::set" is similar to "obj[key] = value"
            // in js, for more info read MDN docs for Reflect
            let result = #expression;
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &error.into()).unwrap();
                }
            };

            // return as JsValue
            obj.into()
        })
    } else {
        syn::parse_quote!({
            #expression
        })
    }
}

/// Collects function arguments and determines if the function has a self receiver
pub fn collect_function_arguments(inputs: &Punctuated<FnArg, Comma>) -> (bool, Vec<TokenStream>) {
    let mut has_self_receiver = false;
    let args = inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => {
                has_self_receiver = true;
                None
            }
            FnArg::Typed(pat_type) => {
                let pat = pat_type.pat.deref();
                Some(quote! { #pat })
            }
        })
        .collect();

    (has_self_receiver, args)
}

/// Tries to extract the inner type T from a Result<T, E> type, returning None if not a Result
pub fn try_extract_result_inner_type(output: &ReturnType) -> Option<&Type> {
    if let ReturnType::Type(_, return_type) = output {
        if let Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) = return_type.deref()
        {
            if let Some(PathSegment {
                ident, arguments, ..
            }) = segments.last()
            {
                if *ident == "Result" {
                    if let syn::PathArguments::AngleBracketed(args) = arguments {
                        if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
                            return Some(t);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Creates the function name from the original name, it is appended by __wasm_export
pub fn populate_name(org_fn_ident: &Ident) -> Ident {
    Ident::new(
        &format!("{}__wasm_export", org_fn_ident),
        org_fn_ident.span(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use proc_macro2::{Span, TokenStream};
    use syn::{parse::Parser, parse_quote};

    #[test]
    fn test_build_body_method_async() {
        // async method static
        let method: ImplItemFn = parse_quote!(
            pub async fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Method(&method),
            preserve_js_class: false,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({ Self::some_name((arg1, arg2)).await.into() });
        assert_eq!(result, expected);

        // async method with self
        let method: ImplItemFn = parse_quote!(
            pub async fn some_name(&self, (arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Method(&method),
            preserve_js_class: false,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({ self.some_name((arg1, arg2)).await.into() });
        assert_eq!(result, expected);

        // async method static with preserve class
        let method: ImplItemFn = parse_quote!(
            pub async fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Method(&method),
            preserve_js_class: true,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({
            use std::str::FromStr;
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = Self::some_name((arg1, arg2)).await.into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);

        // async method self with preserve class
        let method: ImplItemFn = parse_quote!(
            pub async fn some_name(&self, (arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Method(&method),
            preserve_js_class: true,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({
            use std::str::FromStr;
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = self.some_name((arg1, arg2)).await.into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_body_method_sync() {
        // sync method static
        let method: ImplItemFn = parse_quote!(
            pub fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Method(&method),
            preserve_js_class: false,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({ Self::some_name((arg1, arg2)).into() });
        assert_eq!(result, expected);

        // sync method with self
        let method: ImplItemFn = parse_quote!(
            pub fn some_name(&self, (arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Method(&method),
            preserve_js_class: false,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({ self.some_name((arg1, arg2)).into() });
        assert_eq!(result, expected);

        // sync method static with preserve class
        let method: ImplItemFn = parse_quote!(
            pub fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Method(&method),
            preserve_js_class: true,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({
            use std::str::FromStr;
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = Self::some_name((arg1, arg2)).into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);

        // sync method self with preserve class
        let method: ImplItemFn = parse_quote!(
            pub fn some_name(&self, (arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Method(&method),
            preserve_js_class: true,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({
            use std::str::FromStr;
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = self.some_name((arg1, arg2)).into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_body_standalone_function_async() {
        // async function
        let function: ItemFn = parse_quote!(
            pub async fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Standalone(&function),
            preserve_js_class: false,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({ some_name((arg1, arg2)).await.into() });
        assert_eq!(result, expected);

        // async function with preserve class
        let function: ItemFn = parse_quote!(
            pub async fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Standalone(&function),
            preserve_js_class: true,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({
            use std::str::FromStr;
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = some_name((arg1, arg2)).await.into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_body_standalone_function_sync() {
        // sync function
        let function: ItemFn = parse_quote!(
            pub fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Standalone(&function),
            preserve_js_class: false,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({ some_name((arg1, arg2)).into() });
        assert_eq!(result, expected);

        // sync function with preserve class
        let function: ItemFn = parse_quote!(
            pub fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let context = BuildExportFunctionBodyContext {
            function_type: FunctionType::Standalone(&function),
            preserve_js_class: true,
        };
        let result = build_export_function_body(context);
        let expected: Block = parse_quote!({
            use std::str::FromStr;
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = some_name((arg1, arg2)).into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_collect_function_arguments() {
        // without self argument
        let stream = TokenStream::from_str(r#"arg1: String, arg2: u8"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let result = collect_function_arguments(&inputs);
        let expected = (
            false,
            vec![
                TokenStream::from_str(r#"arg1"#).unwrap(),
                TokenStream::from_str(r#"arg2"#).unwrap(),
            ],
        );
        assert_eq!(result.0, expected.0);
        assert_eq!(result.1.len(), expected.1.len());
        assert!(result
            .1
            .iter()
            .zip(expected.1.iter())
            .all(|(res, exp)| { res.to_string() == exp.to_string() }));

        // with self argument
        let stream = TokenStream::from_str(r#"&self, arg1: String"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let result = collect_function_arguments(&inputs);
        let expected = (true, vec![TokenStream::from_str(r#"arg1"#).unwrap()]);
        assert_eq!(result.0, expected.0);
        assert_eq!(result.1.len(), expected.1.len());
        assert!(result
            .1
            .iter()
            .zip(expected.1.iter())
            .all(|(res, exp)| { res.to_string() == exp.to_string() }));
    }

    #[test]
    fn test_try_extract_result_inner_type_happy() {
        let output: ReturnType = parse_quote!(-> Result<SomeType, Error>);
        let result = try_extract_result_inner_type(&output).unwrap();
        let expected: Type = parse_quote!(SomeType);
        assert_eq!(*result, expected);

        let output: ReturnType = parse_quote!(-> Result<(), Error>);
        let result = try_extract_result_inner_type(&output).unwrap();
        let expected: Type = parse_quote!(());
        assert_eq!(*result, expected);
    }

    #[test]
    fn test_try_extract_result_inner_type_unhappy() {
        let output: ReturnType = parse_quote!(-> SomeType);
        assert!(try_extract_result_inner_type(&output).is_none());

        let output: ReturnType = parse_quote!(-> Option<SomeType>);
        assert!(try_extract_result_inner_type(&output).is_none());

        let output: ReturnType = parse_quote!(-> ());
        assert!(try_extract_result_inner_type(&output).is_none());

        let output: ReturnType = parse_quote!();
        assert!(try_extract_result_inner_type(&output).is_none());
    }

    #[test]
    fn test_populate_name() {
        let org_fn_ident = Ident::new("some_name", Span::call_site());
        let result = populate_name(&org_fn_ident);
        assert_eq!(result.to_string(), "some_name__wasm_export");
    }
}
