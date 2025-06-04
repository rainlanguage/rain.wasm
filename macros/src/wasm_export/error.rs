use syn::Error;

/// Extends the original syn error msg with the given msg
pub fn extend_err_msg(msg: &str) -> impl Fn(Error) -> Error + '_ {
    |err| Error::new(err.span(), err.to_string() + msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn test_extend_error() {
        let err = Error::new(Span::call_site(), "some msg");
        let result = extend_err_msg(", extend msg")(err);
        assert_eq!(result.to_string(), "some msg, extend msg");
    }
}
