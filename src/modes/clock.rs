use crate::app::AppResult;
use crate::cli::{ClockArgs, DisplayOptions, StartupRadix};
use crate::color::{paint_foreground, to_terminal_color};
use crate::render::binary_clock::{ClockBase, RadixClockRenderer};
use crate::render::{ClockRenderer, RenderBlock, Viewport, compose_screen};
use crate::terminal::TerminalSession;
use crate::theme::RuntimeTheme;
use chrono::{Local, NaiveTime, Timelike};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::{ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self as ct_terminal, Clear, ClearType},
};
use std::io::{self, Write};
use std::time::Duration;

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(200);

pub fn run(args: ClockArgs, display: DisplayOptions) -> AppResult {
    ClockMode::new(args.startup_radix(), display)?.run()
}

pub struct ClockMode {
    renderer: RadixClockRenderer,
    runtime_theme: RuntimeTheme,
    display: DisplayOptions,
    show_help: bool,
}

impl ClockMode {
    pub fn new(startup_radix: StartupRadix, display: DisplayOptions) -> AppResult<Self> {
        let mut renderer = RadixClockRenderer::default();
        renderer.set_base(startup_radix.into());

        Ok(Self {
            renderer,
            runtime_theme: RuntimeTheme::from_options(
                display.theme_hex.as_deref(),
                display.harmony_mode,
            )?,
            display,
            show_help: false,
        })
    }

    pub fn run(&mut self) -> AppResult {
        let _terminal = TerminalSession::enter()?;
        let mut stdout = io::stdout();
        let mut viewport = current_viewport()?;
        let mut current_time = current_local_time();

        self.draw(&mut stdout, current_time, viewport)?;

        loop {
            let mut should_redraw = false;

            if event::poll(EVENT_POLL_INTERVAL)? {
                match event::read()? {
                    Event::Key(key) if is_key_press(key.kind) => match self.handle_key(key) {
                        LoopControl::Break => break,
                        LoopControl::Continue {
                            should_redraw: redraw,
                        } => {
                            should_redraw |= redraw;
                        }
                    },
                    Event::Resize(width, height) => {
                        viewport = Viewport::new(width, height);
                        should_redraw = true;
                    }
                    _ => {}
                }
            }

            let latest_time = current_local_time();
            if latest_time != current_time {
                current_time = latest_time;
                should_redraw = true;
            }

            if should_redraw {
                self.draw(&mut stdout, current_time, viewport)?;
            }
        }

        Ok(())
    }

    fn draw(&self, stdout: &mut impl Write, time: NaiveTime, viewport: Viewport) -> io::Result<()> {
        let theme = self.runtime_theme.theme();
        let body = if self.show_help {
            self.help_block()
        } else {
            self.renderer.render(time, viewport, &theme)
        };
        let frame = compose_screen(viewport, &body);

        if self.display.transparent {
            execute!(
                stdout,
                ResetColor,
                MoveTo(0, 0),
                Clear(ClearType::All),
                SetForegroundColor(to_terminal_color(&theme.foreground))
            )?;
        } else {
            execute!(
                stdout,
                SetBackgroundColor(to_terminal_color(&theme.background)),
                SetForegroundColor(to_terminal_color(&theme.foreground)),
                MoveTo(0, 0),
                Clear(ClearType::All)
            )?;
        }

        stdout.write_all(frame.as_bytes())?;
        execute!(stdout, ResetColor)?;
        stdout.flush()
    }

    fn handle_key(&mut self, key: KeyEvent) -> LoopControl {
        if self.show_help {
            return self.handle_help_key(key);
        }

        match key.code {
            KeyCode::Char(character) => match character.to_ascii_lowercase() {
                'q' => LoopControl::Break,
                'b' => {
                    self.renderer.set_base(ClockBase::Binary);
                    redraw()
                }
                'o' => {
                    self.renderer.set_base(ClockBase::Octal);
                    redraw()
                }
                'x' => {
                    self.renderer.set_base(ClockBase::Hexadecimal);
                    redraw()
                }
                't' => {
                    self.cycle_theme();
                    redraw()
                }
                'h' => {
                    self.show_help = true;
                    redraw()
                }
                _ => no_redraw(),
            },
            KeyCode::Tab => {
                self.renderer.toggle_orientation();
                redraw()
            }
            _ => no_redraw(),
        }
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> LoopControl {
        match key.code {
            KeyCode::Char(character) if character.eq_ignore_ascii_case(&'q') => LoopControl::Break,
            KeyCode::Char(character) if character.eq_ignore_ascii_case(&'h') => {
                self.show_help = false;
                redraw()
            }
            KeyCode::Esc => {
                self.show_help = false;
                redraw()
            }
            _ => no_redraw(),
        }
    }

    fn help_block(&self) -> RenderBlock {
        let theme = self.runtime_theme.theme();

        RenderBlock::new(vec![
            paint_foreground("bitclk clock help", theme.accent),
            String::new(),
            paint_foreground(
                &format!("mode: {}", self.renderer.base().label()),
                theme.foreground,
            ),
            paint_foreground(
                &format!("layout: {}", self.renderer.orientation().label()),
                theme.foreground,
            ),
            paint_foreground(&self.runtime_theme.help_label(), theme.muted),
            String::new(),
            paint_foreground("b    binary", theme.foreground),
            paint_foreground("o    octal", theme.foreground),
            paint_foreground("x    hexadecimal", theme.foreground),
            paint_foreground("tab  toggle vertical / horizontal layout", theme.foreground),
            paint_foreground(
                &format!("t    {}", self.runtime_theme.cycle_label()),
                theme.foreground,
            ),
            paint_foreground("h    toggle this help", theme.foreground),
            paint_foreground("q    quit", theme.muted),
        ])
    }

    fn cycle_theme(&mut self) {
        self.runtime_theme.cycle();
    }
}

enum LoopControl {
    Break,
    Continue { should_redraw: bool },
}

fn redraw() -> LoopControl {
    LoopControl::Continue {
        should_redraw: true,
    }
}

fn no_redraw() -> LoopControl {
    LoopControl::Continue {
        should_redraw: false,
    }
}

fn current_viewport() -> io::Result<Viewport> {
    let (width, height) = ct_terminal::size()?;
    Ok(Viewport::new(width, height))
}

fn current_local_time() -> NaiveTime {
    let now = Local::now();
    NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second())
        .expect("current local time should always be valid")
}

