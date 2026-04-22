use crate::color_engine::ColorHarmonyMode;
use crate::render::binary_clock::{ClockBase, MAX_DISPLAY_DURATION};
use clap::{ArgGroup, Args, Parser, Subcommand};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayOptions {
    pub transparent: bool,
    pub theme_hex: Option<String>,
    pub harmony_mode: ColorHarmonyMode,
}

#[derive(Debug, Parser)]
#[command(
    name = "bitclk",
    version,
    about = "A minimal terminal clock toolkit with binary, octal, and hexadecimal views.",
    after_help = "Examples:\n  bitclk\n  bitclk --transparent\n  bitclk --theme \"#3b82f6\" --mode triadic\n  bitclk --theme \"#3b82f6\" timer 05:00\n  bitclk clock --hex\n  bitclk stopwatch\n  bitclk timer 05:00\n  bitclk theme \"#3b82f6\" --mode triadic",
    next_line_help = true
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        visible_alias = "no-background",
        help = "Do not paint the fullscreen terminal background canvas"
    )]
    pub transparent: bool,

    #[arg(
        long,
        global = true,
        value_name = "HEX",
        help = "Generate a runtime theme from a base hex color for clock, stopwatch, or timer"
    )]
    pub theme: Option<String>,

    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = ColorHarmonyMode::Triadic,
        help = "Harmony rule used with --theme and the theme preview command"
    )]
    pub mode: ColorHarmonyMode,

    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    pub fn display_options(&self) -> DisplayOptions {
        DisplayOptions {
            transparent: self.transparent,
            theme_hex: self.theme.clone(),
            harmony_mode: self.mode,
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    #[command(about = "Run the live clock view")]
    Clock(ClockArgs),
    #[command(about = "Run an interactive stopwatch")]
    Stopwatch,
    #[command(about = "Run an interactive countdown timer")]
    Timer(TimerArgs),
    #[command(
        about = "Preview a generated theme from a base hex color",
        after_help = "Tip:\n  --theme <HEX> runs clock, stopwatch, or timer with a generated runtime palette.\n  bitclk theme <HEX> previews the generated palette without starting the clock."
    )]
    Theme(ThemeArgs),
}

#[derive(Debug, Clone, Args, Default)]
#[command(group(
    ArgGroup::new("clock-radix")
        .args(["binary", "octal", "hex"])
        .multiple(false)
))]
pub struct ClockArgs {
    #[arg(long, help = "Start the clock in binary mode")]
    pub binary: bool,

    #[arg(long, help = "Start the clock in octal mode")]
    pub octal: bool,

    #[arg(long, help = "Start the clock in hexadecimal mode")]
    pub hex: bool,
}

#[derive(Debug, Clone, Args, Default)]
pub struct TimerArgs {
    #[arg(
        value_name = "DURATION",
        value_parser = parse_timer_duration,
        help = "Optional countdown duration: 90, 05:00, 01:02:03, or 1h2m3s"
    )]
    pub duration: Option<TimerDuration>,
}

