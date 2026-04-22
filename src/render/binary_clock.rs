use crate::color::{Rgb, paint_foreground, paint_sample};
use crate::render::brick_text::{DIGIT_HEIGHT, GROUP_GAP, render_text, rendered_text_width};
use crate::render::{ClockRenderer, RenderBlock, Viewport};
use crate::theme::Theme;
use chrono::{NaiveTime, Timelike};
use std::time::Duration;

const BINARY_WIDTH: usize = 6;
const BCD_WIDTH: usize = 4;
const COMPACT_WIDTH: usize = 2;
const ROWS: usize = 3;
const BCD_DIGITS: usize = 6;
const BCD_COLUMN_HEIGHTS: [usize; BCD_DIGITS] = [2, 4, 3, 4, 3, 4];
const MATRIX_LABELS: [char; ROWS] = ['H', 'M', 'S'];
const BCD_GROUP_NAMES: [&str; ROWS] = ["Hours", "Minutes", "Seconds"];
const BCD_WEIGHTS: [u8; BCD_WIDTH] = [8, 4, 2, 1];
const MATRIX_LIT_COLOR: Rgb = Rgb::new(0xFF, 0xC5, 0x0F);
const BCD_LIT_COLOR: Rgb = Rgb::new(0xF2, 0x50, 0x8D);
const LAMP_OFF_COLOR: Rgb = Rgb::WHITE;
const MAX_LAMP_SCALE: usize = 4;

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
pub enum BinaryStyle {
    Dense,
    Matrix,
    Bcd,
}

impl BinaryStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dense => "dense",
            Self::Matrix => "matrix",
            Self::Bcd => "bcd",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Dense => Self::Matrix,
            Self::Matrix => Self::Bcd,
            Self::Bcd => Self::Dense,
        }
    }
}

impl Default for BinaryStyle {
    fn default() -> Self {
        Self::Dense
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
    binary_style: BinaryStyle,
}

impl RadixClockRenderer {
    #[cfg(test)]
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

    pub fn binary_style(&self) -> BinaryStyle {
        self.binary_style
    }

    pub fn cycle_binary_style(&mut self) {
        self.binary_style = self.binary_style.next();
    }

    pub fn supports_orientation(&self) -> bool {
        !matches!(
            (self.base, self.binary_style),
            (ClockBase::Binary, BinaryStyle::Matrix | BinaryStyle::Bcd)
        )
    }

    pub fn layout_label(&self) -> &'static str {
        if self.supports_orientation() {
            self.orientation.label()
        } else {
            "fixed"
        }
    }

