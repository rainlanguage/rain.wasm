use quote::ToTokens;
use proc_macro2::Span;
use super::{
    tools::{extend_err_msg, try_extract_result_inner_type},
    SKIP_ATTR, UNCHECKED_RETURN_TYPE_ATTR,
};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Error, Meta, ReturnType, Token, Type,
};

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
        if meta.path().is_ident(UNCHECKED_RETURN_TYPE_ATTR) {
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
        } else if meta.path().is_ident(SKIP_ATTR) {
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
