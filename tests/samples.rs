#[test]
fn test_samples() {
    let mut passed = true;
    // Get all .false files in tests/samples
    let samples = std::fs::read_dir("tests/samples").unwrap();
    for sample in samples {
        let sample = sample.unwrap();
        let path = sample.path();
        if path.extension().unwrap() == "false" {
            let contents = std::fs::read_to_string(path).unwrap();
            let result = super::parse(&contents);
        }
    }
}
