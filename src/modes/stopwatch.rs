use crate::app::AppResult;
use crate::cli::DisplayOptions;
use crate::color::{paint_foreground, to_terminal_color};
use crate::render::binary_clock::{ClockBase, MAX_DISPLAY_DURATION, RadixClockRenderer};
use crate::render::{RenderBlock, Viewport, compose_screen};
use crate::terminal::TerminalSession;
use crate::theme::RuntimeTheme;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::{ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self as ct_terminal, Clear, ClearType},
};
use std::io::{self, Write};
use std::time::{Duration, Instant};

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn run(display: DisplayOptions) -> AppResult {
    StopwatchMode::new(display)?.run()
}

struct StopwatchMode {
    state: StopwatchState,
    renderer: RadixClockRenderer,
    runtime_theme: RuntimeTheme,
    display: DisplayOptions,
    show_help: bool,
}

impl StopwatchMode {
    fn new(display: DisplayOptions) -> AppResult<Self> {
        Ok(Self {
            state: StopwatchState::default(),
            renderer: RadixClockRenderer::default(),
            runtime_theme: RuntimeTheme::from_options(
                display.theme_hex.as_deref(),
                display.harmony_mode,
            )?,
            display,
            show_help: false,
        })
    }

    fn run(&mut self) -> AppResult {
        let _terminal = TerminalSession::enter()?;
        let mut stdout = io::stdout();
        let mut viewport = current_viewport()?;
        let mut last_tick = seconds_bucket(Duration::ZERO);

        self.draw(&mut stdout, viewport, Instant::now())?;

        loop {
            let mut should_redraw = false;
            let now = Instant::now();

            if event::poll(EVENT_POLL_INTERVAL)? {
                match event::read()? {
                    Event::Key(key) if is_key_press(key.kind) => match self.handle_key(key, now) {
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

            self.state.sync(now);

            let elapsed = self.state.elapsed(now);
            let current_tick = seconds_bucket(elapsed);
            if current_tick != last_tick {
                last_tick = current_tick;
                should_redraw = true;
            }

            if should_redraw {
                self.draw(&mut stdout, viewport, now)?;
            }
        }

        Ok(())
    }

    fn draw(&self, stdout: &mut impl Write, viewport: Viewport, now: Instant) -> io::Result<()> {
        let theme = self.runtime_theme.theme();
        let body = if self.show_help {
            self.help_block()
        } else {
            self.main_block(now, viewport)
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

    fn main_block(&self, now: Instant, viewport: Viewport) -> RenderBlock {
        let elapsed = self.state.elapsed(now);
        let (hours, minutes, seconds) = duration_components(elapsed);

        self.renderer.render_hms(
            hours,
            minutes,
            seconds,
            viewport,
            &self.runtime_theme.theme(),
        )
    }

    fn help_block(&self) -> RenderBlock {
        let theme = self.runtime_theme.theme();

        RenderBlock::new(vec![
            paint_foreground("bitclk stopwatch help", theme.accent),
            String::new(),
            paint_foreground(
                &format!("mode: {}", self.renderer.base().label()),
                theme.foreground,
            ),
            paint_foreground(
                &format!("layout: {}", self.renderer.orientation().label()),
                theme.foreground,
            ),
            paint_foreground("limit: 63:59:59", theme.muted),
            paint_foreground(&self.runtime_theme.help_label(), theme.muted),
            String::new(),
            paint_foreground("space  start / pause", theme.foreground),
            paint_foreground("r      reset", theme.foreground),
            paint_foreground("b      binary", theme.foreground),
            paint_foreground("o      octal", theme.foreground),
            paint_foreground("x      hexadecimal", theme.foreground),
            paint_foreground(
                "tab    toggle vertical / horizontal layout",
                theme.foreground,
            ),
            paint_foreground(
                &format!("t      {}", self.runtime_theme.cycle_label()),
                theme.foreground,
            ),
            paint_foreground("h      toggle this help", theme.foreground),
            paint_foreground("q      quit", theme.muted),
        ])
    }

    fn handle_key(&mut self, key: KeyEvent, now: Instant) -> LoopControl {
        if self.show_help {
            return self.handle_help_key(key);
        }

        match key.code {
            KeyCode::Char(' ') => {
                self.state.toggle_running(now);
                redraw()
            }
            KeyCode::Char(character) => match character.to_ascii_lowercase() {
                'q' => LoopControl::Break,
                'r' => {
                    self.state.reset();
                    redraw()
                }
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

    fn cycle_theme(&mut self) {
        self.runtime_theme.cycle();
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct StopwatchState {
    running: bool,
    started_at: Option<Instant>,
    elapsed_before_pause: Duration,
}

impl StopwatchState {
    fn elapsed(&self, now: Instant) -> Duration {
        let elapsed = match self.started_at {
            Some(started_at) => {
                self.elapsed_before_pause + now.saturating_duration_since(started_at)
            }
            None => self.elapsed_before_pause,
        };

        elapsed.min(MAX_DISPLAY_DURATION)
    }

    fn sync(&mut self, now: Instant) {
        if self.running && self.elapsed(now) >= MAX_DISPLAY_DURATION {
            self.elapsed_before_pause = MAX_DISPLAY_DURATION;
            self.started_at = None;
            self.running = false;
        }
    }

    fn toggle_running(&mut self, now: Instant) {
        self.sync(now);

        if self.running {
            self.elapsed_before_pause = self.elapsed(now);
            self.started_at = None;
            self.running = false;
        } else if self.elapsed_before_pause < MAX_DISPLAY_DURATION {
            self.started_at = Some(now);
            self.running = true;
        }
    }

    fn reset(&mut self) {
        self.running = false;
        self.started_at = None;
        self.elapsed_before_pause = Duration::ZERO;
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

fn duration_components(duration: Duration) -> (u64, u64, u64) {
    let total_seconds = duration.as_secs();

    (
        total_seconds / 3_600,
        (total_seconds / 60) % 60,
        total_seconds % 60,
    )
}

fn seconds_bucket(duration: Duration) -> u64 {
    duration.as_secs()
}

fn current_viewport() -> io::Result<Viewport> {
    let (width, height) = ct_terminal::size()?;
    Ok(Viewport::new(width, height))
}

fn is_key_press(kind: KeyEventKind) -> bool {
    matches!(kind, KeyEventKind::Press | KeyEventKind::Repeat)
}

#[cfg(test)]
mod tests {
    use super::{StopwatchState, duration_components};
    use crate::render::binary_clock::MAX_DISPLAY_DURATION;
    use std::time::{Duration, Instant};

    #[test]
    fn stopwatch_state_accumulates_elapsed_time_across_pauses() {
        let now = Instant::now();
        let mut state = StopwatchState::default();

        state.toggle_running(now);
        assert!(state.running);

        let paused_at = now + Duration::from_secs(5);
        state.toggle_running(paused_at);
        assert!(!state.running);
        assert_eq!(state.elapsed(paused_at), Duration::from_secs(5));

        state.toggle_running(paused_at);
        let stopped_again = paused_at + Duration::from_secs(3);
        state.toggle_running(stopped_again);
        assert_eq!(state.elapsed(stopped_again), Duration::from_secs(8));
    }

    #[test]
    fn stopwatch_caps_elapsed_time_at_display_limit() {
        let now = Instant::now();
        let mut state = StopwatchState::default();

        state.toggle_running(now);
        state.sync(now + MAX_DISPLAY_DURATION + Duration::from_secs(5));

        assert_eq!(
            state.elapsed(now + MAX_DISPLAY_DURATION + Duration::from_secs(5)),
            MAX_DISPLAY_DURATION
        );
        assert!(!state.running);
    }

    #[test]
    fn reset_clears_stopwatch_state() {
        let now = Instant::now();
        let mut state = StopwatchState::default();
        state.toggle_running(now);
        state.toggle_running(now + Duration::from_secs(2));

        state.reset();

        assert!(!state.running);
        assert!(state.started_at.is_none());
        assert_eq!(state.elapsed_before_pause, Duration::ZERO);
    }

    #[test]
    fn duration_components_render_hours_minutes_and_seconds() {
        let components = duration_components(Duration::from_secs(3_723));

        assert_eq!(components, (1, 2, 3));
    }
}
