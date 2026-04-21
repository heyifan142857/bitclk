use crate::app::AppResult;
use crate::cli::{ClockArgs, StartupView};
use crate::render::binary_clock::BinaryClockRenderer;
use crate::render::normal_clock::NormalClockRenderer;
use crate::render::{ClockRenderer, Viewport, compose_screen};
use crate::terminal::TerminalSession;
use chrono::{Local, NaiveTime, Timelike};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{self as ct_terminal, Clear, ClearType},
};
use std::io::{self, Write};
use std::time::Duration;

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(200);
const FOOTER_HINT: &str = "q quit  tab toggle  b binary  n normal";

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

    fn label(self) -> &'static str {
        match self {
            Self::Binary => "binary",
            Self::Normal => "normal",
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
}

impl ClockMode {
    pub fn new(startup_view: StartupView) -> Self {
        Self {
            view: startup_view.into(),
            normal_renderer: NormalClockRenderer,
            binary_renderer: BinaryClockRenderer::default(),
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
                        LoopControl::Continue { view_changed } => {
                            should_redraw |= view_changed;
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
        let renderer = self.renderer();
        let body = renderer.render(time, viewport);
        let footer = format!("mode: {}  {FOOTER_HINT}", self.view.label());
        let frame = compose_screen(viewport, &body, &footer);

        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        stdout.write_all(frame.as_bytes())?;
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
                    LoopControl::Continue { view_changed: true }
                }
                'n' => {
                    self.view = ClockView::Normal;
                    LoopControl::Continue { view_changed: true }
                }
                _ => LoopControl::Continue {
                    view_changed: false,
                },
            },
            KeyCode::Tab => {
                self.view = self.view.toggle();
                LoopControl::Continue { view_changed: true }
            }
            _ => LoopControl::Continue {
                view_changed: false,
            },
        }
    }
}

enum LoopControl {
    Break,
    Continue { view_changed: bool },
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
    use super::{ClockMode, ClockView};
    use crate::cli::StartupView;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    #[test]
    fn startup_view_maps_to_clock_mode() {
        let mode = ClockMode::new(StartupView::Normal);

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

        let _ = mode.handle_key(key);

        assert_eq!(mode.view, ClockView::Normal);
    }
}
