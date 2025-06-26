use quote::quote;
use std::ops::Deref;
use proc_macro2::{Span, TokenStream};
use syn::{punctuated::Punctuated, token::Comma, Block, FnArg, Ident, ImplItemFn, ItemFn, Meta, Type};

/// Enum to specify whether a function has a self receiver parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfReceiver {
    /// Function has a self receiver (&self, &mut self, self)
    Instance,
    /// Function has no self receiver (static method or standalone function)
    Static,
}

/// Type alias for the complex return type of process_function_parameters
type ProcessFunctionParametersResult = syn::Result<(
    SelfReceiver,
    Punctuated<FnArg, Comma>,
    Punctuated<FnArg, Comma>,
)>;

/// Enum to specify the type of the function
pub enum FunctionType<'a> {
    /// Function is a method within an impl block (can be instance or static)
    Method(&'a ImplItemFn),
    /// Function is a standalone (outside any impl block)
    Standalone(&'a ItemFn),
}

/// Configuration for building a wasm export function
pub struct WasmExportFunctionBuilderConfig {
    pub forward_attrs: Vec<Meta>,
    pub return_type: Type,
    pub preserve_js_class: Option<Span>,
}

/// Provides functionalities to build methods/functions that are going to be exposed to wasm bindgen output
pub struct WasmExportFunctionBuilder;
impl WasmExportFunctionBuilder {
    /// Builds a wasm export method from the given method and configurations, that is,
    /// creating a new method that is exposed to wasm bindgen that calls the original
    /// method and converting the result of that call into a WasmEncodedResult and also
    /// forwards any wasm bindgen attributes to the exporting method
    pub fn build_export_method(
        method: &ImplItemFn,
        config: WasmExportFunctionBuilderConfig,
    ) -> syn::Result<ImplItemFn> {
        let WasmExportFunctionBuilderConfig {
            forward_attrs,
            return_type,
            preserve_js_class,
        } = config;

        // create exported method from original
        let mut export_method = method.clone();

        // set exported method name, it is appended with __wasm_export
        export_method.sig.ident = Self::populate_name(&method.sig.ident);

        // process parameters to handle wasm_export attributes
        let (_, processed_params, _) = Self::process_function_parameters(&method.sig.inputs)?;
        export_method.sig.inputs = processed_params;

        // extract doc comments from original method
        let doc_comments = Self::extract_doc_comments(&method.attrs);

        // forward attributes for exported method + allow none snake_case + doc comments
        export_method.attrs = Vec::new();
        export_method.attrs.extend(doc_comments);
        export_method
            .attrs
            .push(syn::parse_quote!(#[allow(non_snake_case)]));
        if !forward_attrs.is_empty() {
            export_method.attrs.push(syn::parse_quote!(
                #[wasm_bindgen(#(#forward_attrs),*)]
            ));
        }

        // set exported method return type as JsValue if
        // preserve_js_class is true else set it to WasmEncodedResult
        if preserve_js_class.is_some() {
            export_method.sig.output = syn::parse_quote!(-> JsValue);
        } else {
            export_method.sig.output = syn::parse_quote!(-> WasmEncodedResult<#return_type>);
        }

        // build the method body by calling the original method
        export_method.block =
            Self::build_fn_body_unified(FunctionType::Method(method), preserve_js_class.is_some());

        Ok(export_method)
    }

    /// Builds a wasm export standalone function from the given function and configurations,
    /// that is, creating a new function that is exposed to wasm bindgen that calls the original
    /// function and converting the result of that call into a WasmEncodedResult and also forwards
    /// any wasm bindgen attributes to the exporting function
    pub fn build_export_function(
        func: &ItemFn,
        config: WasmExportFunctionBuilderConfig,
    ) -> syn::Result<ItemFn> {
        let WasmExportFunctionBuilderConfig {
            forward_attrs,
            return_type,
            preserve_js_class,
        } = config;

        // create the export function from original
        let mut export_fn = func.clone();

        // set exported function name, it is appended with __wasm_export
        export_fn.sig.ident = Self::populate_name(&func.sig.ident);

        // process parameters to handle wasm_export attributes
        let (_, processed_params, _) = Self::process_function_parameters(&func.sig.inputs)?;
        export_fn.sig.inputs = processed_params;

        // extract doc comments from original function
        let doc_comments = Self::extract_doc_comments(&func.attrs);

        // forward attributes for exported function + allow none snake_case + doc comments
        export_fn.attrs = Vec::new();
        export_fn.attrs.extend(doc_comments);
        export_fn
            .attrs
            .push(syn::parse_quote!(#[allow(non_snake_case)]));
        if !forward_attrs.is_empty() {
            export_fn.attrs.push(syn::parse_quote!(
                #[wasm_bindgen(#(#forward_attrs),*)]
            ));
        } else {
            // Add wasm_bindgen even if no specific attrs were forwarded
            export_fn.attrs.push(syn::parse_quote!(#[wasm_bindgen]));
        }

        // set exported function return type as JsValue if
        // preserve_js_class is true else set it to WasmEncodedResult
        if preserve_js_class.is_some() {
            export_fn.sig.output = syn::parse_quote!(-> JsValue);
        } else {
            export_fn.sig.output = syn::parse_quote!(-> WasmEncodedResult<#return_type>);
        }

        // build the function body by calling the original function
        export_fn.block = Box::new(Self::build_fn_body_unified(
            FunctionType::Standalone(func),
            preserve_js_class.is_some(),
        ));

        Ok(export_fn)
    }

    /// Creates a function call expression (export function/method body) based on the given context (method or standalone)
    pub fn build_fn_body_unified(function_type: FunctionType, preserve_js_class: bool) -> Block {
        // build the base call_expr based on the function type
        let (call_expr, is_async) = match function_type {
            FunctionType::Method(method) => {
                let fn_name = &method.sig.ident;
                let (self_receiver, args) = Self::collect_function_arguments(&method.sig.inputs);

                let call_expr = match self_receiver {
                    SelfReceiver::Instance => {
                        // instance method call: self.method_name(...)
                        quote! { self.#fn_name(#(#args),*) }
                    }
                    SelfReceiver::Static => {
                        // static method call: Self::method_name(...)
                        quote! { Self::#fn_name(#(#args),*) }
                    }
                };

                // return base call expression and asyncness
                (call_expr, method.sig.asyncness.is_some())
            }
            FunctionType::Standalone(function) => {
                let fn_name = &function.sig.ident;
                let (_, args) = Self::collect_function_arguments(&function.sig.inputs);

                // return base call expression and asyncness
                (
                    quote! { #fn_name(#(#args),*) },
                    function.sig.asyncness.is_some(),
                )
            }
        };

        // append .await if the function is async, and .into() in
        // all cases to convert rust Result to WasmEncodedResult
        let call_expr = if is_async {
            quote!( #call_expr.await.into() )
        } else {
            quote!( #call_expr.into() )
        };

        // manually build a js obj that resembles the WasmEncodedResult to preserve
        // the class if preserve_js_class attr was detected and return it as JsValue
        // otherwise return the call expression unchanged
        if preserve_js_class {
            syn::parse_quote!({
                // bring necessary items in scope
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
                let result = #call_expr;
                match result {
                    Ok(value) => {
                        Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                        Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                    }
                    Err(error) => {
                        let wasm_error: WasmEncodedError = error.into();
                        Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                        Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
                    }
                };

                // return as JsValue
                obj.into()
            })
        } else {
            // parses the call expression token stream to syn::Block
            syn::parse_quote!({
                #call_expr
            })
        }
    }

    /// Collects function arguments and determines if the function has a self receiver
    pub fn collect_function_arguments(
        inputs: &Punctuated<FnArg, Comma>,
    ) -> (SelfReceiver, Vec<TokenStream>) {
        let mut self_receiver = SelfReceiver::Static;
        let args = inputs
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Receiver(_) => {
                    self_receiver = SelfReceiver::Instance;
                    None
                }
                FnArg::Typed(pat_type) => {
                    let pat = pat_type.pat.deref();
                    Some(quote! { #pat })
                }
            })
            .collect();

        (self_receiver, args)
    }

    /// Processes function parameters by extracting wasm_export attributes and converting them to wasm_bindgen
    /// Returns: (self_receiver, processed_inputs_for_wrapper, cleaned_inputs_for_original)
    pub fn process_function_parameters(
        inputs: &Punctuated<FnArg, Comma>,
    ) -> ProcessFunctionParametersResult {
        let mut self_receiver = SelfReceiver::Static;
        let mut processed_inputs = Punctuated::new();
        let mut cleaned_inputs = Punctuated::new();

        for input in inputs {
            match input {
                FnArg::Receiver(receiver) => {
                    self_receiver = SelfReceiver::Instance;

                    // Check if receiver has any wasm_export attributes - this should be an error
                    for attr in &receiver.attrs {
                        if attr.path().is_ident("wasm_export") {
                            return Err(syn::Error::new_spanned(
                                attr,
                                "`param_description` cannot be used on receiver parameters (self, &self, &mut self)"
                            ));
                        }
                    }

                    processed_inputs.push(input.clone());
                    cleaned_inputs.push(input.clone());
                }
                FnArg::Typed(pat_type) => {
                    let mut new_pat_type = pat_type.clone();
                    let mut cleaned_pat_type = pat_type.clone();

                    // Process attributes on this parameter
                    let mut wasm_bindgen_attrs = Vec::new();
                    let mut other_attrs = Vec::new();

                    for attr in &pat_type.attrs {
                        if attr.path().is_ident("wasm_export") {
                            // Parse wasm_export attribute and convert to wasm_bindgen
                            let processed_attrs = Self::process_parameter_wasm_export_attr(attr)?;
                            wasm_bindgen_attrs.extend(processed_attrs);
                            // Don't include wasm_export attrs in cleaned version
                        } else {
                            other_attrs.push(attr.clone());
                        }
                    }

                    // For wrapper function: combine processed wasm_bindgen attrs with other attrs
                    new_pat_type.attrs.clone_from(&other_attrs);
                    if !wasm_bindgen_attrs.is_empty() {
                        new_pat_type
                            .attrs
                            .push(syn::parse_quote!(#[wasm_bindgen(#(#wasm_bindgen_attrs),*)]));
                    }

                    // For original function: only other attrs (wasm_export removed)
                    cleaned_pat_type.attrs = other_attrs;

                    processed_inputs.push(FnArg::Typed(new_pat_type));
                    cleaned_inputs.push(FnArg::Typed(cleaned_pat_type));
                }
            }
        }

        Ok((self_receiver, processed_inputs, cleaned_inputs))
    }

    /// Processes a single wasm_export attribute on a parameter and converts it to wasm_bindgen format
    fn process_parameter_wasm_export_attr(attr: &syn::Attribute) -> syn::Result<Vec<syn::Meta>> {
        use syn::{punctuated::Punctuated, token::Comma, Meta};
        use super::error::extend_err_msg;

        let mut wasm_bindgen_metas = Vec::new();
        let mut seen_param_description = false;

        // Handle empty wasm_export attribute
        if matches!(attr.meta, Meta::Path(_)) {
            return Ok(wasm_bindgen_metas);
        }

        // Parse the attribute contents
        let nested_metas = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)?;

        for meta in nested_metas {
            if let Some(ident) = meta.path().get_ident() {
                if ident == "param_description" {
                    // Check for duplicate param_description
                    if seen_param_description {
                        return Err(syn::Error::new_spanned(
                            meta,
                            "duplicate `param_description` attribute",
                        ));
                    }
                    seen_param_description = true;

                    // Validate that it has a string literal value
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(_),
                        ..
                    }) = &meta
                        .require_name_value()
                        .map_err(extend_err_msg(" and it must be a string literal"))?
                        .value
                    {
                        // Convert param_description from wasm_export to wasm_bindgen format
                        wasm_bindgen_metas.push(meta);
                    } else {
                        return Err(syn::Error::new_spanned(meta, "expected string literal"));
                    }
                }
                // Add support for other parameter-level attributes here if needed
            }
        }

        Ok(wasm_bindgen_metas)
    }

    /// Cleans wasm_export attributes from function parameters (for cleaning original functions)
    pub fn clean_parameter_attributes(inputs: &mut Punctuated<FnArg, Comma>) {
        for input in inputs.iter_mut() {
            if let FnArg::Typed(pat_type) = input {
                pat_type
                    .attrs
                    .retain(|attr| !attr.path().is_ident("wasm_export"));
            }
        }
    }

    /// Extracts doc comments (/// ...) from function/method attributes
    pub fn extract_doc_comments(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
        attrs
            .iter()
            .filter(|attr| {
                // Check if attribute is a doc comment (starts with "doc")
                attr.path().is_ident("doc")
            })
            .cloned()
            .collect()
    }

    /// Creates the function name from the original name, it is appended by __wasm_export
    pub fn populate_name(org_fn_ident: &Ident) -> Ident {
        Ident::new(
            &format!("{}__wasm_export", org_fn_ident),
            org_fn_ident.span(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use proc_macro2::{Span, TokenStream};
    use syn::{parse::Parser, parse_quote};
    use quote::ToTokens;

    #[test]
    fn test_from_method() {
        // without preserve js class
        let method: ImplItemFn = parse_quote!(
            pub async fn some_fn(arg1: String) -> Result<SomeType, Error> {}
        );
        let wasm_export_fn_config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![parse_quote!(some_forward_attr)],
            return_type: parse_quote!(SomeType),
            preserve_js_class: None,
        };
        let result =
            WasmExportFunctionBuilder::build_export_method(&method, wasm_export_fn_config).unwrap();
        let expected = parse_quote!(
            #[allow(non_snake_case)]
            #[wasm_bindgen(some_forward_attr)]
            pub async fn some_fn__wasm_export(arg1: String) -> WasmEncodedResult<SomeType> {
                Self::some_fn(arg1).await.into()
            }
        );
        assert_eq!(result, expected);

        // with preserve js class
        let method: ImplItemFn = parse_quote!(
            pub async fn some_fn(arg1: String) -> Result<SomeType, Error> {}
        );
        let wasm_export_fn_config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![parse_quote!(some_forward_attr)],
            return_type: parse_quote!(SomeType),
            preserve_js_class: Some(Span::call_site()),
        };
        let result =
            WasmExportFunctionBuilder::build_export_method(&method, wasm_export_fn_config).unwrap();
        let expected = parse_quote!(
            #[allow(non_snake_case)]
            #[wasm_bindgen(some_forward_attr)]
            pub async fn some_fn__wasm_export(arg1: String) -> JsValue {
                use js_sys::{Reflect, Object};
                let obj = Object::new();
                let result = Self::some_fn(arg1).await.into();
                match result {
                    Ok(value) => {
                        Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                        Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED)
                            .unwrap();
                    }
                    Err(error) => {
                        let wasm_error: WasmEncodedError = error.into();
                        Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED)
                            .unwrap();
                        Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into())
                            .unwrap();
                    }
                };
                obj.into()
            }
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_standalone() {
        // without preserve js class
        let func: ItemFn = parse_quote!(
            pub async fn some_fn(arg1: String) -> Result<SomeType, Error> {}
        );
        let wasm_export_fn_config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![parse_quote!(some_forward_attr)],
            return_type: parse_quote!(SomeType),
            preserve_js_class: None,
        };
        let result =
            WasmExportFunctionBuilder::build_export_function(&func, wasm_export_fn_config).unwrap();
        let expected = parse_quote!(
            #[allow(non_snake_case)]
            #[wasm_bindgen(some_forward_attr)]
            pub async fn some_fn__wasm_export(arg1: String) -> WasmEncodedResult<SomeType> {
                some_fn(arg1).await.into()
            }
        );
        assert_eq!(result, expected);

        // with preserve js class
        let func: ItemFn = parse_quote!(
            pub async fn some_fn(arg1: String) -> Result<SomeType, Error> {}
        );
        let wasm_export_fn_config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![parse_quote!(some_forward_attr)],
            return_type: parse_quote!(SomeType),
            preserve_js_class: Some(Span::call_site()),
        };
        let result =
            WasmExportFunctionBuilder::build_export_function(&func, wasm_export_fn_config).unwrap();
        let expected = parse_quote!(
            #[allow(non_snake_case)]
            #[wasm_bindgen(some_forward_attr)]
            pub async fn some_fn__wasm_export(arg1: String) -> JsValue {
                use js_sys::{Reflect, Object};
                let obj = Object::new();
                let result = some_fn(arg1).await.into();
                match result {
                    Ok(value) => {
                        Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                        Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED)
                            .unwrap();
                    }
                    Err(error) => {
                        let wasm_error: WasmEncodedError = error.into();
                        Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED)
                            .unwrap();
                        Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into())
                            .unwrap();
                    }
                };
                obj.into()
            }
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_fn_body_unified_method_async() {
        // async method static
        let method: ImplItemFn = parse_quote!(
            pub async fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result =
            WasmExportFunctionBuilder::build_fn_body_unified(FunctionType::Method(&method), false);
        let expected: Block = parse_quote!({ Self::some_name((arg1, arg2)).await.into() });
        assert_eq!(result, expected);

        // async method with self
        let method: ImplItemFn = parse_quote!(
            pub async fn some_name(&self, (arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result =
            WasmExportFunctionBuilder::build_fn_body_unified(FunctionType::Method(&method), false);
        let expected: Block = parse_quote!({ self.some_name((arg1, arg2)).await.into() });
        assert_eq!(result, expected);

        // async method static with preserve class
        let method: ImplItemFn = parse_quote!(
            pub async fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result =
            WasmExportFunctionBuilder::build_fn_body_unified(FunctionType::Method(&method), true);
        let expected: Block = parse_quote!({
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = Self::some_name((arg1, arg2)).await.into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    let wasm_error: WasmEncodedError = error.into();
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
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
        let result =
            WasmExportFunctionBuilder::build_fn_body_unified(FunctionType::Method(&method), true);
        let expected: Block = parse_quote!({
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = self.some_name((arg1, arg2)).await.into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    let wasm_error: WasmEncodedError = error.into();
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_fn_body_unified_method_sync() {
        // sync method static
        let method: ImplItemFn = parse_quote!(
            pub fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result =
            WasmExportFunctionBuilder::build_fn_body_unified(FunctionType::Method(&method), false);
        let expected: Block = parse_quote!({ Self::some_name((arg1, arg2)).into() });
        assert_eq!(result, expected);

        // sync method with self
        let method: ImplItemFn = parse_quote!(
            pub fn some_name(&self, (arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result =
            WasmExportFunctionBuilder::build_fn_body_unified(FunctionType::Method(&method), false);
        let expected: Block = parse_quote!({ self.some_name((arg1, arg2)).into() });
        assert_eq!(result, expected);

        // sync method static with preserve class
        let method: ImplItemFn = parse_quote!(
            pub fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result =
            WasmExportFunctionBuilder::build_fn_body_unified(FunctionType::Method(&method), true);
        let expected: Block = parse_quote!({
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = Self::some_name((arg1, arg2)).into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    let wasm_error: WasmEncodedError = error.into();
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
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
        let result =
            WasmExportFunctionBuilder::build_fn_body_unified(FunctionType::Method(&method), true);
        let expected: Block = parse_quote!({
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = self.some_name((arg1, arg2)).into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    let wasm_error: WasmEncodedError = error.into();
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_fn_body_unified_standalone_function_async() {
        // async function
        let function: ItemFn = parse_quote!(
            pub async fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = WasmExportFunctionBuilder::build_fn_body_unified(
            FunctionType::Standalone(&function),
            false,
        );
        let expected: Block = parse_quote!({ some_name((arg1, arg2)).await.into() });
        assert_eq!(result, expected);

        // async function with preserve class
        let function: ItemFn = parse_quote!(
            pub async fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = WasmExportFunctionBuilder::build_fn_body_unified(
            FunctionType::Standalone(&function),
            true,
        );
        let expected: Block = parse_quote!({
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = some_name((arg1, arg2)).await.into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    let wasm_error: WasmEncodedError = error.into();
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
                }
            };
            obj.into()
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_fn_body_unified_standalone_function_sync() {
        // sync function
        let function: ItemFn = parse_quote!(
            pub fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = WasmExportFunctionBuilder::build_fn_body_unified(
            FunctionType::Standalone(&function),
            false,
        );
        let expected: Block = parse_quote!({ some_name((arg1, arg2)).into() });
        assert_eq!(result, expected);

        // sync function with preserve class
        let function: ItemFn = parse_quote!(
            pub fn some_name((arg1, arg2): (String, u8)) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = WasmExportFunctionBuilder::build_fn_body_unified(
            FunctionType::Standalone(&function),
            true,
        );
        let expected: Block = parse_quote!({
            use js_sys::{Reflect, Object};
            let obj = Object::new();
            let result = some_name((arg1, arg2)).into();
            match result {
                Ok(value) => {
                    Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED).unwrap();
                }
                Err(error) => {
                    let wasm_error: WasmEncodedError = error.into();
                    Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
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
        let result = WasmExportFunctionBuilder::collect_function_arguments(&inputs);
        let expected = (
            SelfReceiver::Static,
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
        let result = WasmExportFunctionBuilder::collect_function_arguments(&inputs);
        let expected = (
            SelfReceiver::Instance,
            vec![TokenStream::from_str(r#"arg1"#).unwrap()],
        );
        assert_eq!(result.0, expected.0);
        assert_eq!(result.1.len(), expected.1.len());
        assert!(result
            .1
            .iter()
            .zip(expected.1.iter())
            .all(|(res, exp)| { res.to_string() == exp.to_string() }));
    }

    #[test]
    fn test_populate_name() {
        let org_fn_ident = Ident::new("some_name", Span::call_site());
        let result = WasmExportFunctionBuilder::populate_name(&org_fn_ident);
        assert_eq!(result.to_string(), "some_name__wasm_export");
    }

    #[test]
    fn test_process_function_parameters_basic() {
        // Test basic parameter processing without wasm_export attributes
        let stream = TokenStream::from_str(r#"arg1: String, arg2: u32"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let result = WasmExportFunctionBuilder::process_function_parameters(&inputs).unwrap();

        assert_eq!(result.0, SelfReceiver::Static); // no self receiver
        assert_eq!(result.1.len(), 2); // processed inputs
        assert_eq!(result.2.len(), 2); // cleaned inputs

        // Both processed and cleaned should be identical when no wasm_export attrs
        assert_eq!(result.1.len(), result.2.len());
        // Check that first parameter is the same
        if let (FnArg::Typed(processed), FnArg::Typed(cleaned)) = (&result.1[0], &result.2[0]) {
            assert_eq!(processed.attrs.len(), cleaned.attrs.len());
            assert_eq!(
                processed.pat.to_token_stream().to_string(),
                cleaned.pat.to_token_stream().to_string()
            );
        }
    }

    #[test]
    fn test_process_function_parameters_with_param_description() {
        // Test parameter processing with param_description
        let stream = TokenStream::from_str(
            r#"#[wasm_export(param_description = "first param")] arg1: String, arg2: u32"#,
        )
        .unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let result = WasmExportFunctionBuilder::process_function_parameters(&inputs).unwrap();

        assert_eq!(result.0, SelfReceiver::Static); // no self receiver
        assert_eq!(result.1.len(), 2); // processed inputs
        assert_eq!(result.2.len(), 2); // cleaned inputs

        // Processed should have wasm_bindgen attribute, cleaned should not
        let processed_first = &result.1[0];
        let cleaned_first = &result.2[0];

        if let (FnArg::Typed(processed_pat), FnArg::Typed(cleaned_pat)) =
            (processed_first, cleaned_first)
        {
            // Processed should have wasm_bindgen attribute
            assert!(processed_pat
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("wasm_bindgen")));
            // Cleaned should not have any attributes
            assert!(cleaned_pat.attrs.is_empty());
        } else {
            panic!("Expected FnArg::Typed");
        }
    }

    #[test]
    fn test_process_function_parameters_with_self() {
        // Test parameter processing with self receiver
        let stream = TokenStream::from_str(r#"&self, arg1: String"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let result = WasmExportFunctionBuilder::process_function_parameters(&inputs).unwrap();

        assert_eq!(result.0, SelfReceiver::Instance); // has self receiver
        assert_eq!(result.1.len(), 2); // processed inputs (self + arg1)
        assert_eq!(result.2.len(), 2); // cleaned inputs (self + arg1)
    }

    #[test]
    fn test_process_function_parameters_self_with_wasm_export_error() {
        // Test that wasm_export on self receiver produces error
        let stream = TokenStream::from_str(
            r#"#[wasm_export(param_description = "self desc")] &self, arg1: String"#,
        )
        .unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let result = WasmExportFunctionBuilder::process_function_parameters(&inputs);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("param_description` cannot be used on receiver parameters"));
    }

    #[test]
    fn test_process_parameter_wasm_export_attr_valid() {
        // Test valid param_description processing
        let attr: syn::Attribute =
            syn::parse_quote!(#[wasm_export(param_description = "test description")]);
        let result = WasmExportFunctionBuilder::process_parameter_wasm_export_attr(&attr).unwrap();

        assert_eq!(result.len(), 1);
        let meta = &result[0];
        assert!(meta.path().is_ident("param_description"));
    }

    #[test]
    fn test_process_parameter_wasm_export_attr_duplicate_error() {
        // Test duplicate param_description error
        let attr: syn::Attribute = syn::parse_quote!(
            #[wasm_export(param_description = "first", param_description = "second")]
        );
        let result = WasmExportFunctionBuilder::process_parameter_wasm_export_attr(&attr);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("duplicate `param_description` attribute"));
    }

    #[test]
    fn test_process_parameter_wasm_export_attr_invalid_value() {
        // Test invalid value type error
        let attr: syn::Attribute = syn::parse_quote!(#[wasm_export(param_description = something)]);
        let result = WasmExportFunctionBuilder::process_parameter_wasm_export_attr(&attr);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("expected string literal"));
    }

    #[test]
    fn test_process_parameter_wasm_export_attr_missing_value() {
        // Test missing value error
        let attr: syn::Attribute = syn::parse_quote!(#[wasm_export(param_description)]);
        let result = WasmExportFunctionBuilder::process_parameter_wasm_export_attr(&attr);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("expected a value for this attribute"));
    }

    #[test]
    fn test_clean_parameter_attributes() {
        // Test cleaning wasm_export attributes from parameters
        let stream = TokenStream::from_str(
            r#"#[wasm_export(param_description = "test")] #[other_attr] arg1: String, arg2: u32"#,
        )
        .unwrap();
        let mut inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();

        // Before cleaning - should have both attributes
        if let FnArg::Typed(pat_type) = &inputs[0] {
            assert_eq!(pat_type.attrs.len(), 2);
        }

        WasmExportFunctionBuilder::clean_parameter_attributes(&mut inputs);

        // After cleaning - should only have other_attr
        if let FnArg::Typed(pat_type) = &inputs[0] {
            assert_eq!(pat_type.attrs.len(), 1);
            assert!(pat_type.attrs[0].path().is_ident("other_attr"));
        }
    }

    #[test]
    fn test_extract_doc_comments() {
        // Test extracting doc comments from function attributes
        let func: ItemFn = parse_quote!(
            /// This is a single line doc comment
            /// This is another line
            #[other_attr]
            pub fn test_func() -> Result<(), Error> {
                Ok(())
            }
        );

        let doc_comments = WasmExportFunctionBuilder::extract_doc_comments(&func.attrs);
        assert_eq!(doc_comments.len(), 2);

        // Verify both are doc attributes
        for comment in &doc_comments {
            assert!(comment.path().is_ident("doc"));
        }
    }

    #[test]
    fn test_extract_doc_comments_no_docs() {
        // Test extracting doc comments when there are none
        let func: ItemFn = parse_quote!(
            #[other_attr]
            #[some_macro]
            pub fn test_func() -> Result<(), Error> {
                Ok(())
            }
        );

        let doc_comments = WasmExportFunctionBuilder::extract_doc_comments(&func.attrs);
        assert_eq!(doc_comments.len(), 0);
    }

    #[test]
    fn test_extract_doc_comments_mixed() {
        // Test extracting doc comments mixed with other attributes
        let func: ItemFn = parse_quote!(
            #[cfg(test)]
            /// First doc comment
            #[derive(Debug)]
            /// Second doc comment
            /// Third doc comment
            #[allow(dead_code)]
            pub fn test_func() -> Result<(), Error> {
                Ok(())
            }
        );

        let doc_comments = WasmExportFunctionBuilder::extract_doc_comments(&func.attrs);
        assert_eq!(doc_comments.len(), 3);

        // Verify all are doc attributes
        for comment in &doc_comments {
            assert!(comment.path().is_ident("doc"));
        }
    }

    #[test]
    fn test_build_export_method_with_doc_comments() {
        // Test that doc comments are forwarded to export method
        let method: ImplItemFn = parse_quote!(
            /// This method does something useful
            /// It takes a string and returns a result
            pub fn some_method(arg: String) -> Result<String, Error> {
                Ok(arg)
            }
        );
        let config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![parse_quote!(js_name = "someMethod")],
            return_type: parse_quote!(String),
            preserve_js_class: None,
        };

        let result = WasmExportFunctionBuilder::build_export_method(&method, config).unwrap();

        // Check that doc comments are present in export method
        let doc_count = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .count();
        assert_eq!(doc_count, 2);

        // Check that other attributes are also present
        assert!(result
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("allow")));
        assert!(result
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("wasm_bindgen")));
    }

    #[test]
    fn test_build_export_method_no_doc_comments() {
        // Test that method without doc comments works normally
        let method: ImplItemFn = parse_quote!(
            #[some_attr]
            pub fn some_method(arg: String) -> Result<String, Error> {
                Ok(arg)
            }
        );
        let config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![],
            return_type: parse_quote!(String),
            preserve_js_class: None,
        };

        let result = WasmExportFunctionBuilder::build_export_method(&method, config).unwrap();

        // Check that no doc comments are present
        let doc_count = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .count();
        assert_eq!(doc_count, 0);

        // Check that other attributes are still present
        assert!(result
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("allow")));
    }

    #[test]
    fn test_build_export_function_with_doc_comments() {
        // Test that doc comments are forwarded to export function
        let func: ItemFn = parse_quote!(
            /// This function adds two numbers
            /// # Examples
            /// ```
            /// let result = add(1, 2);
            /// ```
            pub fn add(a: u32, b: u32) -> Result<u32, Error> {
                Ok(a + b)
            }
        );
        let config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![parse_quote!(js_name = "add")],
            return_type: parse_quote!(u32),
            preserve_js_class: None,
        };

        let result = WasmExportFunctionBuilder::build_export_function(&func, config).unwrap();

        // Check that doc comments are present in export function
        let doc_count = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .count();
        assert_eq!(doc_count, 5); // 5 lines of doc comments (updated from incorrect 4)

        // Check that other attributes are also present
        assert!(result
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("allow")));
        assert!(result
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("wasm_bindgen")));
    }

    #[test]
    fn test_build_export_function_no_doc_comments() {
        // Test that function without doc comments works normally
        let func: ItemFn = parse_quote!(
            pub fn simple_func() -> Result<(), Error> {
                Ok(())
            }
        );
        let config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![],
            return_type: parse_quote!(()),
            preserve_js_class: None,
        };

        let result = WasmExportFunctionBuilder::build_export_function(&func, config).unwrap();

        // Check that no doc comments are present
        let doc_count = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .count();
        assert_eq!(doc_count, 0);

        // Check that other attributes are still present
        assert!(result
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("allow")));
        assert!(result
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("wasm_bindgen")));
    }

    #[test]
    fn test_doc_comments_with_preserve_js_class() {
        // Test that doc comments work with preserve_js_class attribute
        let func: ItemFn = parse_quote!(
            /// Returns a JS class instance
            /// This function demonstrates preserve_js_class behavior
            pub fn get_js_instance() -> Result<JsClass, Error> {
                Ok(JsClass::new())
            }
        );
        let config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![],
            return_type: parse_quote!(JsClass),
            preserve_js_class: Some(Span::call_site()),
        };

        let result = WasmExportFunctionBuilder::build_export_function(&func, config).unwrap();

        // Check that doc comments are present
        let doc_count = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .count();
        assert_eq!(doc_count, 2);

        // Check that return type is JsValue for preserve_js_class
        match &result.sig.output {
            syn::ReturnType::Type(_, ty) => {
                assert_eq!(quote!(#ty).to_string(), "JsValue");
            }
            _ => panic!("Expected return type"),
        }
    }

    #[test]
    fn test_doc_comments_with_mixed_attributes() {
        // Test doc comments combined with other wasm_export attributes
        let func: ItemFn = parse_quote!(
            /// Complex function with multiple attributes
            /// Demonstrates mixed usage of doc comments and wasm_export attributes  
            pub fn complex_func(input: String) -> Result<u32, Error> {
                Ok(42)
            }
        );
        let config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![
                parse_quote!(js_name = "complexFunction"),
                parse_quote!(catch),
                parse_quote!(return_description = "a magic number"),
            ],
            return_type: parse_quote!(u32),
            preserve_js_class: None,
        };

        let result = WasmExportFunctionBuilder::build_export_function(&func, config).unwrap();

        // Check that doc comments are present
        let doc_count = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .count();
        assert_eq!(doc_count, 2);

        // Check that wasm_bindgen attributes are present
        let wasm_bindgen_attrs: Vec<_> = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("wasm_bindgen"))
            .collect();
        assert_eq!(wasm_bindgen_attrs.len(), 1);

        // Check that the wasm_bindgen attribute contains all expected attributes
        let attr_tokens = quote!(#(#wasm_bindgen_attrs)*).to_string();
        assert!(attr_tokens.contains("js_name = \"complexFunction\""));
        assert!(attr_tokens.contains("catch"));
        assert!(attr_tokens.contains("return_description = \"a magic number\""));
    }

    #[test]
    fn test_doc_comments_special_characters() {
        // Test doc comments with special characters and formatting
        let func: ItemFn = parse_quote!(
            /// Function with `code`, **bold**, and *italic* markdown
            ///
            /// # Examples
            /// ```rust
            /// let result = special_func("test");
            /// assert_eq!(result.unwrap(), "processed");
            /// ```
            ///
            /// ## Notes
            /// - Supports Unicode: 🦀 Rust
            /// - Handles quotes: "double" and 'single'
            /// - Special chars: @#$%^&*()
            pub fn special_func(input: &str) -> Result<String, Error> {
                Ok(format!("processed: {}", input))
            }
        );
        let config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![],
            return_type: parse_quote!(String),
            preserve_js_class: None,
        };

        let result = WasmExportFunctionBuilder::build_export_function(&func, config).unwrap();

        // Check that all doc comment lines are preserved
        let doc_count = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .count();
        assert_eq!(doc_count, 12); // Should match the number of doc comment lines (updated count)

        // Verify that special characters are preserved in doc comments
        let doc_attrs: Vec<_> = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .collect();

        // Check that some special content is preserved
        let all_docs = quote!(#(#doc_attrs)*).to_string();
        assert!(all_docs.contains("🦀 Rust"));
        assert!(all_docs.contains("```rust"));
        assert!(all_docs.contains("@#$%^&*()"));
    }

    #[test]
    fn test_build_export_method_doc_comments_comprehensive() {
        // Test comprehensive doc comment scenarios for methods
        let method: ImplItemFn = parse_quote!(
            /// This method performs advanced calculations
            ///
            /// It takes multiple parameters and returns a complex result.
            /// The implementation uses sophisticated algorithms.
            ///
            /// # Parameters
            /// - `self`: The instance reference
            /// - `input`: The input value to process
            ///
            /// # Returns
            /// A Result containing the processed value or an error
            pub fn advanced_method(&self, input: u64) -> Result<ProcessedValue, Error> {
                Ok(ProcessedValue::new(input))
            }
        );
        let config = WasmExportFunctionBuilderConfig {
            forward_attrs: vec![parse_quote!(js_name = "advancedMethod")],
            return_type: parse_quote!(ProcessedValue),
            preserve_js_class: None,
        };

        let result = WasmExportFunctionBuilder::build_export_method(&method, config).unwrap();

        // Check that all doc comments are preserved
        let doc_count = result
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .count();
        assert_eq!(doc_count, 11); // Count the doc comment lines

        // Check method name transformation
        assert_eq!(result.sig.ident.to_string(), "advanced_method__wasm_export");

        // Check that js_name attribute is forwarded
        let wasm_bindgen_attr = result
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("wasm_bindgen"))
            .expect("wasm_bindgen attribute should be present");

        let attr_tokens = quote!(#wasm_bindgen_attr).to_string();
        assert!(attr_tokens.contains("js_name = \"advancedMethod\""));
    }
}
