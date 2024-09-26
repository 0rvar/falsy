use std::path::Path;

use falsy::{interpreter, parser::parse};

test_each_file::test_each_path! { in "./tests/samples" => test_samples }

fn test_samples(path: &Path) {
    // Get all .false files in tests/samples
    if path.extension().unwrap() != "false" {
        return;
    }

    let contents = std::fs::read_to_string(&path).unwrap();
    let ast = parse(&contents).into_result().expect("Failed to parse");

    let manifest = path.with_extension("toml");
    let manifest = std::fs::read_to_string(manifest).unwrap();
    let manifest: SampleManifest = toml::from_str(&manifest).unwrap();

    for run in manifest.runs {
        let mut output = Vec::new();
        let mut input = run.input.chars();
        interpreter::Interpreter::new()
            .on_input(|| input.next().map(|c| c as u8))
            .on_output(|s| output.push(s.to_string()))
            .run_program(ast.clone());

        let output = output.join("");

        assert_eq!(
            output, run.output,
            "output mismatch for input {}",
            run.input
        );
    }
}

#[derive(serde_derive::Deserialize)]
struct SampleManifest {
    runs: Vec<SampleManifestRun>,
}

#[derive(serde_derive::Deserialize)]
struct SampleManifestRun {
    input: String,
    output: String,
}
