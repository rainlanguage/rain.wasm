use quote::ToTokens;
use proc_macro2::Span;
use super::tools::{extend_err_msg, try_extract_result_inner_type};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Error, Meta, ReturnType, Token, Type,
};

/// Contains list of wasm_export macro attribute keys
pub struct AttrKeys;
impl AttrKeys {
    pub const SKIP: &'static str = "skip";
    pub const WASM_EXPORT: &'static str = "wasm_export";
    pub const UNCHECKED_RETURN_TYPE: &'static str = "unchecked_return_type";
}

/// Struct that holds the parsed wasm_export attributes details
#[derive(Debug, Clone, Default)]
pub struct WasmExportAttrs {
    pub forward_attrs: Vec<Meta>,
    pub unchecked_return_type: Option<(String, Span)>,
    pub should_skip: Option<Span>,
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
        handle_attrs_sequence(attrs_seq, &mut wasm_export_attrs)?;

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
        let return_type = try_extract_result_inner_type(output).cloned();
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
}

/// Handles wasm_export specified sequence of attributes delimited by comma
pub fn handle_attrs_sequence(
    metas: Punctuated<Meta, Comma>,
    wasm_export_attrs: &mut WasmExportAttrs,
) -> Result<(), Error> {
    for meta in metas {
        if meta.path().is_ident(AttrKeys::UNCHECKED_RETURN_TYPE) {
            if wasm_export_attrs.unchecked_return_type.is_some() {
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
                wasm_export_attrs.unchecked_return_type = Some((str.value(), meta.span()));
            } else {
                return Err(Error::new_spanned(meta, "expected string literal"));
            }
        } else if meta.path().is_ident(AttrKeys::SKIP) {
            if wasm_export_attrs.should_skip.is_some() {
                return Err(Error::new_spanned(meta, "duplicate `skip` attribute"));
            }
            meta.require_path_only().map_err(extend_err_msg(
                ", `skip` attribute does not take any extra tokens or arguments",
            ))?;
            wasm_export_attrs.should_skip = Some(meta.span());
        } else {
            // include unchanged to be forwarded to the corresponding export item
            wasm_export_attrs.forward_attrs.push(meta);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use syn::parse::Parser;
    use proc_macro2::TokenStream;

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
        let stream =
            TokenStream::from_str("some_top_attr, some_other_top_attr = something").unwrap();
        let result: WasmExportAttrs = syn::parse2(stream).unwrap();
        let expected_forward_attrs = vec![
            syn::parse_quote!(some_top_attr),
            syn::parse_quote!(some_other_top_attr = something),
        ];
        assert!(result.should_skip.is_none());
        assert!(result.unchecked_return_type.is_none());
        assert_eq!(result.forward_attrs, expected_forward_attrs);
    }

    #[test]
    fn test_was_export_ret_type_with_override() {
        let ret_type: ReturnType = syn::parse_quote!(-> Result<SomeType, Error>);
        let mut wasm_export_attrs = WasmExportAttrs {
            forward_attrs: vec![],
            should_skip: None,
            unchecked_return_type: Some(("SomeOverrideType".to_string(), Span::call_site())),
        };
        let result = wasm_export_attrs.handle_return_type(&ret_type).unwrap();

        let expected_type: Type = syn::parse_quote!(SomeType);
        assert_eq!(result, expected_type);

        let expected_wasm_export_attrs = WasmExportAttrs {
            forward_attrs: vec![syn::parse_quote!(
                unchecked_return_type = "WasmEncodedResult<SomeOverrideType>"
            )],
            should_skip: None,
            unchecked_return_type: Some(("SomeOverrideType".to_string(), Span::call_site())),
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
    fn test_was_export_ret_type_without_override() {
        let ret_type: ReturnType = syn::parse_quote!(-> Result<SomeType, Error>);
        let mut wasm_export_attrs = WasmExportAttrs {
            forward_attrs: vec![],
            should_skip: None,
            unchecked_return_type: None,
        };
        let result = wasm_export_attrs.handle_return_type(&ret_type).unwrap();

        let expected_type: Type = syn::parse_quote!(SomeType);
        assert_eq!(result, expected_type);

        let expected_wasm_export_attrs = WasmExportAttrs {
            forward_attrs: vec![syn::parse_quote!(
                unchecked_return_type = "WasmEncodedResult<SomeType>"
            )],
            should_skip: None,
            unchecked_return_type: None,
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
        handle_attrs_sequence(seq, &mut wasm_export_attrs).unwrap();
        assert!(wasm_export_attrs.should_skip.is_some());
        assert_eq!(
            wasm_export_attrs.unchecked_return_type.unwrap().0,
            "something"
        );
        assert_eq!(
            wasm_export_attrs.forward_attrs,
            vec![syn::parse_quote!(some_forward_attr)]
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
        let err = handle_attrs_sequence(seq, &mut wasm_export_attrs).unwrap_err();
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
        let err = handle_attrs_sequence(seq, &mut wasm_export_attrs).unwrap_err();
        assert_eq!(
            err.to_string(),
            "duplicate `unchecked_return_type` attribute"
        );

        // invalid skip
        let input = TokenStream::from_str(r#"skip = something"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = handle_attrs_sequence(seq, &mut wasm_export_attrs).unwrap_err();
        assert_eq!(err.to_string(), "unexpected token in attribute, `skip` attribute does not take any extra tokens or arguments");

        // invalid unchecked_return_type
        let input = TokenStream::from_str(r#"unchecked_return_type"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = handle_attrs_sequence(seq, &mut wasm_export_attrs).unwrap_err();
        assert_eq!(err.to_string(), "expected a value for this attribute: `unchecked_return_type = ...` and it must be a string literal");

        // expected string literal for unchecked_return_type
        let input = TokenStream::from_str(r#"unchecked_return_type = notStringLiteral"#).unwrap();
        let seq = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        let mut wasm_export_attrs = WasmExportAttrs::default();
        let err = handle_attrs_sequence(seq, &mut wasm_export_attrs).unwrap_err();
        assert_eq!(err.to_string(), "expected string literal");
    }
}
