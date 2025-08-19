use goldentests::TestConfig;

#[test]
fn test_integration() {
    const DEFAULT_PREFIX: &str = "//_:_// ";
    let test_configs = [
        ("tests/run.sh", "tests/inputs/coverage"),
        ("tests/run.sh", "tests/inputs/asan"),
        ("tests/run.sh", "tests/inputs/race"),
        ("tests/symbolic_run.sh", "tests/inputs/symbolic"),
    ];
    for (binary_path, test_path) in test_configs {
        println!("Running tests for: {}", test_path);
        let config = TestConfig::new(binary_path, test_path, DEFAULT_PREFIX);
        config.run_tests().unwrap();
    }
}
