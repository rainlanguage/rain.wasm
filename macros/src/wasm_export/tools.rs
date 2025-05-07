use quote::quote;
use std::ops::Deref;
use proc_macro2::TokenStream;
use syn::{
    punctuated::Punctuated, token::Comma, Block, Error, FnArg, Ident, Path, PathSegment,
    ReturnType, Type, TypePath,
};

/// Enum to specify the context of the function call
pub enum FunctionContext {
    /// Function is a method within an impl block (can be instance or static)
    Method,
    /// Function is a standalone function (outside any impl block)
    Standalone,
}

/// Creates a function call expression based on the context (method or standalone)
/// and whether it's async or takes a 'self' receiver.
pub fn create_function_call_unified(
    fn_name: &Ident,
    inputs: &Punctuated<FnArg, Comma>,
    is_async: bool,
    context: FunctionContext,
) -> Block {
    let (has_self_receiver, args) = collect_function_arguments(inputs);

    let call_expr = match context {
        FunctionContext::Method => {
            if has_self_receiver {
                // Instance method call: self.method_name(...)
                quote! { self.#fn_name(#(#args),*) }
            } else {
                // Static method call: Self::method_name(...)
                quote! { Self::#fn_name(#(#args),*) }
            }
        }
        FunctionContext::Standalone => {
            if has_self_receiver {
                return syn::parse_quote!({
                    compile_error!("Standalone functions cannot have a 'self' receiver")
                });
            }
            quote! { #fn_name(#(#args),*) }
        }
    };

    // Append .await if the function is async, and .into() in all cases
    if is_async {
        syn::parse_quote!({
            #call_expr.await.into()
        })
    } else {
        syn::parse_quote!({
            #call_expr.into()
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
            }) = segments.first()
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

/// Extends the original syn error msg with the given msg
pub fn extend_err_msg(msg: &str) -> impl Fn(Error) -> Error + '_ {
    |err| Error::new(err.span(), err.to_string() + msg)
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
    fn test_create_function_call_unified_static_method_async() {
        let stream = TokenStream::from_str(r#"(arg1, arg2): (String, u8)"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let fn_name = Ident::new("some_name", Span::call_site());
        let is_async = true;
        let result =
            create_function_call_unified(&fn_name, &inputs, is_async, FunctionContext::Method);
        // Expected: Self::some_name(arg1, arg2).await.into() -> Note: parse_quote! needs parentheses around tuple args
        let expected: Block = parse_quote!({ Self::some_name((arg1, arg2)).await.into() });
        assert_eq!(format!("{:?}", result), format!("{:?}", expected)); // Compare string representation for complex Block types
    }

    #[test]
    fn test_create_function_call_unified_instance_method_sync() {
        let stream = TokenStream::from_str(r#"&self, arg1: String, arg2: u8"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let fn_name = Ident::new("some_name", Span::call_site());
        let is_async = false;
        let result =
            create_function_call_unified(&fn_name, &inputs, is_async, FunctionContext::Method);
        let expected: Block = parse_quote!({ self.some_name(arg1, arg2).into() });
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_create_function_call_unified_standalone_sync() {
        let stream = TokenStream::from_str(r#"arg1: String, arg2: u8"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let fn_name = Ident::new("some_standalone_fn", Span::call_site());
        let is_async = false;
        let result =
            create_function_call_unified(&fn_name, &inputs, is_async, FunctionContext::Standalone);
        let expected: Block = parse_quote!({ some_standalone_fn(arg1, arg2).into() });
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_create_function_call_unified_standalone_async() {
        let stream = TokenStream::from_str(r#"arg1: i32"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let fn_name = Ident::new("another_standalone", Span::call_site());
        let is_async = true;
        let result =
            create_function_call_unified(&fn_name, &inputs, is_async, FunctionContext::Standalone);
        let expected: Block = parse_quote!({ another_standalone(arg1).await.into() });
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_create_function_call_unified_standalone_with_self_error() {
        // This test verifies that providing a 'self' receiver with Standalone context triggers compile error
        let stream = TokenStream::from_str(r#"&self, arg1: String"#).unwrap();
        let inputs = Punctuated::<FnArg, Comma>::parse_terminated
            .parse2(stream)
            .unwrap();
        let fn_name = Ident::new("standalone_error_fn", Span::call_site());
        let is_async = false;
        let result =
            create_function_call_unified(&fn_name, &inputs, is_async, FunctionContext::Standalone);
        let expected: Block =
            parse_quote!({ compile_error!("Standalone functions cannot have a 'self' receiver") });
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
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
    fn test_extend_error() {
        let err = Error::new(Span::call_site(), "some msg");
        let result = extend_err_msg(", extend msg")(err);
        assert_eq!(result.to_string(), "some msg, extend msg");
    }

    #[test]
    fn test_populate_name() {
        let org_fn_ident = Ident::new("some_name", Span::call_site());
        let result = populate_name(&org_fn_ident);
        assert_eq!(result.to_string(), "some_name__wasm_export");
    }
}
