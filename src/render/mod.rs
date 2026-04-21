use chrono::NaiveTime;

pub mod binary_clock;
pub mod normal_clock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u16,
    pub height: u16,
}

impl Viewport {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBlock {
    pub title: &'static str,
    pub lines: Vec<String>,
}

impl RenderBlock {
    pub fn new(title: &'static str, lines: Vec<String>) -> Self {
        Self { title, lines }
    }
}

pub trait ClockRenderer {
    fn render(&self, time: NaiveTime, viewport: Viewport) -> RenderBlock;
}

pub fn compose_screen(viewport: Viewport, body: &RenderBlock, footer: &str) -> String {
    let mut lines = vec!["bitclk".to_string(), body.title.to_string(), String::new()];
    lines.extend(body.lines.iter().cloned());
    lines.push(String::new());
    lines.push(footer.to_string());

    if block_fits(viewport, &lines) {
        layout_lines(viewport, &lines)
    } else {
        let fallback = vec![
            "bitclk".to_string(),
            "terminal too small".to_string(),
            "resize to keep the clock readable".to_string(),
        ];
        layout_lines(viewport, &fallback)
    }
}

fn block_fits(viewport: Viewport, lines: &[String]) -> bool {
    let max_width = lines.iter().map(|line| line_width(line)).max().unwrap_or(0);

    lines.len() <= viewport.height as usize && max_width <= viewport.width as usize
}

fn layout_lines(viewport: Viewport, lines: &[String]) -> String {
    if viewport.width == 0 || viewport.height == 0 {
        return String::new();
    }

    let height = viewport.height as usize;
    let lines_to_show = &lines[..lines.len().min(height)];
    let mut frame_lines = Vec::with_capacity(height);
    let top_padding = height.saturating_sub(lines_to_show.len()) / 2;
    let bottom_padding = height.saturating_sub(lines_to_show.len() + top_padding);

    frame_lines.extend(std::iter::repeat_n(String::new(), top_padding));
    frame_lines.extend(
        lines_to_show
            .iter()
            .map(|line| center_line(line, viewport.width as usize)),
    );
    frame_lines.extend(std::iter::repeat_n(String::new(), bottom_padding));

    frame_lines.join("\r\n")
}

fn center_line(line: &str, width: usize) -> String {
    let fitted = truncate_to_width(line, width);
    let padding = width.saturating_sub(line_width(&fitted)) / 2;

    format!("{}{}", " ".repeat(padding), fitted)
}

fn truncate_to_width(line: &str, width: usize) -> String {
    line.chars().take(width).collect()
}

fn line_width(line: &str) -> usize {
    line.chars().count()
}