    pub fn tab_help_label(&self) -> &'static str {
        if self.supports_orientation() {
            "toggle vertical / horizontal layout"
        } else {
            "layout is fixed in this style"
        }
    }

    pub fn render_hms(
        &self,
        hours: u64,
        minutes: u64,
        seconds: u64,
        viewport: Viewport,
        theme: &Theme,
    ) -> RenderBlock {
        match (self.base, self.binary_style) {
            (ClockBase::Binary, BinaryStyle::Matrix) => {
                let groups = ClockBase::Binary.format_groups(hours, minutes, seconds);

                best_custom_scale(viewport, MAX_LAMP_SCALE, |scale| {
                    render_matrix(
                        [&groups[0], &groups[1], &groups[2]],
                        MATRIX_LIT_COLOR,
                        LAMP_OFF_COLOR,
                        MATRIX_LIT_COLOR,
                        scale,
                    )
                })
            }
            (ClockBase::Binary, BinaryStyle::Bcd) => {
                let digits = bcd_digits(hours, minutes, seconds);

                best_custom_scale(viewport, MAX_LAMP_SCALE, |scale| {
                    render_bcd_panel(digits, BCD_LIT_COLOR, LAMP_OFF_COLOR, BCD_LIT_COLOR, scale)
                })
            }
            _ => {
                let groups = self.base.format_groups(hours, minutes, seconds);
                self.render_text_groups([&groups[0], &groups[1], &groups[2]], viewport, theme)
            }
        }
    }

    fn render_text_groups(
        &self,
        groups: [&str; ROWS],
        viewport: Viewport,
        theme: &Theme,
    ) -> RenderBlock {
        let colors = [theme.primary, theme.secondary, theme.accent];
        let scale = best_text_scale(&groups, viewport, self.orientation);

        match self.orientation {
            ClockOrientation::Vertical => render_text_vertical(groups, colors, scale),
            ClockOrientation::Horizontal => render_text_horizontal(groups, colors, scale),
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

fn render_text_vertical(groups: [&str; ROWS], colors: [Rgb; ROWS], scale: usize) -> RenderBlock {
    let spacer = text_spacer_lines(scale);
    let mut lines = Vec::with_capacity(text_total_height(scale, ClockOrientation::Vertical));

    for (index, (digits, color)) in groups.into_iter().zip(colors).enumerate() {
        if index > 0 {
            lines.extend(std::iter::repeat_n(String::new(), spacer));
        }

        lines.extend(render_text(digits, color, scale));
    }

    RenderBlock::new(lines)
}

fn render_text_horizontal(groups: [&str; ROWS], colors: [Rgb; ROWS], scale: usize) -> RenderBlock {
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

fn render_matrix(
    groups: [&str; ROWS],
    lamp_color: Rgb,
    off_color: Rgb,
    frame_color: Rgb,
    scale: usize,
) -> RenderBlock {
    let cell_gap = lamp_cell_gap();
    let group_gap = matrix_group_gap(scale);
    let row_height = lamp_cell_height(scale);
    let separator = paint_foreground("│", frame_color);
    let mut lines = Vec::with_capacity(ROWS * row_height + (ROWS - 1) * group_gap);

    for index in 0..ROWS {
        let label = paint_foreground(&MATRIX_LABELS[index].to_string(), frame_color);
        let prefix = format!("{label} {separator} ");
        let empty_prefix = " ".repeat(visible_width(&prefix));
        let cells = groups[index]
            .chars()
            .map(|bit| render_lamp(bit == '1', lamp_color, off_color, scale))
            .collect::<Vec<_>>()
            .join(cell_gap);

        for repeat in 0..row_height {
            let row_prefix = if repeat == row_height / 2 {
                prefix.as_str()
            } else {
                empty_prefix.as_str()
            };

            lines.push(format!("{row_prefix}{cells}"));
        }

        if index + 1 < ROWS {
            lines.extend(std::iter::repeat_n(String::new(), group_gap));
        }
    }

    RenderBlock::new(lines)
}

fn render_bcd_panel(
    digits: [u8; BCD_DIGITS],
    lamp_color: Rgb,
    off_color: Rgb,
    frame_color: Rgb,
    scale: usize,
) -> RenderBlock {
    let columns: Vec<[bool; BCD_WIDTH]> = digits.iter().copied().map(bcd_bits).collect();
    let column_heights: [usize; BCD_DIGITS] =
        std::array::from_fn(|index| bcd_column_height(index, digits[index]));
    let cell_gap = lamp_cell_gap();
    let row_height = lamp_cell_height(scale);
    let separator = paint_foreground("│", frame_color);
    let group_gap = format!(" {separator} ");
    let digit_width = lamp_cell_width(scale);
    let pair_width = digit_width * 2 + cell_gap.len();
    let header_width = BCD_GROUP_NAMES
        .iter()
        .map(|label| label.chars().count())
        .max()
        .unwrap_or(0);
    let group_width = (pair_width + 2).max(header_width);
    let mut lines = Vec::with_capacity(BCD_WIDTH * row_height + 2);

    let header = (0..ROWS)
        .map(|group_index| {
            paint_foreground(
                &format!(
                    "{:^width$}",
                    BCD_GROUP_NAMES[group_index],
                    width = group_width
                ),
                frame_color,
            )
        })
        .collect::<Vec<_>>()
        .join(&group_gap);
    lines.push(header);

    for (row, weight) in BCD_WEIGHTS.into_iter().enumerate() {
        let row_groups = (0..ROWS)
            .map(|group_index| {
                let start = group_index * 2;
                let left = render_bcd_lamp(
                    columns[start][row],
                    column_heights[start],
                    row,
                    lamp_color,
                    off_color,
                    scale,
                );
                let right = render_bcd_lamp(
                    columns[start + 1][row],
                    column_heights[start + 1],
                    row,
                    lamp_color,
                    off_color,
                    scale,
                );

                center_visible(&format!("{left}{cell_gap}{right}"), group_width)
            })
            .collect::<Vec<_>>()
            .join(&group_gap);
        let weight = format!("  {}", paint_foreground(&weight.to_string(), frame_color));
        let empty_weight = " ".repeat(visible_width(&weight));

        for repeat in 0..row_height {
            let row_weight = if repeat == row_height / 2 {
                weight.as_str()
            } else {
                empty_weight.as_str()
            };

            lines.push(format!("{row_groups}{row_weight}"));
        }
    }

    let footer = (0..ROWS)
        .map(|group_index| {
            let start = group_index * 2;
            let left = render_digit_cell(digits[start], frame_color, digit_width);
            let right = render_digit_cell(digits[start + 1], frame_color, digit_width);

            center_visible(&format!("{left}{cell_gap}{right}"), group_width)
        })
        .collect::<Vec<_>>()
        .join(&group_gap);
    lines.push(footer);

    RenderBlock::new(lines)
}

fn render_lamp(lit: bool, lit_color: Rgb, off_color: Rgb, scale: usize) -> String {
    let color = if lit { lit_color } else { off_color };
    let width = lamp_cell_width(scale);

    paint_sample(&" ".repeat(width), color, color)
}

fn render_digit_cell(digit: u8, color: Rgb, width: usize) -> String {
    paint_foreground(&format!("{digit:^width$}"), color)
}

fn render_bcd_lamp(
    lit: bool,
    column_height: usize,
    row: usize,
    lit_color: Rgb,
    off_color: Rgb,
    scale: usize,
) -> String {
    if bcd_row_is_active(column_height, row) {
        render_lamp(lit, lit_color, off_color, scale)
    } else {
        blank_lamp_cell(scale)
    }
}

fn lamp_cell_width(scale: usize) -> usize {
    scale.max(1).saturating_mul(2)
}

fn lamp_cell_height(scale: usize) -> usize {
    scale.max(1)
}

fn lamp_cell_gap() -> &'static str {
    " "
}

fn matrix_group_gap(scale: usize) -> usize {
    scale.max(1).div_ceil(2)
}

fn bcd_column_height(index: usize, digit: u8) -> usize {
    BCD_COLUMN_HEIGHTS[index]
        .max(bcd_required_height(digit))
        .min(BCD_WIDTH)
}

fn bcd_required_height(digit: u8) -> usize {
    match digit {
        0 | 1 => 1,
        2 | 3 => 2,
        4..=7 => 3,
        _ => 4,
    }
}

fn bcd_row_is_active(column_height: usize, row: usize) -> bool {
    row + column_height >= BCD_WIDTH
}

fn blank_lamp_cell(scale: usize) -> String {
    " ".repeat(lamp_cell_width(scale))
}

fn bcd_digits(hours: u64, minutes: u64, seconds: u64) -> [u8; BCD_DIGITS] {
    [
        (hours / 10) as u8,
        (hours % 10) as u8,
        (minutes / 10) as u8,
        (minutes % 10) as u8,
        (seconds / 10) as u8,
        (seconds % 10) as u8,
    ]
}

fn bcd_bits(digit: u8) -> [bool; BCD_WIDTH] {
    [
        digit & 0b1000 != 0,
        digit & 0b0100 != 0,
        digit & 0b0010 != 0,
        digit & 0b0001 != 0,
    ]
}

fn best_text_scale(
    groups: &[&str; ROWS],
    viewport: Viewport,
    orientation: ClockOrientation,
) -> usize {
    let width = viewport.width as usize;
    let height = viewport.height as usize;
    let upper_bound = width.max(height).max(1);

    for scale in (1..=upper_bound).rev() {
        if text_content_width(groups, scale, orientation) <= width
            && text_total_height(scale, orientation) <= height
        {
            return scale;
        }
    }

    1
}

fn best_custom_scale(
    viewport: Viewport,
    max_scale: usize,
    render: impl Fn(usize) -> RenderBlock,
) -> RenderBlock {
    let upper_bound = (viewport.width.max(viewport.height).max(1) as usize).min(max_scale.max(1));

    for scale in (1..=upper_bound).rev() {
        let block = render(scale);

        if custom_block_fits(viewport, &block) {
            return block;
        }
    }

    render(1)
}

fn text_content_width(groups: &[&str; ROWS], scale: usize, orientation: ClockOrientation) -> usize {
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

fn text_spacer_lines(scale: usize) -> usize {
    scale
}

fn text_total_height(scale: usize, orientation: ClockOrientation) -> usize {
    match orientation {
        ClockOrientation::Vertical => {
            DIGIT_HEIGHT * ROWS * scale + text_spacer_lines(scale) * (ROWS - 1)
        }
        ClockOrientation::Horizontal => DIGIT_HEIGHT * scale,
    }
}

fn custom_block_fits(viewport: Viewport, block: &RenderBlock) -> bool {
    let max_width = block
        .lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);

    block.lines.len() <= viewport.height as usize && max_width <= viewport.width as usize
}

fn center_visible(content: &str, width: usize) -> String {
    let content_width = visible_width(content);

    if content_width >= width {
        return content.to_string();
    }

    let padding = width - content_width;
    let left = padding / 2;
    let right = padding - left;

    format!("{}{}{}", " ".repeat(left), content, " ".repeat(right))
}

fn visible_width(line: &str) -> usize {
    let mut width = 0;
    let mut chars = line.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            chars.next();

            for codepoint in chars.by_ref() {
                if ('@'..='~').contains(&codepoint) {
                    break;
                }
            }

            continue;
        }

        width += 1;
    }

    width
}

