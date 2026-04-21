use crate::cli::{Cli, ClockArgs, Command};
use crate::modes;
use std::error::Error;

pub type AppResult<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub enum AppCommand {
    Clock(ClockArgs),
    Stopwatch,
    Timer,
}

pub fn run(cli: Cli) -> AppResult {
    match resolve_command(cli) {
        AppCommand::Clock(args) => modes::clock::run(args),
        AppCommand::Stopwatch => modes::stopwatch::run(),
        AppCommand::Timer => modes::timer::run(),
    }
}

pub fn resolve_command(cli: Cli) -> AppCommand {
    match cli.command {
        Some(Command::Clock(args)) => AppCommand::Clock(args),
        Some(Command::Stopwatch) => AppCommand::Stopwatch,
        Some(Command::Timer) => AppCommand::Timer,
        None => AppCommand::Clock(ClockArgs::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::{AppCommand, resolve_command};
    use crate::cli::{Cli, ClockArgs, Command};

    #[test]
    fn root_command_defaults_to_clock() {
        let cli = Cli { command: None };

        match resolve_command(cli) {
            AppCommand::Clock(args) => {
                assert_eq!(args.startup_view(), crate::cli::StartupView::Binary)
            }
            command => panic!("expected clock command, got {command:?}"),
        }
    }

    #[test]
    fn explicit_subcommands_are_preserved() {
        let cli = Cli {
            command: Some(Command::Stopwatch),
        };

        assert!(matches!(resolve_command(cli), AppCommand::Stopwatch));

        let cli = Cli {
            command: Some(Command::Clock(ClockArgs {
                binary: false,
                normal: true,
            })),
        };

        match resolve_command(cli) {
            AppCommand::Clock(args) => assert!(args.normal),
            command => panic!("expected clock command, got {command:?}"),
        }
    }
}
