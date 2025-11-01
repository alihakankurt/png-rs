use png_rs::parser::Parser;
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let exe = &args[0];
        println!("Usage: {exe} <file>");
        return ExitCode::FAILURE;
    }

    let filepath = &args[1];
    if let Some(mut parser) = Parser::new(filepath) {
        if parser.parse() {
            println!("It's a PNG file.");
        } else {
            println!("Provided file is not a PNG.");
        }
    }

    return ExitCode::SUCCESS;
}
