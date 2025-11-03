use png_rs::parser::Parser;
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let exe = &args[0];
        println!("Usage: {exe} <file>");
        return ExitCode::FAILURE;
    }

    let filepath = &args[1];
    let mut file = match fs::File::open(filepath) {
        Ok(f) => f,
        Err(ref e) => {
            println!("Unable to open {}: {}", filepath, e);
            return ExitCode::FAILURE;
        }
    };

    return match Parser::parse(&mut file) {
        Ok(info) => {
            println!("{:#?}", info);
            ExitCode::SUCCESS
        }
        Err(ref e) => {
            println!("Parser Error: {}", e);
            ExitCode::FAILURE
        }
    };
}
