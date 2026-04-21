use crate::render::brick_text::{DIGIT_HEIGHT, DIGIT_WIDTH, GROUP_GAP, render_text};
use crate::render::{ClockRenderer, RenderBlock, Viewport};
use crate::theme::Theme;
use chrono::{NaiveTime, Timelike};

#[derive(Debug, Default)]
pub struct NormalClockRenderer;

impl ClockRenderer for NormalClockRenderer {
    fn render(&self, time: NaiveTime, viewport: Viewport, theme: &Theme) -> RenderBlock {
        let scale = best_scale(viewport);
        let groups = [
            (format!("{:02}", time.hour()), theme.primary),
            (format!("{:02}", time.minute()), theme.secondary),
            (format!("{:02}", time.second()), theme.accent),
        ];

        let group_lines: Vec<Vec<String>> = groups
            .into_iter()
            .map(|(group, color)| render_text(&group, color, scale))
            .collect();
        let mut lines = Vec::with_capacity(DIGIT_HEIGHT * scale);

        for row in 0..(DIGIT_HEIGHT * scale) {
            let mut line = String::new();

            for (index, group) in group_lines.iter().enumerate() {
                if index > 0 {
                    line.push_str(GROUP_GAP);
                }

                line.push_str(&group[row]);
            }

            lines.push(line);
        }

        RenderBlock::new(lines)
    }
}

fn best_scale(viewport: Viewport) -> usize {
    let fixed_width = GROUP_GAP.len() * 2 + 3;
    let width = viewport.width as usize;
    let height = viewport.height as usize;
    let max_by_width = width.saturating_sub(fixed_width) / (DIGIT_WIDTH * 6);
    let max_by_height = height / DIGIT_HEIGHT;

    max_by_width.min(max_by_height).max(1)
}

#[cfg(test)]
mod tests {
    use super::NormalClockRenderer;
    use crate::render::{ClockRenderer, Viewport};
    use crate::theme::Theme;
    use chrono::NaiveTime;

    #[test]
    fn renders_large_clock_lines() {
        let renderer = NormalClockRenderer;
        let time = NaiveTime::from_hms_opt(9, 7, 3).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(80, 24), &Theme::default());

        assert_eq!(output.lines.len(), 5);
        assert!(output.lines[0].contains('\u{1b}'));
        assert!(output.lines[0].contains("██████"));
        assert!(output.lines.iter().any(|line| line.contains("    ██")));
    }

    #[test]
    fn scales_up_when_the_terminal_is_wide_enough() {
        let renderer = NormalClockRenderer;
        let time = NaiveTime::from_hms_opt(12, 34, 56).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(100, 24), &Theme::default());

        assert_eq!(output.lines.len(), 10);
    }
}
