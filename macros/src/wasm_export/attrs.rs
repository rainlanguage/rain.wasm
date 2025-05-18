use std::ops::Deref;

use quote::ToTokens;
use proc_macro2::Span;
use super::{error::extend_err_msg};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Error, ImplItemFn, Meta, Path, PathSegment, ReturnType, Token, Type, TypePath,
};

/// Contains list of wasm_export macro attribute keys
pub struct AttrKeys;
impl AttrKeys {
    pub const SKIP: &'static str = "skip";
    pub const WASM_EXPORT: &'static str = "wasm_export";
    pub const PRESERVE_JS_CLASS: &'static str = "preserve_js_class";
    pub const UNCHECKED_RETURN_TYPE: &'static str = "unchecked_return_type";
}

/// Struct that holds the parsed wasm_export attributes details
#[derive(Debug, Clone, Default)]
pub struct WasmExportAttrs {
    pub forward_attrs: Vec<Meta>,
    pub unchecked_return_type: Option<(String, Span)>,
    pub should_skip: Option<Span>,
    pub preserve_js_class: Option<Span>,
}

impl Parse for WasmExportAttrs {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let mut wasm_export_attrs = WasmExportAttrs::default();
        if input.is_empty() {
            // return early if there are no attributes
            return Ok(wasm_export_attrs);
        }

        // process attributes sequence delimited by comma
        let attrs_seq = Punctuated::<Meta, Token![,]>::parse_terminated(input).map_err(
            extend_err_msg(" as wasm_export attributes must be delimited by comma"),
        )?;
        wasm_export_attrs.handle_attrs_sequence(attrs_seq)?;

        // skip cannot be used as top attributes since it is only
        // valid for skipping over methods inside of an impl block
        if let Some(span) = wasm_export_attrs.should_skip {
            return Err(Error::new(
                span,
                "unexpected `skip` attribute, it is only valid for methods of an impl block",
            ));
        }

        Ok(wasm_export_attrs)
    }
}

impl WasmExportAttrs {
    /// Processes the return type for the exporting function/method from the specified
    /// `unchecked_return_type` attr, falls back to original return inner type if not
    /// provided by `unchecked_return_type` attribute
    pub fn handle_return_type(&mut self, output: &ReturnType) -> Option<Type> {
        let return_type = Self::try_extract_result_inner_type(output).cloned();
        let as_str = return_type
            .as_ref()
            .map(|v| format!("{}", v.to_token_stream()));

        // handle return type attr for exporting item's wasm_bindgen macro invocation
        if let Some(v) = self
            .unchecked_return_type
            .as_ref()
            .map(|v| &v.0)
            .or(as_str.as_ref())
        {
            let return_type = format!("WasmEncodedResult<{}>", v);
            self.forward_attrs.push(syn::parse_quote!(
                unchecked_return_type = #return_type
            ));
        }

        return_type
    }

