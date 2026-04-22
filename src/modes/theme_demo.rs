use crate::app::AppResult;
use crate::cli::ThemeArgs;
use crate::color::{Rgb, contrast_ratio, paint_sample};
use crate::color_engine::ColorHarmonyMode;
use crate::theme::Theme;

pub fn run(args: ThemeArgs, mode: ColorHarmonyMode) -> AppResult {
    let base = Rgb::from_hex(&args.base)?;
    let theme = Theme::from_base(base, mode);
    let mode = mode.to_string();

    print_theme_demo(base, &mode, theme);
    Ok(())
}

fn print_theme_demo(base: Rgb, mode: &str, theme: Theme) {
    println!("bitclk theme demo");
    println!("base color: {base}");
    println!("mode: {mode}");
    println!();
    println!("generated theme");

    for (name, usage, color) in theme.roles() {
        println!(
            "{:<12} {:<8} {:<28} {}",
            name,
            color,
            usage,
            role_swatch(name, color)
        );
    }

    println!();
    println!("preview");
    println!(
        "{}",
        paint_sample("  bitclk preview  ", theme.foreground, theme.background)
    );
    println!("{}", clock_preview(theme));
    println!(
        "{} {}",
        paint_sample(" foreground ", theme.foreground, theme.background),
        paint_sample(" muted hint ", theme.muted, theme.background)
    );
}

fn role_swatch(label: &str, color: Rgb) -> String {
    let fg = readable_label_color(color);
    let text = format!(" {:^12} ", label);

    paint_sample(&text, fg, color)
}

fn clock_preview(theme: Theme) -> String {
    let [hours, minutes, seconds] = theme.clock_colors();

    format!(
        "{}{}{}{}{}{}",
        paint_sample(" 12 ", hours, theme.background),
        paint_sample(":", theme.muted, theme.background),
        paint_sample(" 34 ", minutes, theme.background),
        paint_sample(":", theme.muted, theme.background),
        paint_sample(" 56 ", seconds, theme.background),
        paint_sample("  q quit / t theme  ", theme.foreground, theme.background),
    )
}

fn readable_label_color(bg: Rgb) -> Rgb {
    if contrast_ratio(Rgb::WHITE, bg) >= contrast_ratio(Rgb::BLACK, bg) {
        Rgb::WHITE
    } else {
        Rgb::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::{clock_preview, role_swatch};
    use crate::theme::Theme;

    #[test]
    fn swatch_contains_ansi_color_escape() {
        let swatch = role_swatch("primary", Theme::default().primary);

        assert!(swatch.contains('\u{1b}'));
    }

    #[test]
    fn preview_mentions_runtime_theme_shortcut() {
        let preview = clock_preview(Theme::default());

        assert!(preview.contains("t theme"));
    }
}
