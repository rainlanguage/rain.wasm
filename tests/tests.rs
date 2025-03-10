#[test]
pub fn happy() {
    // to update the tests "*.expanded.rs" fixtures, first
    // delete them and then run cargo test to regenerate them
    macrotest::expand("tests/happy/*.test.rs");
}

#[test]
fn unhappy() {
    // to update the tests "*.stderr" fixtures, set
    // env TRYBUILD=overwrite and then run cargo test
    let test_runner = trybuild::TestCases::new();
    test_runner.compile_fail("tests/unhappy/*.test.rs");
}
