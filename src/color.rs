use crossterm::style::Color as TerminalColor;
use palette::{FromColor, Hsl as PaletteHsl, ShiftHue, Srgb as PaletteSrgb};
use std::error::Error;
use std::fmt;

const DEFAULT_MIN_CONTRAST: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const WHITE: Self = Self::new(0xFF, 0xFF, 0xFF);
    pub const BLACK: Self = Self::new(0x00, 0x00, 0x00);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(input: &str) -> Result<Self, HexColorError> {
        let hex = input.trim();

        if hex.is_empty() {
            return Err(HexColorError::Empty);
        }

        let hex = hex.strip_prefix('#').unwrap_or(hex);

        match hex.len() {
            3 => {
                let mut chars = hex.chars();
                let r = duplicated_hex_pair(chars.next().expect("length should be 3"))?;
                let g = duplicated_hex_pair(chars.next().expect("length should be 3"))?;
                let b = duplicated_hex_pair(chars.next().expect("length should be 3"))?;

                Ok(Self::new(r, g, b))
            }
            6 => {
                let r = parse_hex_byte(&hex[0..2])?;
                let g = parse_hex_byte(&hex[2..4])?;
                let b = parse_hex_byte(&hex[4..6])?;

                Ok(Self::new(r, g, b))
            }
            length => Err(HexColorError::InvalidLength(length)),
        }
    }

    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn to_hsl(self) -> Hsl {
        rgb_to_hsl(self)
    }

    pub fn from_hsl(hsl: Hsl) -> Self {
        hsl_to_rgb(hsl)
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl Hsl {
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self {
            h: normalize_hue(h),
            s: s.clamp(0.0, 1.0),
            l: l.clamp(0.0, 1.0),
        }
    }

    pub fn rotate_hue(self, degrees: f32) -> Self {
        Self::from_palette(self.to_palette().shift_hue(degrees))
    }

    pub fn with_saturation(self, s: f32) -> Self {
        Self::new(self.h, s, self.l)
    }

    pub fn with_lightness(self, l: f32) -> Self {
        Self::new(self.h, self.s, l)
    }

    fn to_palette(self) -> PaletteHsl {
        PaletteHsl::new_srgb(self.h, self.s, self.l)
    }

    fn from_palette(hsl: PaletteHsl) -> Self {
        Self::new(
            hsl.hue.into_positive_degrees(),
            hsl.saturation,
            hsl.lightness,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexColorError {
    Empty,
    InvalidLength(usize),
    InvalidHex(String),
}

impl fmt::Display for HexColorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("hex color cannot be empty"),
            Self::InvalidLength(length) => {
                write!(formatter, "hex color must use 3 or 6 digits, got {length}")
            }
            Self::InvalidHex(value) => write!(formatter, "invalid hex color component: {value}"),
        }
    }
}

impl Error for HexColorError {}

pub fn rgb_to_hsl(rgb: Rgb) -> Hsl {
    let palette_rgb: PaletteSrgb<f32> = PaletteSrgb::new(rgb.r, rgb.g, rgb.b).into_format();
    let palette_hsl = PaletteHsl::from_color(palette_rgb);

    Hsl::from_palette(palette_hsl)
}

pub fn hsl_to_rgb(hsl: Hsl) -> Rgb {
    let palette_rgb: PaletteSrgb<f32> = PaletteSrgb::from_color(hsl.to_palette());
    let palette_rgb: PaletteSrgb<u8> = palette_rgb.into_format();

    Rgb::new(palette_rgb.red, palette_rgb.green, palette_rgb.blue)
}

pub fn normalize_hue(hue: f32) -> f32 {
    hue.rem_euclid(360.0)
}

