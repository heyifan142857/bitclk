mod app;
mod cli;
mod color;
mod color_engine;
mod modes;
mod render;
mod terminal;
mod theme;

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
