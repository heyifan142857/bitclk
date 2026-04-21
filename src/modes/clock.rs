use crate::app::AppResult;
use crate::cli::{ClockArgs, StartupView};
use crate::color::to_terminal_color;
use crate::color_engine::ColorHarmonyMode;
use crate::render::binary_clock::BinaryClockRenderer;
use crate::render::normal_clock::NormalClockRenderer;
use crate::render::{ClockRenderer, Viewport, compose_screen};
use crate::terminal::TerminalSession;
use crate::theme::{DEFAULT_THEME_BASE, DEFAULT_THEME_MODE, Theme};
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

pub fn run(args: ClockArgs) -> AppResult {
    ClockMode::new(args.startup_view()).run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockView {
    Binary,
    Normal,
}

impl ClockView {
    fn toggle(self) -> Self {
        match self {
            Self::Binary => Self::Normal,
            Self::Normal => Self::Binary,
        }
    }
}

impl From<StartupView> for ClockView {
    fn from(value: StartupView) -> Self {
        match value {
            StartupView::Binary => Self::Binary,
            StartupView::Normal => Self::Normal,
        }
    }
}

pub struct ClockMode {
    view: ClockView,
    normal_renderer: NormalClockRenderer,
    binary_renderer: BinaryClockRenderer,
    theme: Theme,
    theme_base: crate::color::Rgb,
    theme_mode: ColorHarmonyMode,
}

impl ClockMode {
    pub fn new(startup_view: StartupView) -> Self {
        Self {
            view: startup_view.into(),
            normal_renderer: NormalClockRenderer,
            binary_renderer: BinaryClockRenderer::default(),
            theme: Theme::from_base(DEFAULT_THEME_BASE, DEFAULT_THEME_MODE),
            theme_base: DEFAULT_THEME_BASE,
            theme_mode: DEFAULT_THEME_MODE,
        }
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
        let body = self.renderer().render(time, viewport, &self.theme);
        let frame = compose_screen(viewport, &body);

        execute!(
            stdout,
            SetBackgroundColor(to_terminal_color(&self.theme.background)),
            SetForegroundColor(to_terminal_color(&self.theme.foreground)),
            MoveTo(0, 0),
            Clear(ClearType::All)
        )?;
        stdout.write_all(frame.as_bytes())?;
        execute!(stdout, ResetColor)?;
        stdout.flush()
    }

    fn renderer(&self) -> &dyn ClockRenderer {
        match self.view {
            ClockView::Binary => &self.binary_renderer,
            ClockView::Normal => &self.normal_renderer,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> LoopControl {
        match key.code {
            KeyCode::Char(character) => match character.to_ascii_lowercase() {
                'q' => LoopControl::Break,
                'b' => {
                    self.view = ClockView::Binary;
                    LoopControl::Continue {
                        should_redraw: true,
                    }
                }
                'n' => {
                    self.view = ClockView::Normal;
                    LoopControl::Continue {
                        should_redraw: true,
                    }
                }
                't' => {
                    self.cycle_theme();
                    LoopControl::Continue {
                        should_redraw: true,
                    }
                }
                _ => LoopControl::Continue {
                    should_redraw: false,
                },
            },
            KeyCode::Tab => {
                self.view = self.view.toggle();
                LoopControl::Continue {
                    should_redraw: true,
                }
            }
            _ => LoopControl::Continue {
                should_redraw: false,
            },
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_mode = self.theme_mode.next();
        self.theme = Theme::from_base(self.theme_base, self.theme_mode);
    }
}

enum LoopControl {
    Break,
    Continue { should_redraw: bool },
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
    use super::{ClockMode, ClockView, LoopControl};
    use crate::cli::StartupView;
    use crate::color_engine::ColorHarmonyMode;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    #[test]
    fn startup_view_maps_to_clock_mode() {
        let mode = ClockMode::new(StartupView::Normal);

        assert_eq!(mode.view, ClockView::Normal);
    }

    #[test]
    fn q_exits_clock_mode() {
        let mut mode = ClockMode::new(StartupView::Binary);
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
    fn b_switches_to_binary_view() {
        let mut mode = ClockMode::new(StartupView::Normal);
        let key = KeyEvent {
            code: KeyCode::Char('b'),
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
        assert_eq!(mode.view, ClockView::Binary);
    }

    #[test]
    fn n_switches_to_normal_view() {
        let mut mode = ClockMode::new(StartupView::Binary);
        let key = KeyEvent {
            code: KeyCode::Char('n'),
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
        assert_eq!(mode.view, ClockView::Normal);
    }

    #[test]
    fn tab_toggles_between_views() {
        let mut mode = ClockMode::new(StartupView::Binary);
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
        assert_eq!(mode.view, ClockView::Normal);
    }

    #[test]
    fn t_cycles_to_the_next_theme_mode() {
        let mut mode = ClockMode::new(StartupView::Binary);
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
        assert_eq!(mode.theme_mode, ColorHarmonyMode::SplitComplementary);
    }
}
