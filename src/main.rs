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
        match parser.parse() {
            Ok(chunks) => chunks.iter().for_each(|chunk| println!("{:?}", chunk)),
            Err(e) => {
                println!("Parser error: {}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    return ExitCode::SUCCESS;
}
