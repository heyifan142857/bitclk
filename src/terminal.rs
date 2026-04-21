use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io;

#[derive(Debug, Default)]
pub struct TerminalSession {
    raw_mode_enabled: bool,
    alternate_screen_enabled: bool,
    cursor_hidden: bool,
}

impl TerminalSession {
    pub fn enter() -> io::Result<Self> {
        let mut session = Self::default();

        enable_raw_mode()?;
        session.raw_mode_enabled = true;

        let mut stdout = io::stdout();
        match execute!(stdout, EnterAlternateScreen, Hide) {
            Ok(()) => {
                session.alternate_screen_enabled = true;
                session.cursor_hidden = true;
                Ok(session)
            }
            Err(error) => {
                session.restore();
                Err(error)
            }
        }
    }

    fn restore(&mut self) {
        let mut stdout = io::stdout();

        if self.cursor_hidden || self.alternate_screen_enabled {
            let _ = execute!(stdout, Show, LeaveAlternateScreen);
        }

        if self.raw_mode_enabled {
            let _ = disable_raw_mode();
        }

        self.cursor_hidden = false;
        self.alternate_screen_enabled = false;
        self.raw_mode_enabled = false;
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.restore();
    }
}
