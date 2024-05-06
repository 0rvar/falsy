use falsy::parser::parse;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Expected path to source file");
    let contents = std::fs::read_to_string(path).expect("Failed to read file");
    let result = parse(&contents);
    if result.has_errors() {
        for error in result.errors() {
            eprintln!("{}", error);
        }
        std::process::exit(1);
    }
    println!("{:?}", result.output());
}
