use crate::render::brick_text::{DIGIT_HEIGHT, GROUP_GAP, render_text, rendered_text_width};
use crate::render::{ClockRenderer, RenderBlock, Viewport};
use crate::theme::Theme;
use chrono::{NaiveTime, Timelike};
use std::time::Duration;

const BINARY_WIDTH: usize = 6;
const COMPACT_WIDTH: usize = 2;
const ROWS: usize = 3;

pub const MAX_DISPLAY_HOURS: u64 = 63;
pub const MAX_DISPLAY_DURATION: Duration =
    Duration::from_secs(MAX_DISPLAY_HOURS * 3_600 + 59 * 60 + 59);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockBase {
    Binary,
    Octal,
    Hexadecimal,
}

impl ClockBase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Binary => "binary",
            Self::Octal => "octal",
            Self::Hexadecimal => "hexadecimal",
        }
    }

    pub fn format_groups(self, hours: u64, minutes: u64, seconds: u64) -> [String; ROWS] {
        debug_assert!(hours <= MAX_DISPLAY_HOURS);
        debug_assert!(minutes < 60);
        debug_assert!(seconds < 60);

        match self {
            Self::Binary => [
                format!("{hours:0width$b}", width = BINARY_WIDTH),
                format!("{minutes:0width$b}", width = BINARY_WIDTH),
                format!("{seconds:0width$b}", width = BINARY_WIDTH),
            ],
            Self::Octal => [
                format!("{hours:0width$o}", width = COMPACT_WIDTH),
                format!("{minutes:0width$o}", width = COMPACT_WIDTH),
                format!("{seconds:0width$o}", width = COMPACT_WIDTH),
            ],
            Self::Hexadecimal => [
                format!("{hours:0width$X}", width = COMPACT_WIDTH),
                format!("{minutes:0width$X}", width = COMPACT_WIDTH),
                format!("{seconds:0width$X}", width = COMPACT_WIDTH),
            ],
        }
    }
}

