use clap::{ArgGroup, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "bitclk",
    version,
    about = "A minimal binary clock for the terminal."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Clock(ClockArgs),
    Stopwatch,
    Timer,
}

#[derive(Debug, Clone, Args, Default)]
#[command(group(
    ArgGroup::new("clock-view")
        .args(["binary", "normal"])
        .multiple(false)
))]
pub struct ClockArgs {
    #[arg(long, help = "Start the clock in binary mode")]
    pub binary: bool,

    #[arg(long, help = "Start the clock in normal mode")]
    pub normal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupView {
    Binary,
    Normal,
}

impl ClockArgs {
    pub fn startup_view(&self) -> StartupView {
        if self.normal {
            StartupView::Normal
        } else {
            StartupView::Binary
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, ClockArgs, Command, StartupView};
    use clap::Parser;

    #[test]
    fn parses_without_a_subcommand() {
        let cli = Cli::parse_from(["bitclk"]);

        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_clock_binary_flag() {
        let cli = Cli::parse_from(["bitclk", "clock", "--binary"]);

        match cli.command {
            Some(Command::Clock(args)) => {
                assert!(args.binary);
                assert_eq!(args.startup_view(), StartupView::Binary);
            }
            command => panic!("expected clock command, got {command:?}"),
        }
    }

    #[test]
    fn parses_clock_normal_flag() {
        let cli = Cli::parse_from(["bitclk", "clock", "--normal"]);

        match cli.command {
            Some(Command::Clock(args)) => {
                assert!(args.normal);
                assert_eq!(args.startup_view(), StartupView::Normal);
            }
            command => panic!("expected clock command, got {command:?}"),
        }
    }

    #[test]
    fn rejects_conflicting_startup_view_flags() {
        let result = Cli::try_parse_from(["bitclk", "clock", "--binary", "--normal"]);

        assert!(result.is_err());
    }

    #[test]
    fn default_clock_args_start_in_binary_mode() {
        let args = ClockArgs::default();

        assert_eq!(args.startup_view(), StartupView::Binary);
    }
}
