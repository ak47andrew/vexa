pub mod parser;

use std::env::args;
use std::fs::read_to_string;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: {} <file.vxn>", args[0]);
        return ExitCode::FAILURE;
    }
    let source = read_to_string(args[1].clone()).unwrap();
    match parser::parser::Program(source.as_str()) {
        Ok(asts) => {
            for ast in asts {
                parser::print_ast(ast, 0);
            }
            ExitCode::SUCCESS
        },
        Err(e) => {
            println!("Error: {}", e);
            ExitCode::FAILURE
        },
    }
}
