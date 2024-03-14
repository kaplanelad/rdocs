#[test]
fn cli_tests() {
    let t = trycmd::TestCases::new();
    t.case("tests/cmd/*.toml").env("RUST_LOG", "OFF");
    t.case("tests/cmd/*.trycmd").env("RUST_LOG", "OFF");
}
