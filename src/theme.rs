use crate::color::Rgb;
use crate::color_engine::{ColorHarmonyMode, generate_theme};

pub const DEFAULT_THEME_BASE: Rgb = Rgb::new(0x3B, 0x82, 0xF6);
pub const DEFAULT_THEME_MODE: ColorHarmonyMode = ColorHarmonyMode::Triadic;

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
        Self::from_base(DEFAULT_THEME_BASE, DEFAULT_THEME_MODE)
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_THEME_BASE, DEFAULT_THEME_MODE, Theme};
    use crate::color::contrast_ratio;

    #[test]
    fn default_theme_uses_default_spec() {
        assert_eq!(
            Theme::default(),
            Theme::from_base(DEFAULT_THEME_BASE, DEFAULT_THEME_MODE)
        );
    }

    #[test]
    fn default_theme_keeps_foreground_readable() {
        let theme = Theme::default();

        assert!(contrast_ratio(theme.foreground, theme.background) >= 5.0);
        assert!(contrast_ratio(theme.accent, theme.background) >= 4.0);
    }
}
