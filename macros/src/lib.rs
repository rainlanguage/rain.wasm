use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn fn_bindgen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    unimplemented!("this macro isnt yet implemented");
}

#[proc_macro_attribute]
pub fn error_bindgen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    unimplemented!("this macro isnt yet implemented");
}
