use crate::render::{ClockRenderer, RenderBlock, Viewport};
use chrono::{NaiveTime, Timelike};

#[derive(Debug, Default)]
pub struct NormalClockRenderer;

impl ClockRenderer for NormalClockRenderer {
    fn render(&self, time: NaiveTime, _viewport: Viewport) -> RenderBlock {
        let main = format!(
            "{:02}:{:02}:{:02}",
            time.hour(),
            time.minute(),
            time.second()
        );

        RenderBlock::new("normal clock", vec![main, "local time".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::NormalClockRenderer;
    use crate::render::{ClockRenderer, Viewport};
    use chrono::NaiveTime;

    #[test]
    fn renders_digital_time() {
        let renderer = NormalClockRenderer;
        let time = NaiveTime::from_hms_opt(9, 7, 3).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(80, 24));

        assert_eq!(output.title, "normal clock");
        assert_eq!(output.lines[0], "09:07:03");
        assert_eq!(output.lines[1], "local time");
    }
}