pub fn relative_luminance(rgb: Rgb) -> f32 {
    let r = luminance_channel(rgb.r);
    let g = luminance_channel(rgb.g);
    let b = luminance_channel(rgb.b);

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

pub fn contrast_ratio(left: Rgb, right: Rgb) -> f32 {
    let left = relative_luminance(left);
    let right = relative_luminance(right);
    let (lighter, darker) = if left >= right {
        (left, right)
    } else {
        (right, left)
    };

    (lighter + 0.05) / (darker + 0.05)
}

pub fn ensure_contrast(fg: Rgb, bg: Rgb) -> Rgb {
    ensure_min_contrast(fg, bg, DEFAULT_MIN_CONTRAST)
}

pub fn ensure_min_contrast(fg: Rgb, bg: Rgb, min_ratio: f32) -> Rgb {
    if contrast_ratio(fg, bg) >= min_ratio {
        return fg;
    }

    let source = fg.to_hsl();
    let should_lighten = relative_luminance(bg) < 0.28;
    let saturation = source.s.max(0.12);
    let target_lightness = if should_lighten { 0.96 } else { 0.08 };

    for step in 0..=24 {
        let progress = step as f32 / 24.0;
        let lightness = lerp(source.l, target_lightness, progress);
        let candidate = Rgb::from_hsl(Hsl::new(source.h, saturation, lightness));

        if contrast_ratio(candidate, bg) >= min_ratio {
            return candidate;
        }
    }

    [Rgb::WHITE, Rgb::BLACK]
        .into_iter()
        .max_by(|left, right| {
            contrast_ratio(*left, bg)
                .partial_cmp(&contrast_ratio(*right, bg))
                .expect("contrast ratios should be comparable")
        })
        .expect("black and white candidates should exist")
}

pub fn to_terminal_color(rgb: &Rgb) -> TerminalColor {
    TerminalColor::Rgb {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
    }
}

pub fn paint_foreground(text: &str, fg: Rgb) -> String {
    format!("\u{1b}[38;2;{};{};{}m{text}\u{1b}[39m", fg.r, fg.g, fg.b)
}

pub fn paint_sample(text: &str, fg: Rgb, bg: Rgb) -> String {
    format!(
        "\u{1b}[38;2;{};{};{}m\u{1b}[48;2;{};{};{}m{text}\u{1b}[0m",
        fg.r, fg.g, fg.b, bg.r, bg.g, bg.b
    )
}

fn duplicated_hex_pair(character: char) -> Result<u8, HexColorError> {
    let pair = format!("{character}{character}");
    parse_hex_byte(&pair)
}

fn parse_hex_byte(component: &str) -> Result<u8, HexColorError> {
    u8::from_str_radix(component, 16).map_err(|_| HexColorError::InvalidHex(component.to_string()))
}

fn luminance_channel(channel: u8) -> f32 {
    let value = channel as f32 / 255.0;

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + (end - start) * amount.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        HexColorError, Hsl, Rgb, contrast_ratio, ensure_contrast, hsl_to_rgb, normalize_hue,
        relative_luminance, rgb_to_hsl,
    };

    fn approx_eq(left: f32, right: f32, tolerance: f32) {
        assert!(
            (left - right).abs() <= tolerance,
            "expected {left} to be within {tolerance} of {right}"
        );
    }

    #[test]
    fn parses_hex_with_or_without_hash() {
        assert_eq!(
            Rgb::from_hex("#3b82f6").expect("hex should parse"),
            Rgb::new(0x3B, 0x82, 0xF6)
        );
        assert_eq!(
            Rgb::from_hex("fff").expect("short hex should parse"),
            Rgb::new(0xFF, 0xFF, 0xFF)
        );
    }

    #[test]
    fn rejects_invalid_hex_inputs() {
        assert_eq!(Rgb::from_hex(""), Err(HexColorError::Empty));
        assert_eq!(Rgb::from_hex("#1234"), Err(HexColorError::InvalidLength(4)));

        let error = Rgb::from_hex("#zzzzzz").expect_err("hex should be invalid");
        assert!(matches!(error, HexColorError::InvalidHex(_)));
    }

    #[test]
    fn formats_hex_in_uppercase() {
        let rgb = Rgb::new(0x3B, 0x82, 0xF6);

        assert_eq!(rgb.to_hex(), "#3B82F6");
        assert_eq!(rgb.to_string(), "#3B82F6");
    }

    #[test]
    fn converts_rgb_to_hsl_for_blue() {
        let hsl = rgb_to_hsl(Rgb::new(0x3B, 0x82, 0xF6));

        approx_eq(hsl.h, 217.0, 0.5);
        approx_eq(hsl.s, 0.91, 0.02);
        approx_eq(hsl.l, 0.60, 0.02);
    }

    #[test]
    fn converts_hsl_back_to_rgb() {
        let rgb = hsl_to_rgb(Hsl::new(217.0, 0.91, 0.60));

        assert!((rgb.r as i16 - 0x3B).abs() <= 1);
        assert!((rgb.g as i16 - 0x82).abs() <= 1);
        assert!((rgb.b as i16 - 0xF6).abs() <= 1);
    }

    #[test]
    fn normalizes_hue_across_the_circle() {
        approx_eq(normalize_hue(390.0), 30.0, 0.001);
        approx_eq(normalize_hue(-30.0), 330.0, 0.001);
    }

    #[test]
    fn luminance_orders_light_and_dark_colors() {
        assert!(relative_luminance(Rgb::WHITE) > relative_luminance(Rgb::BLACK));
    }

    #[test]
    fn ensure_contrast_raises_low_contrast_foreground() {
        let bg = Rgb::new(0x08, 0x10, 0x1F);
        let fg = Rgb::new(0x18, 0x24, 0x36);

        let corrected = ensure_contrast(fg, bg);

        assert!(contrast_ratio(corrected, bg) >= 4.0);
    }
}
