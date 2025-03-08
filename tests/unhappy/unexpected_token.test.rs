#[macro_use]
extern crate wasm_bindgen_utils;

struct SomeType;
impl_wasm_traits!(SomeType,);
impl_wasm_traits!(SomeType.);
impl_wasm_traits!(SomeType/);
impl_wasm_traits!(SomeType;);
impl_wasm_traits!(SomeType:);
impl_wasm_traits!(SomeType::);
impl_wasm_traits!(SomeType());
impl_wasm_traits!(SomeType<>);

fn main() {}
