pub mod steps;
pub mod visualizations;

use std::env::args;
use std::fs::{read_to_string, write};
use std::process::{Command, ExitCode};
use crate::steps::parser;
use crate::visualizations::ast_tree::ast_to_dot;

fn main() -> ExitCode {
    let args = args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: {} <file.vxn>", args[0]);
        return ExitCode::FAILURE;
    }
    let source = read_to_string(args[1].clone()).unwrap();
    match parser::parser::Program(source.as_str()) {
        Ok(asts) => {
            write("output.dot", ast_to_dot(&asts)).unwrap();
            Command::new("dot")
                .args(["-Tsvg", "output.dot", "-o", "ast.svg"])
                .output()
                .expect("Failed to execute dot. Is Graphviz installed and in your PATH?");
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