#[cfg(test)]
mod tests {
    use super::{
        BCD_LIT_COLOR, BinaryStyle, ClockBase, ClockOrientation, LAMP_OFF_COLOR,
        RadixClockRenderer, bcd_digits, render_bcd_panel,
    };
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
    fn binary_style_cycle_covers_all_variants() {
        let mut renderer = RadixClockRenderer::default();

        assert_eq!(renderer.binary_style(), BinaryStyle::Dense);

        renderer.cycle_binary_style();
        assert_eq!(renderer.binary_style(), BinaryStyle::Matrix);

        renderer.cycle_binary_style();
        assert_eq!(renderer.binary_style(), BinaryStyle::Bcd);

        renderer.cycle_binary_style();
        assert_eq!(renderer.binary_style(), BinaryStyle::Dense);
    }

    #[test]
    fn renders_vertical_clock_by_default() {
        let renderer = RadixClockRenderer::default();
        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(80, 24), &Theme::default());

        assert_eq!(renderer.orientation(), ClockOrientation::Vertical);
        assert_eq!(renderer.base(), ClockBase::Binary);
        assert_eq!(renderer.binary_style(), BinaryStyle::Dense);
        assert_eq!(output.lines.len(), 17);
        assert!(output.lines[0].contains('\u{1b}'));
        assert_eq!(output.lines[5], "");
        assert_eq!(output.lines[11], "");
    }

    #[test]
    fn renders_matrix_with_individual_lamps() {
        let mut renderer = RadixClockRenderer::default();
        renderer.cycle_binary_style();
        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(80, 24), &Theme::default());

        assert_eq!(renderer.binary_style(), BinaryStyle::Matrix);
        assert!(output.lines.len() > 3);
        assert!(output.lines.iter().any(|line| line.contains('H')));
        assert!(output.lines.iter().any(|line| line.contains('M')));
        assert!(output.lines.iter().any(|line| line.contains('S')));
        assert!(output.lines.iter().any(|line| line.contains('│')));
        assert!(output.lines.iter().filter(|line| line.is_empty()).count() >= 2);
        assert!(
            output
                .lines
                .iter()
                .any(|line| line.contains("48;2;255;197;15"))
        );
        assert!(
            output
                .lines
                .iter()
                .any(|line| line.contains("48;2;255;255;255"))
        );
    }

    #[test]
    fn renders_bcd_panel_with_headers_and_weights() {
        let mut renderer = RadixClockRenderer::default();
        renderer.cycle_binary_style();
        renderer.cycle_binary_style();
        let time = NaiveTime::from_hms_opt(12, 34, 56).expect("time should be valid");
        let output = renderer.render(time, Viewport::new(120, 24), &Theme::default());

        assert_eq!(renderer.binary_style(), BinaryStyle::Bcd);
        assert!(output.lines.len() > 6);
        assert!(output.lines.iter().any(|line| line.contains("Hours")));
        assert!(output.lines.iter().any(|line| line.contains("Minutes")));
        assert!(output.lines.iter().any(|line| line.contains("Seconds")));
        assert!(output.lines.iter().any(|line| line.contains('8')));
        assert!(output.lines.iter().any(|line| line.contains('1')));
        assert!(output.lines.iter().any(|line| line.contains('2')));
        assert!(output.lines.iter().any(|line| line.contains('6')));
        assert!(
            output
                .lines
                .iter()
                .any(|line| line.contains("48;2;242;80;141"))
        );
        assert!(
            output
                .lines
                .iter()
                .any(|line| line.contains("48;2;255;255;255"))
        );
    }

    #[test]
    fn bcd_panel_uses_standard_clock_column_heights() {
        let output = render_bcd_panel(
            bcd_digits(12, 34, 56),
            BCD_LIT_COLOR,
            LAMP_OFF_COLOR,
            BCD_LIT_COLOR,
            1,
        );

        assert_eq!(background_count(&output.lines[1]), 3);
        assert_eq!(background_count(&output.lines[2]), 5);
        assert_eq!(background_count(&output.lines[3]), 6);
        assert_eq!(background_count(&output.lines[4]), 6);
    }

    #[test]
    fn bcd_panel_aligns_group_separators_across_header_body_and_footer() {
        let output = render_bcd_panel(
            bcd_digits(12, 34, 56),
            BCD_LIT_COLOR,
            LAMP_OFF_COLOR,
            BCD_LIT_COLOR,
            1,
        );

        let header_separators = visible_positions_of(&output.lines[0], '│');
        assert_eq!(
            header_separators,
            visible_positions_of(&output.lines[1], '│')
        );
        assert_eq!(
            header_separators,
            visible_positions_of(&output.lines[4], '│')
        );
        assert_eq!(
            header_separators,
            visible_positions_of(&output.lines[5], '│')
        );
    }

    #[test]
    fn fixed_lamp_views_cap_their_scale() {
        let time = NaiveTime::from_hms_opt(12, 34, 56).expect("time should be valid");
        let viewport = Viewport::new(240, 80);
        let theme = Theme::default();

        let mut matrix_renderer = RadixClockRenderer::default();
        matrix_renderer.cycle_binary_style();
        let matrix_output = matrix_renderer.render(time, viewport, &theme);
        assert_eq!(matrix_output.lines.len(), 16);

        let mut bcd_renderer = RadixClockRenderer::default();
        bcd_renderer.cycle_binary_style();
        bcd_renderer.cycle_binary_style();
        let bcd_output = bcd_renderer.render(time, viewport, &theme);
        assert_eq!(bcd_output.lines.len(), 18);
    }

    #[test]
    fn fixed_binary_styles_ignore_theme_colors() {
        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let viewport = Viewport::new(120, 24);
        let theme_a = Theme::preset(0);
        let theme_b = Theme::preset(7);

        let mut matrix_renderer = RadixClockRenderer::default();
        matrix_renderer.cycle_binary_style();
        let matrix_a = matrix_renderer.render(time, viewport, &theme_a);
        let matrix_b = matrix_renderer.render(time, viewport, &theme_b);
        assert_eq!(matrix_a.lines, matrix_b.lines);

        let mut bcd_renderer = RadixClockRenderer::default();
        bcd_renderer.cycle_binary_style();
        bcd_renderer.cycle_binary_style();
        let bcd_a = bcd_renderer.render(time, viewport, &theme_a);
        let bcd_b = bcd_renderer.render(time, viewport, &theme_b);
        assert_eq!(bcd_a.lines, bcd_b.lines);
    }

    #[test]
    fn fixed_binary_styles_ignore_orientation() {
        let mut vertical_renderer = RadixClockRenderer::default();
        vertical_renderer.cycle_binary_style();

        let mut horizontal_renderer = vertical_renderer;
        horizontal_renderer.toggle_orientation();

        let time = NaiveTime::from_hms_opt(13, 5, 9).expect("time should be valid");
        let vertical_output =
            vertical_renderer.render(time, Viewport::new(120, 24), &Theme::default());
        let horizontal_output =
            horizontal_renderer.render(time, Viewport::new(120, 24), &Theme::default());

        assert_eq!(vertical_output.lines, horizontal_output.lines);

        vertical_renderer.cycle_binary_style();
        horizontal_renderer.cycle_binary_style();

        let vertical_output =
            vertical_renderer.render(time, Viewport::new(120, 24), &Theme::default());
        let horizontal_output =
            horizontal_renderer.render(time, Viewport::new(120, 24), &Theme::default());

        assert_eq!(vertical_output.lines, horizontal_output.lines);
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

    fn background_count(line: &str) -> usize {
        line.match_indices("48;2;").count()
    }

    fn visible_positions_of(line: &str, needle: char) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut width = 0;
        let mut chars = line.chars().peekable();

        while let Some(character) = chars.next() {
            if character == '\u{1b}' && matches!(chars.peek(), Some('[')) {
                chars.next();

                for codepoint in chars.by_ref() {
                    if ('@'..='~').contains(&codepoint) {
                        break;
                    }
                }

                continue;
            }

            if character == needle {
                positions.push(width);
            }

            width += 1;
        }

        positions
    }
}
