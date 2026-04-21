use crate::render::brick_text::{DIGIT_HEIGHT, render_text, rendered_text_width};
use crate::render::{ClockRenderer, RenderBlock, Viewport};
use crate::theme::Theme;
use chrono::{NaiveTime, Timelike};

const BINARY_WIDTH: usize = 6;
const ROWS: usize = 3;

#[derive(Debug, Default, Clone, Copy)]
pub struct BinaryClockRenderer;

impl ClockRenderer for BinaryClockRenderer {
    fn render(&self, time: NaiveTime, viewport: Viewport, theme: &Theme) -> RenderBlock {
        let groups = binary_groups(time);
        let scale = best_scale(viewport);
        let spacer = spacer_lines(scale);
        let mut lines = Vec::with_capacity(total_height(scale));

        for (index, (digits, color)) in [
            (&groups[0], theme.primary),
            (&groups[1], theme.secondary),
            (&groups[2], theme.accent),
        ]
        .into_iter()
        .enumerate()
        {
            if index > 0 {
                lines.extend(std::iter::repeat_n(String::new(), spacer));
            }

            lines.extend(render_text(digits, color, scale));
        }

        RenderBlock::new(lines)
    }
}

fn binary_groups(time: NaiveTime) -> [String; ROWS] {
    [
        format!("{:0width$b}", time.hour(), width = BINARY_WIDTH),
        format!("{:0width$b}", time.minute(), width = BINARY_WIDTH),
        format!("{:0width$b}", time.second(), width = BINARY_WIDTH),
    ]
}

fn best_scale(viewport: Viewport) -> usize {
    let width = viewport.width as usize;
    let height = viewport.height as usize;
    let sample_width = rendered_text_width("000000", 1);
    let max_by_width = width / sample_width.max(1);
    let max_by_height = height / DIGIT_HEIGHT.max(1);

    for scale in (1..=max_by_width.min(max_by_height).max(1)).rev() {
        if rendered_text_width("000000", scale) <= width && total_height(scale) <= height {
            return scale;
        }
    }

    1
}

fn spacer_lines(scale: usize) -> usize {
    scale
}

fn total_height(scale: usize) -> usize {
    DIGIT_HEIGHT * ROWS * scale + spacer_lines(scale) * (ROWS - 1)
}

#[cfg(test)]
mod tests {
    use super::{BinaryClockRenderer, binary_groups};
    use crate::render::{ClockRenderer, Viewport};
    use crate::theme::Theme;
    use chrono::NaiveTime;

    #[test]
    fn formats_every_group_to_six_bits() {
        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let output = binary_groups(time);

        assert_eq!(output[0], "001101");
        assert_eq!(output[1], "000101");
        assert_eq!(output[2], "001001");
    }

    #[test]
    fn renders_binary_clock_in_the_same_brick_theme() {
        let renderer = BinaryClockRenderer::default();
        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(80, 24), &Theme::default());

        assert_eq!(output.lines.len(), 17);
        assert!(output.lines[0].contains('\u{1b}'));
        assert!(output.lines[0].contains("██████"));
        assert!(output.lines.iter().any(|line| line.contains("██  ██")));
        assert_eq!(output.lines[5], "");
        assert_eq!(output.lines[11], "");
    }
}
