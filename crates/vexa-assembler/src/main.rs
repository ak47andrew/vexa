use std::io::Write;
use std::{env, fs};
use std::path::PathBuf;
use std::process::ExitCode;
use env_logger::fmt::style::{AnsiColor, Color, Style};
use log::*;
use crate::vexa::assemble;

mod errors;
mod operand_checking;
mod parsing;
mod vexa;

fn main() -> ExitCode {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
        .format(|buf, record| {
            let level_style = buf.default_level_style(record.level());
            let secondary_style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
            let ts = buf.timestamp();
            const RESET_STYLE: &str = "\x1B[0m";

            writeln!(buf, "[{secondary_style}{}{RESET_STYLE}][{level_style}{}{RESET_STYLE}]: {}", ts, record.level().as_str(), record.args())
        })
        .init();

    // Args handling is... a hack bc there's a single arg and I don't want to add extra deps
    // TODO: If you'll need more flags and stuff - just add a dep, don't bodge this
    let mut args = env::args().collect::<Vec<String>>();

    let output_path = if args.len() >= 3 {
        if args[args.len() - 2] == "-o" {
            let name = args.pop().unwrap(); // Last - name of the output
            args.pop();  // remove "-o" to only leave the files + caller path for `Usage`
            name
        } else {
            "output.bin".into()
        }
    } else {
        "output.bin".into()
    };

    if args.len() < 2 {
        eprintln!("Usage: {} <source file> [-o binary output]", args[0]);
        return ExitCode::FAILURE;
    }

    let files: Vec<PathBuf> = args[1..].iter().map(|x| x.into()).collect();

    let mut file_not_found = false;
    for file in &files {
        if !fs::exists(file).is_ok_and(|x| x) {
            error!("File {:?} not found", file);
            file_not_found = true;
        }
    }
    if file_not_found {
        return ExitCode::FAILURE;
    }

    let bytecode = assemble(files);
    if bytecode.is_none() {
        error!("Error occurred while assembling. Aborting");
        return ExitCode::FAILURE;
    }
    let bytecode = bytecode.unwrap();
    match fs::write(output_path, bytecode) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            error!("Error writing output file: {}", e);
            ExitCode::FAILURE
        }
    }
}