impl Default for ClockBase {
    fn default() -> Self {
        Self::Binary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockOrientation {
    Vertical,
    Horizontal,
}

impl ClockOrientation {
    pub fn toggle(self) -> Self {
        match self {
            Self::Vertical => Self::Horizontal,
            Self::Horizontal => Self::Vertical,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

impl Default for ClockOrientation {
    fn default() -> Self {
        Self::Vertical
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RadixClockRenderer {
    orientation: ClockOrientation,
    base: ClockBase,
}

impl RadixClockRenderer {
    pub fn orientation(&self) -> ClockOrientation {
        self.orientation
    }

    pub fn toggle_orientation(&mut self) {
        self.orientation = self.orientation.toggle();
    }

    pub fn base(&self) -> ClockBase {
        self.base
    }

    pub fn set_base(&mut self, base: ClockBase) {
        self.base = base;
    }

    pub fn render_hms(
        &self,
        hours: u64,
        minutes: u64,
        seconds: u64,
        viewport: Viewport,
        theme: &Theme,
    ) -> RenderBlock {
        let groups = self.base.format_groups(hours, minutes, seconds);

        self.render_groups([&groups[0], &groups[1], &groups[2]], viewport, theme)
    }

    pub fn render_groups(
        &self,
        groups: [&str; ROWS],
        viewport: Viewport,
        theme: &Theme,
    ) -> RenderBlock {
        let colors = [theme.primary, theme.secondary, theme.accent];
        let scale = best_scale(&groups, viewport, self.orientation);

        match self.orientation {
            ClockOrientation::Vertical => render_vertical(groups, colors, scale),
            ClockOrientation::Horizontal => render_horizontal(groups, colors, scale),
        }
    }
}

impl ClockRenderer for RadixClockRenderer {
    fn render(&self, time: NaiveTime, viewport: Viewport, theme: &Theme) -> RenderBlock {
        self.render_hms(
            time.hour() as u64,
            time.minute() as u64,
            time.second() as u64,
            viewport,
            theme,
        )
    }
}

fn render_vertical(
    groups: [&str; ROWS],
    colors: [crate::color::Rgb; ROWS],
    scale: usize,
) -> RenderBlock {
    let spacer = spacer_lines(scale);
    let mut lines = Vec::with_capacity(total_height(scale, ClockOrientation::Vertical));

    for (index, (digits, color)) in groups.into_iter().zip(colors).enumerate() {
        if index > 0 {
            lines.extend(std::iter::repeat_n(String::new(), spacer));
        }

        lines.extend(render_text(digits, color, scale));
    }

    RenderBlock::new(lines)
}

fn render_horizontal(
    groups: [&str; ROWS],
    colors: [crate::color::Rgb; ROWS],
    scale: usize,
) -> RenderBlock {
    let rendered_groups: Vec<Vec<String>> = groups
        .into_iter()
        .zip(colors)
        .map(|(digits, color)| render_text(digits, color, scale))
        .collect();
    let mut lines = Vec::with_capacity(DIGIT_HEIGHT * scale);

    for row in 0..(DIGIT_HEIGHT * scale) {
        let mut line = String::new();

        for (index, group) in rendered_groups.iter().enumerate() {
            if index > 0 {
                line.push_str(GROUP_GAP);
            }

            line.push_str(&group[row]);
        }

        lines.push(line);
    }

    RenderBlock::new(lines)
}

fn best_scale(groups: &[&str; ROWS], viewport: Viewport, orientation: ClockOrientation) -> usize {
    let width = viewport.width as usize;
    let height = viewport.height as usize;
    let upper_bound = width.max(height).max(1);

    for scale in (1..=upper_bound).rev() {
        if content_width(groups, scale, orientation) <= width
            && total_height(scale, orientation) <= height
        {
            return scale;
        }
    }

    1
}

fn content_width(groups: &[&str; ROWS], scale: usize, orientation: ClockOrientation) -> usize {
    match orientation {
        ClockOrientation::Vertical => groups
            .iter()
            .map(|group| rendered_text_width(group, scale))
            .max()
            .unwrap_or(0),
        ClockOrientation::Horizontal => {
            groups
                .iter()
                .map(|group| rendered_text_width(group, scale))
                .sum::<usize>()
                + GROUP_GAP.len() * (ROWS - 1)
        }
    }
}

fn spacer_lines(scale: usize) -> usize {
    scale
}

fn total_height(scale: usize, orientation: ClockOrientation) -> usize {
    match orientation {
        ClockOrientation::Vertical => {
            DIGIT_HEIGHT * ROWS * scale + spacer_lines(scale) * (ROWS - 1)
        }
        ClockOrientation::Horizontal => DIGIT_HEIGHT * scale,
    }
}

#[cfg(test)]
mod tests {
    use super::{ClockBase, ClockOrientation, RadixClockRenderer};
    use crate::render::{ClockRenderer, Viewport};
    use crate::theme::Theme;
    use chrono::NaiveTime;

    #[test]
    fn formats_binary_groups_to_six_bits() {
        let output = ClockBase::Binary.format_groups(13, 5, 9);

        assert_eq!(output[0], "001101");
        assert_eq!(output[1], "000101");
        assert_eq!(output[2], "001001");
    }

    #[test]
    fn formats_octal_groups_to_two_digits() {
        let output = ClockBase::Octal.format_groups(13, 5, 9);

        assert_eq!(output[0], "15");
        assert_eq!(output[1], "05");
        assert_eq!(output[2], "11");
    }

    #[test]
    fn formats_hex_groups_to_two_digits() {
        let output = ClockBase::Hexadecimal.format_groups(13, 5, 9);

        assert_eq!(output[0], "0D");
        assert_eq!(output[1], "05");
        assert_eq!(output[2], "09");
    }

    #[test]
    fn renders_vertical_clock_by_default() {
        let renderer = RadixClockRenderer::default();
        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(80, 24), &Theme::default());

        assert_eq!(renderer.orientation(), ClockOrientation::Vertical);
        assert_eq!(renderer.base(), ClockBase::Binary);
        assert_eq!(output.lines.len(), 17);
        assert!(output.lines[0].contains('\u{1b}'));
        assert_eq!(output.lines[5], "");
        assert_eq!(output.lines[11], "");
    }

    #[test]
    fn renders_horizontal_hex_clock_when_requested() {
        let mut renderer = RadixClockRenderer::default();
        renderer.toggle_orientation();
        renderer.set_base(ClockBase::Hexadecimal);
        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(120, 24), &Theme::default());

        assert_eq!(renderer.orientation(), ClockOrientation::Horizontal);
        assert_eq!(renderer.base(), ClockBase::Hexadecimal);
        assert!(output.lines.len() >= 5);
        assert!(output.lines[0].contains('\u{1b}'));
        assert!(output.lines[0].contains("██████"));
        assert!(output.lines.iter().all(|line| !line.is_empty()));
    }
}