fn is_key_press(kind: KeyEventKind) -> bool {
    matches!(kind, KeyEventKind::Press | KeyEventKind::Repeat)
}

#[cfg(test)]
mod tests {
    use super::{ClockMode, LoopControl};
    use crate::cli::{DisplayOptions, StartupRadix};
    use crate::color::Rgb;
    use crate::color_engine::ColorHarmonyMode;
    use crate::render::binary_clock::{ClockBase, ClockOrientation};
    use crate::theme::Theme;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn display_options(theme_hex: Option<&str>, harmony_mode: ColorHarmonyMode) -> DisplayOptions {
        DisplayOptions {
            transparent: false,
            theme_hex: theme_hex.map(str::to_string),
            harmony_mode,
        }
    }

    #[test]
    fn startup_radix_maps_to_clock_mode() {
        let mode = ClockMode::new(
            StartupRadix::Hexadecimal,
            display_options(None, ColorHarmonyMode::Triadic),
        )
        .expect("clock mode");

        assert_eq!(mode.renderer.base(), ClockBase::Hexadecimal);
    }

    #[test]
    fn q_exits_clock_mode() {
        let mut mode = ClockMode::new(
            StartupRadix::Binary,
            display_options(None, ColorHarmonyMode::Triadic),
        )
        .expect("clock mode");
        let key = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        let control = mode.handle_key(key);

        assert!(matches!(control, LoopControl::Break));
    }

    #[test]
    fn o_switches_to_octal_mode() {
        let mut mode = ClockMode::new(
            StartupRadix::Binary,
            display_options(None, ColorHarmonyMode::Triadic),
        )
        .expect("clock mode");
        let key = KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        let control = mode.handle_key(key);

        assert!(matches!(
            control,
            LoopControl::Continue {
                should_redraw: true
            }
        ));
        assert_eq!(mode.renderer.base(), ClockBase::Octal);
    }

    #[test]
    fn x_switches_to_hex_mode() {
        let mut mode = ClockMode::new(
            StartupRadix::Binary,
            display_options(None, ColorHarmonyMode::Triadic),
        )
        .expect("clock mode");
        let key = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        let control = mode.handle_key(key);

        assert!(matches!(
            control,
            LoopControl::Continue {
                should_redraw: true
            }
        ));
        assert_eq!(mode.renderer.base(), ClockBase::Hexadecimal);
    }

    #[test]
    fn tab_toggles_orientation() {
        let mut mode = ClockMode::new(
            StartupRadix::Binary,
            display_options(None, ColorHarmonyMode::Triadic),
        )
        .expect("clock mode");
        let key = KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        let control = mode.handle_key(key);

        assert!(matches!(
            control,
            LoopControl::Continue {
                should_redraw: true
            }
        ));
        assert_eq!(mode.renderer.orientation(), ClockOrientation::Horizontal);
    }

    #[test]
    fn h_toggles_help_panel() {
        let mut mode = ClockMode::new(
            StartupRadix::Binary,
            display_options(None, ColorHarmonyMode::Triadic),
        )
        .expect("clock mode");
        let open_help = KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        let close_help = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        let control = mode.handle_key(open_help);
        assert!(matches!(
            control,
            LoopControl::Continue {
                should_redraw: true
            }
        ));
        assert!(mode.show_help);

        let control = mode.handle_key(close_help);
        assert!(matches!(
            control,
            LoopControl::Continue {
                should_redraw: true
            }
        ));
        assert!(!mode.show_help);
    }

    #[test]
    fn t_cycles_to_the_next_theme_preset() {
        let mut mode = ClockMode::new(
            StartupRadix::Binary,
            display_options(None, ColorHarmonyMode::Triadic),
        )
        .expect("clock mode");
        let starting_theme = mode.runtime_theme.theme();
        let key = KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        let control = mode.handle_key(key);

        assert!(matches!(
            control,
            LoopControl::Continue {
                should_redraw: true
            }
        ));
        assert_eq!(mode.runtime_theme.theme(), Theme::preset(1));
        assert_ne!(mode.runtime_theme.theme(), starting_theme);
    }

    #[test]
    fn t_cycles_harmony_mode_for_generated_runtime_theme() {
        let mut mode = ClockMode::new(
            StartupRadix::Binary,
            display_options(Some("#3b82f6"), ColorHarmonyMode::Triadic),
        )
        .expect("clock mode");
        let starting_theme = mode.runtime_theme.theme();
        let key = KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        let control = mode.handle_key(key);

        assert!(matches!(
            control,
            LoopControl::Continue {
                should_redraw: true
            }
        ));
        assert_eq!(
            mode.runtime_theme.theme(),
            Theme::from_base(
                Rgb::from_hex("#3b82f6").expect("hex"),
                ColorHarmonyMode::SplitComplementary
            )
        );
        assert_ne!(mode.runtime_theme.theme(), starting_theme);
    }
}
