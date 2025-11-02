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
    match Parser::new(filepath) {
        Ok(mut parser) => match parser.parse() {
            Ok(info) => {
                println!("{:?}", info);
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                println!("Parser Error: {}", e);
                return ExitCode::FAILURE;
            }
        },
        Err(e) => {
            println!("Parser Error: {}", e);
            return ExitCode::FAILURE;
        }
    }
}
