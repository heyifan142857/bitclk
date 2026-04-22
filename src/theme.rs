use crate::color::{HexColorError, Rgb, ensure_min_contrast, relative_luminance};
use crate::color_engine::{ColorHarmonyMode, generate_theme};

pub const DEFAULT_THEME_PRESET_INDEX: usize = 0;

const FOREGROUND_CONTRAST: f32 = 5.5;
const SIGNAL_CONTRAST: f32 = 3.2;
const ACCENT_CONTRAST: f32 = 4.0;
const MUTED_CONTRAST: f32 = 3.0;
const DEFAULT_THEME_PRESETS: [ThemePreset; 15] = [
    ThemePreset::new(
        Rgb::new(0x54, 0x6B, 0x41),
        Rgb::new(0xDC, 0xCC, 0xAC),
        Rgb::new(0xFF, 0xF8, 0xEC),
    ),
    ThemePreset::new(
        Rgb::new(0xBF, 0xA2, 0x8C),
        Rgb::new(0xF3, 0xE4, 0xC9),
        Rgb::new(0xBA, 0xBF, 0x94),
    ),
    ThemePreset::new(
        Rgb::new(0xFA, 0xAC, 0xBF),
        Rgb::new(0xFB, 0xC3, 0xC1),
        Rgb::new(0xFF, 0xEA, 0xBB),
    ),
    ThemePreset::new(
        Rgb::new(0xD2, 0x53, 0x53),
        Rgb::new(0x9E, 0x3B, 0x3B),
        Rgb::new(0xFF, 0xEA, 0xD3),
    ),
    ThemePreset::new(
        Rgb::new(0x00, 0x65, 0xF8),
        Rgb::new(0x00, 0xCA, 0xFF),
        Rgb::new(0x00, 0xFF, 0xDE),
    ),
    ThemePreset::new(
        Rgb::new(0xC0, 0x85, 0x52),
        Rgb::new(0x8C, 0x5A, 0x3C),
        Rgb::new(0x4B, 0x2E, 0x2B),
    ),
    ThemePreset::new(
        Rgb::new(0x84, 0x94, 0xFF),
        Rgb::new(0xC9, 0xBE, 0xFF),
        Rgb::new(0xFF, 0xDB, 0xFD),
    ),
    ThemePreset::new(
        Rgb::new(0xB0, 0xFF, 0xFA),
        Rgb::new(0xFF, 0x00, 0x87),
        Rgb::new(0xFF, 0x7D, 0xB0),
    ),
    ThemePreset::new(
        Rgb::new(0xEF, 0xE9, 0xE3),
        Rgb::new(0xD9, 0xCF, 0xC7),
        Rgb::new(0xC9, 0xB5, 0x9C),
    ),
    ThemePreset::new(
        Rgb::new(0x8A, 0xBE, 0xB9),
        Rgb::new(0x30, 0x56, 0x69),
        Rgb::new(0xC1, 0x78, 0x5A),
    ),
    ThemePreset::new(
        Rgb::new(0xCB, 0xF3, 0xBB),
        Rgb::new(0xAB, 0xE7, 0xB2),
        Rgb::new(0x93, 0xBF, 0xC7),
    ),
    ThemePreset::new(
        Rgb::new(0x8C, 0x00, 0xFF),
        Rgb::new(0xFF, 0x3F, 0x7F),
        Rgb::new(0xFF, 0xC4, 0x00),
    ),
    ThemePreset::new(
        Rgb::new(0xFA, 0xB1, 0x2F),
        Rgb::new(0xFA, 0x81, 0x2F),
        Rgb::new(0xDD, 0x03, 0x03),
    ),
    ThemePreset::new(
        Rgb::new(0x50, 0x58, 0x9C),
        Rgb::new(0x63, 0x6C, 0xCB),
        Rgb::new(0x6E, 0x8C, 0xFB),
    ),
    ThemePreset::new(
        Rgb::new(0x8D, 0x5F, 0x8C),
        Rgb::new(0xA3, 0x76, 0xA2),
        Rgb::new(0xDD, 0xC3, 0xC3),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThemePreset {
    primary: Rgb,
    secondary: Rgb,
    accent: Rgb,
}

impl ThemePreset {
    const fn new(primary: Rgb, secondary: Rgb, accent: Rgb) -> Self {
        Self {
            primary,
            secondary,
            accent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub primary: Rgb,
    pub secondary: Rgb,
    pub accent: Rgb,
    pub background: Rgb,
    pub foreground: Rgb,
    pub muted: Rgb,
}

impl Theme {
    pub const fn new(
        primary: Rgb,
        secondary: Rgb,
        accent: Rgb,
        background: Rgb,
        foreground: Rgb,
        muted: Rgb,
    ) -> Self {
        Self {
            primary,
            secondary,
            accent,
            background,
            foreground,
            muted,
        }
    }

    pub fn from_base(base: Rgb, mode: ColorHarmonyMode) -> Self {
        generate_theme(base, mode)
    }

    pub fn from_signal_colors(primary: Rgb, secondary: Rgb, accent: Rgb) -> Self {
        let signals = [primary, secondary, accent];
        let background = build_preset_background(signals);
        let foreground = ensure_min_contrast(
            build_preset_foreground(signals),
            background,
            FOREGROUND_CONTRAST,
        );
        let muted = ensure_min_contrast(build_preset_muted(signals), background, MUTED_CONTRAST);

        Self::new(
            ensure_min_contrast(primary, background, SIGNAL_CONTRAST),
            ensure_min_contrast(secondary, background, SIGNAL_CONTRAST),
            ensure_min_contrast(accent, background, ACCENT_CONTRAST),
            background,
            foreground,
            muted,
        )
    }

    pub fn preset(index: usize) -> Self {
        let preset = DEFAULT_THEME_PRESETS[index % DEFAULT_THEME_PRESETS.len()];

        Self::from_signal_colors(preset.primary, preset.secondary, preset.accent)
    }

    pub fn preset_count() -> usize {
        DEFAULT_THEME_PRESETS.len()
    }

    pub fn next_preset_index(index: usize) -> usize {
        (index + 1) % Self::preset_count()
    }

    pub fn clock_colors(&self) -> [Rgb; 3] {
        [self.primary, self.secondary, self.accent]
    }

    pub fn roles(&self) -> [(&'static str, &'static str, Rgb); 6] {
        [
            ("primary", "main clock emphasis", self.primary),
            ("secondary", "supporting time emphasis", self.secondary),
            ("accent", "active highlight / seconds", self.accent),
            ("background", "terminal canvas", self.background),
            ("foreground", "default text", self.foreground),
            ("muted", "secondary hints", self.muted),
        ]
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::preset(DEFAULT_THEME_PRESET_INDEX)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeThemeSource {
    Preset { index: usize },
    Generated { base: Rgb, mode: ColorHarmonyMode },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeTheme {
    source: RuntimeThemeSource,
    theme: Theme,
}

impl RuntimeTheme {
    pub fn from_options(
        theme_hex: Option<&str>,
        mode: ColorHarmonyMode,
    ) -> Result<Self, HexColorError> {
        match theme_hex {
            Some(hex) => {
                let base = Rgb::from_hex(hex)?;
                Ok(Self::generated(base, mode))
            }
            None => Ok(Self::preset(DEFAULT_THEME_PRESET_INDEX)),
        }
    }

    pub fn preset(index: usize) -> Self {
        Self {
            source: RuntimeThemeSource::Preset { index },
            theme: Theme::preset(index),
        }
    }

    pub fn generated(base: Rgb, mode: ColorHarmonyMode) -> Self {
        Self {
            source: RuntimeThemeSource::Generated { base, mode },
            theme: Theme::from_base(base, mode),
        }
    }

    pub fn theme(self) -> Theme {
        self.theme
    }

    #[cfg(test)]
    pub fn source(self) -> RuntimeThemeSource {
        self.source
    }

    pub fn cycle(&mut self) {
        match self.source {
            RuntimeThemeSource::Preset { index } => {
                let next_index = Theme::next_preset_index(index);
                self.source = RuntimeThemeSource::Preset { index: next_index };
                self.theme = Theme::preset(next_index);
            }
            RuntimeThemeSource::Generated { base, mode } => {
                let next_mode = mode.next();
                self.source = RuntimeThemeSource::Generated {
                    base,
                    mode: next_mode,
                };
                self.theme = Theme::from_base(base, next_mode);
            }
        }
    }

    pub fn help_label(self) -> String {
        match self.source {
            RuntimeThemeSource::Preset { index } => {
                format!("theme: preset {}/{}", index + 1, Theme::preset_count())
            }
            RuntimeThemeSource::Generated { base, mode } => {
                format!("theme: {} {}", base, mode)
            }
        }
    }

    pub fn cycle_label(self) -> &'static str {
        match self.source {
            RuntimeThemeSource::Preset { .. } => "cycle theme preset",
            RuntimeThemeSource::Generated { .. } => "cycle harmony mode",
        }
    }
}

fn build_preset_background(signals: [Rgb; 3]) -> Rgb {
    let anchor = darkest_color(signals);
    let hsl = anchor.to_hsl();
    let saturation = (hsl.s * 0.30 + 0.02).clamp(0.05, 0.18);
    let lightness = (hsl.l * 0.18 + 0.03).clamp(0.055, 0.15);

    Rgb::from_hsl(hsl.with_saturation(saturation).with_lightness(lightness))
}

fn build_preset_foreground(signals: [Rgb; 3]) -> Rgb {
    let anchor = lightest_color(signals);
    let hsl = anchor.to_hsl();
    let saturation = (hsl.s * 0.18).clamp(0.02, 0.16);

    Rgb::from_hsl(hsl.with_saturation(saturation).with_lightness(0.92))
}

fn build_preset_muted(signals: [Rgb; 3]) -> Rgb {
    let averaged = average_color(signals);
    let hsl = averaged.to_hsl();
    let saturation = (hsl.s * 0.20).clamp(0.05, 0.18);
    let lightness = 0.68;

    Rgb::from_hsl(hsl.with_saturation(saturation).with_lightness(lightness))
}

fn darkest_color(colors: [Rgb; 3]) -> Rgb {
    colors
        .into_iter()
        .min_by(|left, right| relative_luminance(*left).total_cmp(&relative_luminance(*right)))
        .expect("preset palette should contain colors")
}

fn lightest_color(colors: [Rgb; 3]) -> Rgb {
    colors
        .into_iter()
        .max_by(|left, right| relative_luminance(*left).total_cmp(&relative_luminance(*right)))
        .expect("preset palette should contain colors")
}

fn average_color(colors: [Rgb; 3]) -> Rgb {
    let (r, g, b) = colors
        .into_iter()
        .fold((0_u16, 0_u16, 0_u16), |(r, g, b), color| {
            (r + color.r as u16, g + color.g as u16, b + color.b as u16)
        });

    Rgb::new((r / 3) as u8, (g / 3) as u8, (b / 3) as u8)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_THEME_PRESET_INDEX, RuntimeTheme, RuntimeThemeSource, Theme};
    use crate::color::contrast_ratio;
    use crate::color_engine::ColorHarmonyMode;

    #[test]
    fn default_theme_uses_the_first_preset() {
        assert_eq!(Theme::default(), Theme::preset(DEFAULT_THEME_PRESET_INDEX));
    }

    #[test]
    fn every_preset_keeps_foreground_readable() {
        for index in 0..Theme::preset_count() {
            let theme = Theme::preset(index);

            assert!(contrast_ratio(theme.foreground, theme.background) >= 5.0);
            assert!(contrast_ratio(theme.accent, theme.background) >= 4.0);
            assert!(contrast_ratio(theme.primary, theme.background) >= 3.2);
        }
    }

    #[test]
    fn preset_cycle_wraps_back_to_the_beginning() {
        assert_eq!(Theme::next_preset_index(Theme::preset_count() - 1), 0);
    }

    #[test]
    fn runtime_theme_cycles_presets_when_no_custom_theme_is_set() {
        let mut runtime_theme =
            RuntimeTheme::from_options(None, ColorHarmonyMode::Triadic).expect("preset theme");

        runtime_theme.cycle();

        assert_eq!(
            runtime_theme.source(),
            RuntimeThemeSource::Preset { index: 1 }
        );
        assert_eq!(runtime_theme.theme(), Theme::preset(1));
    }

    #[test]
    fn runtime_theme_cycles_harmony_modes_when_custom_theme_is_set() {
        let mut runtime_theme =
            RuntimeTheme::from_options(Some("#3b82f6"), ColorHarmonyMode::Triadic)
                .expect("generated theme");

        runtime_theme.cycle();

        assert_eq!(
            runtime_theme.source(),
            RuntimeThemeSource::Generated {
                base: crate::color::Rgb::from_hex("#3b82f6").expect("hex should parse"),
                mode: ColorHarmonyMode::SplitComplementary,
            }
        );
        assert_eq!(
            runtime_theme.theme(),
            Theme::from_base(
                crate::color::Rgb::from_hex("#3b82f6").expect("hex should parse"),
                ColorHarmonyMode::SplitComplementary,
            )
        );
    }
}
