#[test]
pub fn test() {
    macrotest::expand("tests/expand/*.test.rs");
}
