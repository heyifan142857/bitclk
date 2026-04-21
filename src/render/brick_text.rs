use crate::color::{Rgb, paint_foreground};

pub const DIGIT_HEIGHT: usize = 5;
pub const DIGIT_WIDTH: usize = 6;
pub const DIGIT_GAP: &str = " ";
pub const GROUP_GAP: &str = "    ";

pub fn render_text(text: &str, color: Rgb, scale: usize) -> Vec<String> {
    let digits: Vec<Vec<String>> = text
        .chars()
        .map(|digit| render_digit(digit, scale))
        .collect();
    let mut lines = Vec::with_capacity(DIGIT_HEIGHT * scale);

    for row in 0..(DIGIT_HEIGHT * scale) {
        let plain_line = digits
            .iter()
            .map(|digit| digit[row].as_str())
            .collect::<Vec<_>>()
            .join(DIGIT_GAP);

        lines.push(paint_foreground(&plain_line, color));
    }

    lines
}

pub fn rendered_text_width(text: &str, scale: usize) -> usize {
    let char_count = text.chars().count();

    char_count * DIGIT_WIDTH * scale + char_count.saturating_sub(1) * DIGIT_GAP.len()
}

fn render_digit(digit: char, scale: usize) -> Vec<String> {
    digit_rows(digit)
        .iter()
        .flat_map(|row| {
            let scaled = scale_row(row, scale);
            std::iter::repeat_n(scaled, scale)
        })
        .collect()
}

fn scale_row(row: &str, scale: usize) -> String {
    row.chars()
        .flat_map(|character| std::iter::repeat_n(character, scale))
        .collect()
}

fn digit_rows(digit: char) -> [&'static str; DIGIT_HEIGHT] {
    match digit {
        '0' => ["██████", "██  ██", "██  ██", "██  ██", "██████"],
        '1' => ["████  ", "  ██  ", "  ██  ", "  ██  ", "██████"],
        '2' => ["██████", "    ██", "██████", "██    ", "██████"],
        '3' => ["██████", "    ██", "██████", "    ██", "██████"],
        '4' => ["██  ██", "██  ██", "██████", "    ██", "    ██"],
        '5' => ["██████", "██    ", "██████", "    ██", "██████"],
        '6' => ["██████", "██    ", "██████", "██  ██", "██████"],
        '7' => ["██████", "    ██", "    ██", "    ██", "    ██"],
        '8' => ["██████", "██  ██", "██████", "██  ██", "██████"],
        '9' => ["██████", "██  ██", "██████", "    ██", "██████"],
        _ => ["      ", "      ", "      ", "      ", "      "],
    }
}
