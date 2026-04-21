mod app;
mod cli;
mod modes;
mod render;
mod terminal;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();

    match app::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bitclk: {error}");
            ExitCode::FAILURE
        }
    }
}
