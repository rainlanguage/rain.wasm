use quote::quote;
use std::ops::Deref;
use proc_macro2::TokenStream;
use syn::{
    punctuated::Punctuated, token::Comma, Block, Error, FnArg, Ident, Path, PathSegment,
    ReturnType, Type, TypePath,
};

/// Creates a function call expression based on whether it's an instance or static method
pub fn create_function_call(
    fn_name: &Ident,
    inputs: &Punctuated<FnArg, Comma>,
    is_async: bool,
) -> Block {
    let (has_self_receiver, args) = collect_function_arguments(inputs);
    let call_expr = if has_self_receiver {
        // Instance method call
        quote! { self.#fn_name(#(#args),*) }
    } else {
        // Static method call
        quote! { Self::#fn_name(#(#args),*) }
    };

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
