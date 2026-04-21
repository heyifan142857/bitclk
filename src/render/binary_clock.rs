use crate::render::{ClockRenderer, RenderBlock, Viewport};
use chrono::{NaiveTime, Timelike};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryGlyphStyle {
    Digits,
}

#[derive(Debug, Clone, Copy)]
pub struct BinaryClockRenderer {
    style: BinaryGlyphStyle,
}

impl Default for BinaryClockRenderer {
    fn default() -> Self {
        Self {
            style: BinaryGlyphStyle::Digits,
        }
    }
}

impl ClockRenderer for BinaryClockRenderer {
    fn render(&self, time: NaiveTime, _viewport: Viewport) -> RenderBlock {
        RenderBlock::new(
            "binary clock",
            vec![
                self.render_row("HH", time.hour(), 5),
                self.render_row("MM", time.minute(), 6),
                self.render_row("SS", time.second(), 6),
                "layout: direct binary HH / MM / SS".to_string(),
            ],
        )
    }
}

impl BinaryClockRenderer {
    fn render_row(&self, label: &str, value: u32, width: usize) -> String {
        format!("{label} | {}  ({value:02})", self.render_bits(value, width))
    }

    fn render_bits(&self, value: u32, width: usize) -> String {
        (0..width)
            .rev()
            .map(|shift| {
                let bit = ((value >> shift) & 1) as u8;
                self.style.render_bit(bit)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl BinaryGlyphStyle {
    fn render_bit(self, bit: u8) -> &'static str {
        match self {
            Self::Digits => {
                if bit == 1 {
                    "1"
                } else {
                    "0"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryClockRenderer;
    use crate::render::{ClockRenderer, Viewport};
    use chrono::NaiveTime;

    #[test]
    fn renders_direct_binary_rows() {
        let renderer = BinaryClockRenderer::default();
        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(80, 24));

        assert_eq!(output.title, "binary clock");
        assert_eq!(output.lines[0], "HH | 0 1 1 0 1  (13)");
        assert_eq!(output.lines[1], "MM | 0 0 0 1 0 1  (05)");
        assert_eq!(output.lines[2], "SS | 0 0 1 0 0 1  (09)");
    }

    #[test]
    fn uses_fixed_bit_widths_for_each_row() {
        let renderer = BinaryClockRenderer::default();
        let time = NaiveTime::from_hms_opt(23, 59, 59).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(80, 24));

        assert!(output.lines[0].contains("1 0 1 1 1"));
        assert!(output.lines[1].contains("1 1 1 0 1 1"));
        assert!(output.lines[2].contains("1 1 1 0 1 1"));
    }
}
