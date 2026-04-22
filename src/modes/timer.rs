use crate::app::AppResult;
use crate::cli::{DisplayOptions, TimerArgs};
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

pub fn run(args: TimerArgs, display: DisplayOptions) -> AppResult {
    let initial_duration = args
        .duration
        .map_or(Duration::ZERO, |value| value.into_inner());

    TimerMode::new(initial_duration, display)?.run()
}

struct TimerMode {
    state: TimerState,
    renderer: RadixClockRenderer,
    runtime_theme: RuntimeTheme,
    display: DisplayOptions,
    show_help: bool,
}

impl TimerMode {
    fn new(initial_duration: Duration, display: DisplayOptions) -> AppResult<Self> {
        Ok(Self {
            state: TimerState::new(initial_duration),
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
        let mut last_tick = seconds_bucket(self.state.remaining(Instant::now()));

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
            if self.state.take_bell() {
                stdout.write_all(b"\x07")?;
                stdout.flush()?;
                should_redraw = true;
            }

            let remaining = self.state.remaining(now);
            let current_tick = seconds_bucket(remaining);
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
        let remaining = self.state.remaining(now);
        let (hours, minutes, seconds) = duration_components(remaining);

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
            paint_foreground("bitclk timer help", theme.accent),
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
            paint_foreground("r      reset to the configured duration", theme.foreground),
            paint_foreground("0      clear the timer", theme.foreground),
            paint_foreground("left   subtract 10 seconds", theme.foreground),
            paint_foreground("right  add 10 seconds", theme.foreground),
            paint_foreground("down   subtract 1 minute", theme.foreground),
            paint_foreground("up     add 1 minute", theme.foreground),
            paint_foreground("pgdn   subtract 10 minutes", theme.foreground),
            paint_foreground("pgup   add 10 minutes", theme.foreground),
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
            KeyCode::Left => {
                self.state.adjust_seconds(-10, now);
                redraw()
            }
            KeyCode::Right => {
                self.state.adjust_seconds(10, now);
                redraw()
            }
            KeyCode::Down => {
                self.state.adjust_seconds(-60, now);
                redraw()
            }
            KeyCode::Up => {
                self.state.adjust_seconds(60, now);
                redraw()
            }
            KeyCode::PageDown => {
                self.state.adjust_seconds(-600, now);
                redraw()
            }
            KeyCode::PageUp => {
                self.state.adjust_seconds(600, now);
                redraw()
            }
            KeyCode::Char(character) => match character.to_ascii_lowercase() {
                'q' => LoopControl::Break,
                'r' => {
                    self.state.reset();
                    redraw()
                }
                '0' => {
                    self.state.clear();
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

#[derive(Debug, Clone, Copy)]
struct TimerState {
    configured_duration: Duration,
    paused_remaining: Duration,
    deadline: Option<Instant>,
    finished: bool,
    bell_pending: bool,
}

impl TimerState {
    fn new(initial_duration: Duration) -> Self {
        let duration = initial_duration.min(MAX_DISPLAY_DURATION);

        Self {
            configured_duration: duration,
            paused_remaining: duration,
            deadline: None,
            finished: false,
            bell_pending: false,
        }
    }

    fn running(&self) -> bool {
        self.deadline.is_some()
    }

    fn remaining(&self, now: Instant) -> Duration {
        match self.deadline {
            Some(deadline) => deadline.saturating_duration_since(now),
            None => self.paused_remaining,
        }
    }

    fn toggle_running(&mut self, now: Instant) {
        self.sync(now);

        if self.running() {
            self.paused_remaining = self.remaining(now);
            self.deadline = None;
        } else if !self.paused_remaining.is_zero() {
            self.finished = false;
            self.bell_pending = false;
            self.deadline = Some(now + self.paused_remaining);
        }
    }

    fn adjust_seconds(&mut self, delta_seconds: i64, now: Instant) {
        self.sync(now);

        if self.running() {
            return;
        }

        let current = self.paused_remaining.as_secs();
        let next = if delta_seconds >= 0 {
            current.saturating_add(delta_seconds as u64)
        } else {
            current.saturating_sub(delta_seconds.unsigned_abs())
        }
        .min(MAX_DISPLAY_DURATION.as_secs());
        let updated = Duration::from_secs(next);

        self.configured_duration = updated;
        self.paused_remaining = updated;
        self.finished = false;
        self.bell_pending = false;
    }

    fn reset(&mut self) {
        self.deadline = None;
        self.paused_remaining = self.configured_duration;
        self.finished = false;
        self.bell_pending = false;
    }

    fn clear(&mut self) {
        self.configured_duration = Duration::ZERO;
        self.paused_remaining = Duration::ZERO;
        self.deadline = None;
        self.finished = false;
        self.bell_pending = false;
    }

    fn sync(&mut self, now: Instant) {
        if let Some(deadline) = self.deadline {
            if deadline <= now {
                self.deadline = None;
                self.paused_remaining = Duration::ZERO;
                self.finished = true;
                self.bell_pending = true;
            }
        }
    }

    fn take_bell(&mut self) -> bool {
        std::mem::take(&mut self.bell_pending)
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
    use super::{TimerState, duration_components};
    use crate::render::binary_clock::MAX_DISPLAY_DURATION;
    use std::time::{Duration, Instant};

    #[test]
    fn timer_state_counts_down_and_completes() {
        let now = Instant::now();
        let mut state = TimerState::new(Duration::from_secs(10));

        state.toggle_running(now);
        state.sync(now + Duration::from_secs(11));

        assert_eq!(
            state.remaining(now + Duration::from_secs(11)),
            Duration::ZERO
        );
        assert!(state.finished);
        assert!(state.take_bell());
        assert!(!state.take_bell());
    }

    #[test]
    fn adjusting_duration_is_capped_by_display_limit() {
        let now = Instant::now();
        let mut state = TimerState::new(Duration::from_secs(90));

        state.adjust_seconds(MAX_DISPLAY_DURATION.as_secs() as i64, now);

        assert_eq!(state.configured_duration, MAX_DISPLAY_DURATION);
        assert_eq!(state.paused_remaining, MAX_DISPLAY_DURATION);
    }

    #[test]
    fn clear_resets_timer_state() {
        let mut state = TimerState::new(Duration::from_secs(30));

        state.clear();

        assert_eq!(state.configured_duration, Duration::ZERO);
        assert_eq!(state.paused_remaining, Duration::ZERO);
        assert!(!state.finished);
    }

    #[test]
    fn duration_components_render_hours_minutes_and_seconds() {
        let components = duration_components(Duration::from_secs(3_723));

        assert_eq!(components, (1, 2, 3));
    }
}
