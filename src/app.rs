use crate::cli::{Cli, ClockArgs, Command, DisplayOptions, ThemeArgs, TimerArgs};
use crate::color_engine::ColorHarmonyMode;
use crate::modes;
use std::error::Error;

pub type AppResult<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub enum AppCommand {
    Clock(ClockArgs, DisplayOptions),
    Stopwatch(DisplayOptions),
    Timer(TimerArgs, DisplayOptions),
    Theme(ThemeArgs, ColorHarmonyMode),
}

pub fn run(cli: Cli) -> AppResult {
    match resolve_command(cli) {
        AppCommand::Clock(args, display) => modes::clock::run(args, display),
        AppCommand::Stopwatch(display) => modes::stopwatch::run(display),
        AppCommand::Timer(args, display) => modes::timer::run(args, display),
        AppCommand::Theme(args, mode) => modes::theme_demo::run(args, mode),
    }
}

pub fn resolve_command(cli: Cli) -> AppCommand {
    let display = cli.display_options();
    let mode = cli.mode;

    match cli.command {
        Some(Command::Clock(args)) => AppCommand::Clock(args, display),
        Some(Command::Stopwatch) => AppCommand::Stopwatch(display),
        Some(Command::Timer(args)) => AppCommand::Timer(args, display),
        Some(Command::Theme(args)) => AppCommand::Theme(args, mode),
        None => AppCommand::Clock(ClockArgs::default(), display),
    }
}

#[cfg(test)]
mod tests {
    use super::{AppCommand, resolve_command};
    use crate::cli::{Cli, ClockArgs, Command, ThemeArgs, TimerArgs};
    use crate::color_engine::ColorHarmonyMode;

    #[test]
    fn root_command_defaults_to_clock() {
        let cli = Cli {
            transparent: true,
            theme: Some("#3b82f6".to_string()),
            mode: ColorHarmonyMode::Analogous,
            command: None,
        };

        match resolve_command(cli) {
            AppCommand::Clock(args, display) => {
                assert_eq!(args.startup_radix(), crate::cli::StartupRadix::Binary);
                assert!(display.transparent);
                assert_eq!(display.theme_hex.as_deref(), Some("#3b82f6"));
                assert_eq!(display.harmony_mode, ColorHarmonyMode::Analogous);
            }
            command => panic!("expected clock command, got {command:?}"),
        }
    }

    #[test]
    fn explicit_subcommands_are_preserved() {
        let cli = Cli {
            transparent: false,
            theme: None,
            mode: ColorHarmonyMode::Triadic,
            command: Some(Command::Stopwatch),
        };

        assert!(matches!(resolve_command(cli), AppCommand::Stopwatch(_)));

        let cli = Cli {
            transparent: false,
            theme: None,
            mode: ColorHarmonyMode::Triadic,
            command: Some(Command::Clock(ClockArgs {
                binary: false,
                octal: false,
                hex: true,
            })),
        };

        match resolve_command(cli) {
            AppCommand::Clock(args, _) => assert!(args.hex),
            command => panic!("expected clock command, got {command:?}"),
        }

        let cli = Cli {
            transparent: true,
            theme: None,
            mode: ColorHarmonyMode::Triadic,
            command: Some(Command::Timer(TimerArgs::default())),
        };

        match resolve_command(cli) {
            AppCommand::Timer(_, display) => assert!(display.transparent),
            command => panic!("expected timer command, got {command:?}"),
        }

        let cli = Cli {
            transparent: false,
            theme: None,
            mode: ColorHarmonyMode::SplitComplementary,
            command: Some(Command::Theme(ThemeArgs {
                base: "#3b82f6".to_string(),
            })),
        };

        match resolve_command(cli) {
            AppCommand::Theme(args, mode) => {
                assert_eq!(args.base, "#3b82f6");
                assert_eq!(mode, ColorHarmonyMode::SplitComplementary);
            }
            command => panic!("expected theme command, got {command:?}"),
        }
    }
}