    /// Handles wasm_export specified sequence of attributes delimited by comma
    pub fn handle_attrs_sequence(&mut self, metas: Punctuated<Meta, Comma>) -> Result<(), Error> {
        for meta in metas {
            match meta.path().get_ident().map(ToString::to_string).as_deref() {
                Some(AttrKeys::UNCHECKED_RETURN_TYPE) => {
                    if self.unchecked_return_type.is_some() {
                        return Err(Error::new_spanned(
                            meta,
                            "duplicate `unchecked_return_type` attribute",
                        ));
                    } else if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(str),
                        ..
                    }) = &meta
                        .require_name_value()
                        .map_err(extend_err_msg(" and it must be a string literal"))?
                        .value
                    {
                        self.unchecked_return_type = Some((str.value(), meta.span()));
                    } else {
                        return Err(Error::new_spanned(meta, "expected string literal"));
                    }
                }
                Some(AttrKeys::SKIP) => {
                    if self.should_skip.is_some() {
                        return Err(Error::new_spanned(meta, "duplicate `skip` attribute"));
                    }
                    meta.require_path_only().map_err(extend_err_msg(
                        ", `skip` attribute does not take any extra tokens or arguments",
                    ))?;
                    self.should_skip = Some(meta.span());
                }
                Some(AttrKeys::PRESERVE_JS_CLASS) => {
                    if self.preserve_js_class.is_some() {
                        return Err(Error::new_spanned(
                            meta,
                            "duplicate `preserve_js_class` attribute",
                        ));
                    }
                    meta.require_path_only().map_err(extend_err_msg(
                        ", `preserve_js_class` attribute does not take any extra tokens or arguments",
                    ))?;
                    self.preserve_js_class = Some(meta.span());
                }
                _ => {
                    // include unchanged to be forwarded to the corresponding export item
                    self.forward_attrs.push(meta);
                }
            }
        }
        Ok(())
    }

    // Handles wasm_export macro attributes for a given impl method
    pub fn handle_method_attrs(method: &mut ImplItemFn) -> Result<Self, Error> {
        // start parsing nested attributes of this method
        let mut keep = Vec::new();
        let mut wasm_export_attrs = Self::default();
        for attr in method.attrs.iter_mut() {
            if attr.path().is_ident(AttrKeys::WASM_EXPORT) {
                // skip parsing by delimited comma if there are no nested attrs
                if !matches!(attr.meta, Meta::Path(_)) {
                    let nested_seq = attr
                        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                        .map_err(extend_err_msg(
                            " as wasm_export attributes must be delimited by comma",
                        ))?;
                    wasm_export_attrs.handle_attrs_sequence(nested_seq)?;
                }
                keep.push(false);
            } else {
                keep.push(true);
            }
        }

        // extract wasm_export attrs from this method input
        let mut keep = keep.into_iter();
        method.attrs.retain(|_| keep.next().unwrap_or(true));

        Ok(wasm_export_attrs)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use proc_macro2::TokenStream;
    use syn::{parse::Parser, parse_quote};

    #[test]
    fn test_wasm_export_attrs_parse() {
        // no attributes
        let stream = TokenStream::new();
        let result: WasmExportAttrs = syn::parse2(stream).unwrap();
        assert!(result.should_skip.is_none());
        assert!(result.forward_attrs.is_empty());
        assert!(result.unchecked_return_type.is_none());

        // only skip attr
        let stream = TokenStream::from_str("skip").unwrap();
        let result = syn::parse2::<WasmExportAttrs>(stream).unwrap_err();
        assert_eq!(
            result.to_string(),
            "unexpected `skip` attribute, it is only valid for methods of an impl block"
        );

        // mixed
        let stream = TokenStream::from_str(
            "some_top_attr, some_other_top_attr = something, preserve_js_class",
        )
        .unwrap();
        let result: WasmExportAttrs = syn::parse2(stream).unwrap();
        let expected_forward_attrs = vec![
            parse_quote!(some_top_attr),
            parse_quote!(some_other_top_attr = something),
        ];
        assert!(result.should_skip.is_none());
        assert!(result.unchecked_return_type.is_none());
        assert!(result.preserve_js_class.is_some());
        assert_eq!(result.forward_attrs, expected_forward_attrs);
    }

    #[test]
    fn test_wasm_export_ret_type_with_override() {
        let ret_type: ReturnType = parse_quote!(-> Result<SomeType, Error>);
        let mut wasm_export_attrs = WasmExportAttrs {
            forward_attrs: vec![],
            should_skip: None,
            unchecked_return_type: Some(("SomeOverrideType".to_string(), Span::call_site())),
            preserve_js_class: None,
        };
        let result = wasm_export_attrs.handle_return_type(&ret_type).unwrap();

        let expected_type: Type = parse_quote!(SomeType);
        assert_eq!(result, expected_type);

        let expected_wasm_export_attrs = WasmExportAttrs {
            forward_attrs: vec![parse_quote!(
                unchecked_return_type = "WasmEncodedResult<SomeOverrideType>"
            )],
            should_skip: None,
            unchecked_return_type: Some(("SomeOverrideType".to_string(), Span::call_site())),
            preserve_js_class: None,
        };
        assert!(wasm_export_attrs.should_skip.is_none());
        assert_eq!(
            wasm_export_attrs.forward_attrs,
            expected_wasm_export_attrs.forward_attrs
        );
        assert_eq!(
            wasm_export_attrs.unchecked_return_type.unwrap().0,
            expected_wasm_export_attrs.unchecked_return_type.unwrap().0
        );
    }

    #[test]
    fn test_wasm_export_ret_type_without_override() {
        let ret_type: ReturnType = parse_quote!(-> Result<SomeType, Error>);
        let mut wasm_export_attrs = WasmExportAttrs {
            forward_attrs: vec![],
            should_skip: None,
            unchecked_return_type: None,
            preserve_js_class: None,
        };
        let result = wasm_export_attrs.handle_return_type(&ret_type).unwrap();

        let expected_type: Type = parse_quote!(SomeType);
        assert_eq!(result, expected_type);

        let expected_wasm_export_attrs = WasmExportAttrs {
            forward_attrs: vec![parse_quote!(
                unchecked_return_type = "WasmEncodedResult<SomeType>"
            )],
            should_skip: None,
            unchecked_return_type: None,
            preserve_js_class: None,
        };
        assert!(wasm_export_attrs.should_skip.is_none());
        assert!(wasm_export_attrs.unchecked_return_type.is_none());
        assert_eq!(
            wasm_export_attrs.forward_attrs,
            expected_wasm_export_attrs.forward_attrs
        );
    }

    #[test]
    fn test_handle_attrs_sequence_happy() {
        // parse a mixed seq of attrs
        let input = TokenStream::from_str(
            r#"skip, unchecked_return_type = "something", some_forward_attr"#,
        )
        .unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        wasm_export_attrs.handle_attrs_sequence(seq).unwrap();
        assert!(wasm_export_attrs.should_skip.is_some());
        assert_eq!(
            wasm_export_attrs.unchecked_return_type.unwrap().0,
            "something"
        );
        assert_eq!(
            wasm_export_attrs.forward_attrs,
            vec![parse_quote!(some_forward_attr)]
        );
    }

    #[test]
    fn test_handle_attrs_sequence_unhappy() {
        // dup skip
        let input = TokenStream::from_str(r#"skip, skip"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = wasm_export_attrs.handle_attrs_sequence(seq).unwrap_err();
        assert_eq!(err.to_string(), "duplicate `skip` attribute");

        // dup unchecked_return_type
        let input = TokenStream::from_str(
            r#"unchecked_return_type = "somethingElse", unchecked_return_type = "something""#,
        )
        .unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = wasm_export_attrs.handle_attrs_sequence(seq).unwrap_err();
        assert_eq!(
            err.to_string(),
            "duplicate `unchecked_return_type` attribute"
        );

        // dup preserve_js_class
        let input = TokenStream::from_str(r#"preserve_js_class, preserve_js_class"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = wasm_export_attrs.handle_attrs_sequence(seq).unwrap_err();
        assert_eq!(err.to_string(), "duplicate `preserve_js_class` attribute");

        // invalid skip
        let input = TokenStream::from_str(r#"skip = something"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = wasm_export_attrs.handle_attrs_sequence(seq).unwrap_err();
        assert_eq!(err.to_string(), "unexpected token in attribute, `skip` attribute does not take any extra tokens or arguments");

        // invalid unchecked_return_type
        let input = TokenStream::from_str(r#"unchecked_return_type"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = wasm_export_attrs.handle_attrs_sequence(seq).unwrap_err();
        assert_eq!(err.to_string(), "expected a value for this attribute: `unchecked_return_type = ...` and it must be a string literal");

        // expected string literal for unchecked_return_type
        let input = TokenStream::from_str(r#"unchecked_return_type = notStringLiteral"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = wasm_export_attrs.handle_attrs_sequence(seq).unwrap_err();
        assert_eq!(err.to_string(), "expected string literal");

        // invalid preserve_js_class
        let input = TokenStream::from_str(r#"preserve_js_class = something"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = wasm_export_attrs.handle_attrs_sequence(seq).unwrap_err();
        assert_eq!(err.to_string(), "unexpected token in attribute, `preserve_js_class` attribute does not take any extra tokens or arguments");
    }

    #[test]
    fn test_handle_method_attrs_happy() {
        let mut method: ImplItemFn = parse_quote!(
            #[some_external_macro]
            #[wasm_export(some_forward_attr, unchecked_return_type = "string", skip)]
            pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = WasmExportAttrs::handle_method_attrs(&mut method).unwrap();
        assert_eq!(result.forward_attrs, vec![parse_quote!(some_forward_attr),]);
        assert!(result.preserve_js_class.is_none());
        assert!(result.should_skip.is_some());
        assert!(result
            .unchecked_return_type
            .is_some_and(|v| v.0 == "string"));
        assert_eq!(method.attrs, vec![parse_quote!(#[some_external_macro])]);

        let mut method: ImplItemFn = parse_quote!(
            #[some_external_macro]
            #[wasm_export(some_forward_attr, preserve_js_class)]
            pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = WasmExportAttrs::handle_method_attrs(&mut method).unwrap();
        assert_eq!(result.forward_attrs, vec![parse_quote!(some_forward_attr),]);
        assert!(result.preserve_js_class.is_some());
        assert!(result.should_skip.is_none());
        assert!(result.unchecked_return_type.is_none());
        assert_eq!(method.attrs, vec![parse_quote!(#[some_external_macro])]);

        let mut method: ImplItemFn = parse_quote!(
            #[wasm_export]
            pub fn some_fn(arg1: String) -> Result<SomeType, Error> {
                Ok(SomeType::new())
            }
        );
        let result = WasmExportAttrs::handle_method_attrs(&mut method).unwrap();
        assert_eq!(result.forward_attrs, vec![]);
        assert!(result.preserve_js_class.is_none());
        assert!(result.should_skip.is_none());
        assert!(result.unchecked_return_type.is_none());
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
        let err = WasmExportAttrs::handle_method_attrs(&mut method).unwrap_err();
        assert_eq!(
            err.to_string(),
            "expected `,` as wasm_export attributes must be delimited by comma"
        );
    }

    #[test]
    fn test_try_extract_result_inner_type_happy() {
        let output: ReturnType = parse_quote!(-> Result<SomeType, Error>);
        let result = WasmExportAttrs::try_extract_result_inner_type(&output).unwrap();
        let expected: Type = parse_quote!(SomeType);
        assert_eq!(*result, expected);

        let output: ReturnType = parse_quote!(-> Result<(), Error>);
        let result = WasmExportAttrs::try_extract_result_inner_type(&output).unwrap();
        let expected: Type = parse_quote!(());
        assert_eq!(*result, expected);
    }

    #[test]
    fn test_try_extract_result_inner_type_unhappy() {
        let output: ReturnType = parse_quote!(-> SomeType);
        assert!(WasmExportAttrs::try_extract_result_inner_type(&output).is_none());

        let output: ReturnType = parse_quote!(-> Option<SomeType>);
        assert!(WasmExportAttrs::try_extract_result_inner_type(&output).is_none());

        let output: ReturnType = parse_quote!(-> ());
        assert!(WasmExportAttrs::try_extract_result_inner_type(&output).is_none());

        let output: ReturnType = parse_quote!();
        assert!(WasmExportAttrs::try_extract_result_inner_type(&output).is_none());
    }
}
