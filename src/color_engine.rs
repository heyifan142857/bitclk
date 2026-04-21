use crate::color::{Hsl, Rgb, ensure_contrast, ensure_min_contrast};
use crate::theme::Theme;
use clap::ValueEnum;
use std::fmt;

const FOREGROUND_CONTRAST: f32 = 5.5;
const SIGNAL_CONTRAST: f32 = 3.2;
const ACCENT_CONTRAST: f32 = 4.0;
const MUTED_CONTRAST: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ColorHarmonyMode {
    Complementary,
    Analogous,
    Triadic,
    SplitComplementary,
}

impl ColorHarmonyMode {
    pub const ALL: [Self; 4] = [
        Self::Complementary,
        Self::Analogous,
        Self::Triadic,
        Self::SplitComplementary,
    ];

    pub fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|mode| *mode == self)
            .expect("current mode should always exist");

        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn accent_offsets(self) -> (f32, f32) {
        match self {
            Self::Complementary => (180.0, 180.0),
            Self::Analogous => (-30.0, 30.0),
            Self::Triadic => (120.0, 240.0),
            Self::SplitComplementary => (150.0, 210.0),
        }
    }
}

impl fmt::Display for ColorHarmonyMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Complementary => "complementary",
            Self::Analogous => "analogous",
            Self::Triadic => "triadic",
            Self::SplitComplementary => "split-complementary",
        })
    }
}

pub fn generate_theme(base: Rgb, mode: ColorHarmonyMode) -> Theme {
    let base_hsl = base.to_hsl();
    let background = build_background(base_hsl);
    let foreground =
        ensure_min_contrast(build_foreground(base_hsl), background, FOREGROUND_CONTRAST);
    let muted = ensure_min_contrast(build_muted(base_hsl), background, MUTED_CONTRAST);
    let (secondary_offset, accent_offset) = mode.accent_offsets();

    let primary = ensure_min_contrast(
        build_signal_color(base_hsl, 0.0, SignalRole::Primary),
        background,
        SIGNAL_CONTRAST,
    );
    let secondary = ensure_min_contrast(
        build_signal_color(base_hsl, secondary_offset, SignalRole::Secondary),
        background,
        SIGNAL_CONTRAST,
    );
    let accent = ensure_min_contrast(
        ensure_contrast(
            build_signal_color(base_hsl, accent_offset, SignalRole::Accent),
            background,
        ),
        background,
        ACCENT_CONTRAST,
    );

    Theme::new(primary, secondary, accent, background, foreground, muted)
}

#[derive(Debug, Clone, Copy)]
enum SignalRole {
    Primary,
    Secondary,
    Accent,
}

fn build_signal_color(base: Hsl, hue_offset: f32, role: SignalRole) -> Rgb {
    let shifted = base.rotate_hue(hue_offset);
    let saturation = match role {
        SignalRole::Primary => shifted.s.clamp(0.55, 0.92),
        SignalRole::Secondary => (shifted.s * 0.90 + 0.08).clamp(0.45, 0.86),
        SignalRole::Accent => (shifted.s * 0.95 + 0.10).clamp(0.58, 0.96),
    };
    let lightness = match role {
        SignalRole::Primary => shifted.l.max(0.60).clamp(0.56, 0.74),
        SignalRole::Secondary => (shifted.l * 0.92 + 0.08).clamp(0.52, 0.70),
        SignalRole::Accent => shifted.l.max(0.66).clamp(0.62, 0.80),
    };

    Rgb::from_hsl(
        shifted
            .with_saturation(saturation)
            .with_lightness(lightness),
    )
}

fn build_background(base: Hsl) -> Rgb {
    let saturation = (base.s * 0.22).clamp(0.08, 0.20);
    let lightness = (0.06 + base.l * 0.06).clamp(0.07, 0.13);

    Rgb::from_hsl(base.with_saturation(saturation).with_lightness(lightness))
}

fn build_foreground(base: Hsl) -> Rgb {
    let saturation = (base.s * 0.10).clamp(0.02, 0.12);
    let lightness = 0.92;

    Rgb::from_hsl(base.with_saturation(saturation).with_lightness(lightness))
}

fn build_muted(base: Hsl) -> Rgb {
    let saturation = (base.s * 0.18).clamp(0.08, 0.24);
    let lightness = (0.58 + base.l * 0.08).clamp(0.58, 0.72);

    Rgb::from_hsl(base.with_saturation(saturation).with_lightness(lightness))
}

#[cfg(test)]
mod tests {
    use super::{ColorHarmonyMode, generate_theme};
    use crate::color::{Rgb, contrast_ratio};

    #[test]
    fn analogous_offsets_wrap_around_the_hue_circle() {
        let theme = generate_theme(
            Rgb::from_hsl(crate::color::Hsl::new(350.0, 0.7, 0.55)),
            ColorHarmonyMode::Analogous,
        );

        assert_ne!(theme.primary, theme.secondary);
        assert_ne!(theme.primary, theme.accent);
    }

    #[test]
    fn generated_theme_stays_readable_on_dark_background() {
        let base = Rgb::from_hex("#3b82f6").expect("hex should parse");
        let theme = generate_theme(base, ColorHarmonyMode::Triadic);

        assert!(contrast_ratio(theme.foreground, theme.background) >= 5.5);
        assert!(contrast_ratio(theme.accent, theme.background) >= 4.0);
        assert!(contrast_ratio(theme.primary, theme.background) >= 3.2);
    }

    #[test]
    fn split_complementary_produces_distinct_secondary_and_accent() {
        let base = Rgb::from_hex("#f97316").expect("hex should parse");
        let theme = generate_theme(base, ColorHarmonyMode::SplitComplementary);

        assert_ne!(theme.secondary, theme.accent);
    }

    #[test]
    fn cycle_order_covers_all_modes() {
        assert_eq!(
            ColorHarmonyMode::Complementary.next(),
            ColorHarmonyMode::Analogous
        );
        assert_eq!(
            ColorHarmonyMode::SplitComplementary.next(),
            ColorHarmonyMode::Complementary
        );
    }
}
