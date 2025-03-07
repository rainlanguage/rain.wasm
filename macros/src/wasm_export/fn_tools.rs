use quote::quote;
use std::ops::Deref;
use proc_macro2::TokenStream;
use syn::{FnArg, Ident, ImplItemFn, Path, PathSegment, ReturnType, Type, TypePath};

/// Creates a function call expression based on whether it's an instance or static method
pub fn create_new_function_call(
    fn_name: &Ident,
    has_self_receiver: bool,
    args: &[TokenStream],
) -> TokenStream {
    if has_self_receiver {
        // Instance method call
        quote! { self.#fn_name(#(#args),*) }
    } else {
        // Static method call
        quote! { Self::#fn_name(#(#args),*) }
    }
}

/// Collects function arguments and determines if the function has a self receiver
pub fn collect_function_arguments(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> (bool, Vec<TokenStream>) {
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
pub fn try_extract_result_inner_type(method: &ImplItemFn) -> Option<&Type> {
    if let ReturnType::Type(_, return_type) = &method.sig.output {
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