#[derive(Debug, Clone, Args)]
pub struct ThemeArgs {
    #[arg(value_name = "HEX", help = "Preview base color, for example #3b82f6")]
    pub base: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupRadix {
    Binary,
    Octal,
    Hexadecimal,
}

impl From<StartupRadix> for ClockBase {
    fn from(value: StartupRadix) -> Self {
        match value {
            StartupRadix::Binary => Self::Binary,
            StartupRadix::Octal => Self::Octal,
            StartupRadix::Hexadecimal => Self::Hexadecimal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerDuration(Duration);

impl TimerDuration {
    pub fn into_inner(self) -> Duration {
        self.0
    }
}

impl ClockArgs {
    pub fn startup_radix(&self) -> StartupRadix {
        if self.octal {
            StartupRadix::Octal
        } else if self.hex {
            StartupRadix::Hexadecimal
        } else {
            StartupRadix::Binary
        }
    }
}

fn parse_timer_duration(input: &str) -> Result<TimerDuration, String> {
    parse_duration(input).map(TimerDuration)
}

fn parse_duration(input: &str) -> Result<Duration, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("timer duration cannot be empty".to_string());
    }

    let duration = if trimmed.contains(':') {
        parse_colon_duration(trimmed)?
    } else if trimmed
        .chars()
        .any(|character| character.is_ascii_alphabetic())
    {
        parse_unit_duration(trimmed)?
    } else {
        let seconds = trimmed
            .parse::<u64>()
            .map_err(|_| format!("invalid duration value: {trimmed}"))?;

        Duration::from_secs(seconds)
    };

    validate_duration_limit(duration)
}

fn parse_colon_duration(input: &str) -> Result<Duration, String> {
    let parts: Vec<&str> = input.split(':').collect();

    match parts.as_slice() {
        [minutes, seconds] => {
            let minutes = parse_duration_part("minutes", minutes)?;
            let seconds = parse_duration_part("seconds", seconds)?;

            if seconds >= 60 {
                return Err("seconds must be below 60 in MM:SS format".to_string());
            }

            Ok(Duration::from_secs(minutes * 60 + seconds))
        }
        [hours, minutes, seconds] => {
            let hours = parse_duration_part("hours", hours)?;
            let minutes = parse_duration_part("minutes", minutes)?;
            let seconds = parse_duration_part("seconds", seconds)?;

            if minutes >= 60 || seconds >= 60 {
                return Err("minutes and seconds must be below 60 in HH:MM:SS format".to_string());
            }

            Ok(Duration::from_secs(hours * 3_600 + minutes * 60 + seconds))
        }
        _ => Err("duration must use MM:SS or HH:MM:SS".to_string()),
    }
}

fn parse_unit_duration(input: &str) -> Result<Duration, String> {
    let mut total_seconds = 0_u64;
    let mut digits = String::new();
    let mut seen_unit = false;
    let mut used_hours = false;
    let mut used_minutes = false;
    let mut used_seconds = false;

    for character in input.chars() {
        if character.is_ascii_digit() {
            digits.push(character);
            continue;
        }

        if digits.is_empty() {
            return Err(format!("missing value before duration suffix: {character}"));
        }

        let value = digits
            .parse::<u64>()
            .map_err(|_| format!("invalid duration value: {digits}"))?;
        digits.clear();
        seen_unit = true;

        let seconds = match character.to_ascii_lowercase() {
            'h' if !used_hours => {
                used_hours = true;
                value.saturating_mul(3_600)
            }
            'm' if !used_minutes => {
                used_minutes = true;
                value.saturating_mul(60)
            }
            's' if !used_seconds => {
                used_seconds = true;
                value
            }
            'h' | 'm' | 's' => {
                return Err(format!(
                    "duration suffix can only be used once: {character}"
                ));
            }
            _ => return Err(format!("invalid duration suffix: {character}")),
        };

        total_seconds = total_seconds.saturating_add(seconds);
    }

    if !digits.is_empty() {
        return Err("duration with unit suffixes must end in h, m, or s".to_string());
    }

    if !seen_unit {
        return Err("invalid duration value".to_string());
    }

    Ok(Duration::from_secs(total_seconds))
}

fn parse_duration_part(label: &str, input: &str) -> Result<u64, String> {
    input
        .parse::<u64>()
        .map_err(|_| format!("invalid {label} value: {input}"))
}

fn validate_duration_limit(duration: Duration) -> Result<Duration, String> {
    if duration > MAX_DISPLAY_DURATION {
        return Err("timer duration cannot exceed 63:59:59".to_string());
    }

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::{Cli, ClockArgs, Command, StartupRadix};
    use crate::color_engine::ColorHarmonyMode;
    use clap::{CommandFactory, Parser};
    use std::time::Duration;

    #[test]
    fn parses_without_a_subcommand() {
        let cli = Cli::parse_from(["bitclk"]);

        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_global_theme_and_mode_flags() {
        let cli = Cli::parse_from(["bitclk", "--theme", "#3b82f6", "--mode", "analogous"]);

        assert_eq!(cli.theme.as_deref(), Some("#3b82f6"));
        assert_eq!(cli.mode, ColorHarmonyMode::Analogous);
    }

    #[test]
    fn parses_global_transparent_flag() {
        let cli = Cli::parse_from(["bitclk", "--transparent"]);

        assert!(cli.transparent);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_clock_octal_flag() {
        let cli = Cli::parse_from(["bitclk", "clock", "--octal"]);

        match cli.command {
            Some(Command::Clock(args)) => {
                assert!(args.octal);
                assert_eq!(args.startup_radix(), StartupRadix::Octal);
            }
            command => panic!("expected clock command, got {command:?}"),
        }
    }

    #[test]
    fn parses_clock_hex_flag() {
        let cli = Cli::parse_from(["bitclk", "clock", "--hex"]);

        match cli.command {
            Some(Command::Clock(args)) => {
                assert!(args.hex);
                assert_eq!(args.startup_radix(), StartupRadix::Hexadecimal);
            }
            command => panic!("expected clock command, got {command:?}"),
        }
    }

    #[test]
    fn rejects_conflicting_startup_radix_flags() {
        let result = Cli::try_parse_from(["bitclk", "clock", "--binary", "--hex"]);

        assert!(result.is_err());
    }

    #[test]
    fn default_clock_args_start_in_binary_mode() {
        let args = ClockArgs::default();

        assert_eq!(args.startup_radix(), StartupRadix::Binary);
    }

    #[test]
    fn parses_theme_command() {
        let cli = Cli::parse_from([
            "bitclk",
            "theme",
            "#3b82f6",
            "--mode",
            "split-complementary",
        ]);

        match cli.command {
            Some(Command::Theme(args)) => {
                assert_eq!(args.base, "#3b82f6");
                assert_eq!(cli.mode, ColorHarmonyMode::SplitComplementary);
            }
            command => panic!("expected theme command, got {command:?}"),
        }
    }

    #[test]
    fn parses_timer_duration_argument() {
        let cli = Cli::parse_from(["bitclk", "timer", "05:00"]);

        match cli.command {
            Some(Command::Timer(args)) => {
                assert_eq!(
                    args.duration.expect("duration should parse").into_inner(),
                    Duration::from_secs(300)
                );
            }
            command => panic!("expected timer command, got {command:?}"),
        }
    }

    #[test]
    fn parses_timer_duration_with_units() {
        let cli = Cli::parse_from(["bitclk", "timer", "1h2m3s"]);

        match cli.command {
            Some(Command::Timer(args)) => {
                assert_eq!(
                    args.duration.expect("duration should parse").into_inner(),
                    Duration::from_secs(3_723)
                );
            }
            command => panic!("expected timer command, got {command:?}"),
        }
    }

    #[test]
    fn rejects_timer_duration_above_display_limit() {
        let result = Cli::try_parse_from(["bitclk", "timer", "64:00:00"]);

        assert!(result.is_err());
    }

    #[test]
    fn long_help_mentions_runtime_theme_and_timer_example() {
        let mut command = Cli::command();
        let mut buffer = Vec::new();
        command
            .write_long_help(&mut buffer)
            .expect("help should render");
        let help = String::from_utf8(buffer).expect("help should be utf-8");

        assert!(help.contains("--theme"));
        assert!(help.contains("bitclk --theme \"#3b82f6\" --mode triadic"));
    }
}
