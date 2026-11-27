use super::super::theme::*;
use insta::assert_snapshot;
use std::path::{Path, PathBuf};

fn theme_test_dir(theme: String) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let theme_dir = root.join("src/input/unit/fixtures/themes");
    theme_dir.join(theme)
}
#[test]
fn dracula_theme_from_file() {
    let path = theme_test_dir("dracula.kdl".into());
    let theme = Themes::from_path(path).unwrap();
    assert_snapshot!(format!("{:#?}", theme));
}

#[test]
fn no_theme_is_err() {
    let path = theme_test_dir("nonexistent.kdl".into());
    let theme = Themes::from_path(path);
    assert!(theme.is_err());
}

/// WCAG relative luminance of a 24-bit palette color.
fn relative_luminance(color: crate::data::PaletteColor) -> f64 {
    let crate::data::PaletteColor::Rgb((r, g, b)) = color else {
        panic!("the product fallback theme is authored in 24-bit color");
    };
    let linear = |channel: u8| {
        let c = f64::from(channel) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn contrast(ink: crate::data::PaletteColor, ground: crate::data::PaletteColor) -> f64 {
    let (a, b) = (relative_luminance(ink), relative_luminance(ground));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// docs/THEMES_GUIDE.md assigns `emphasis_1` to the accent ("you are here":
/// the rail's current session and active tab) and `emphasis_2` to dim chrome
/// (ordinals, markers, separators). `monochrome` is the product's static
/// fallback, so it must honor both roles on the chrome ground and on the
/// block tint: the accent clearly brighter than ink, dim chrome clearly
/// quieter and still legible.
#[test]
fn product_fallback_theme_keeps_the_accent_and_dim_chrome_roles() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let themes = Themes::from_path(root.join("assets/themes/monochrome.kdl")).unwrap();
    let palette = themes.get_theme("monochrome").unwrap().palette;
    for (role, declaration) in [
        ("text_unselected", palette.text_unselected),
        ("text_selected", palette.text_selected),
    ] {
        let ink = contrast(declaration.base, declaration.background);
        let accent = contrast(declaration.emphasis_1, declaration.background);
        let dim = contrast(declaration.emphasis_2, declaration.background);
        assert!(
            accent >= ink * 1.25,
            "{role}: the accent (emphasis_1, {accent:.2}:1) must stand out from ink ({ink:.2}:1)"
        );
        assert!(
            dim <= ink * 0.75,
            "{role}: dim chrome (emphasis_2, {dim:.2}:1) must sit below ink ({ink:.2}:1)"
        );
        assert!(
            dim >= 3.0,
            "{role}: dim chrome (emphasis_2, {dim:.2}:1) must stay legible"
        );
    }
}

#[test]
fn default_themes_from_assets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let assets_dir = root.join("assets/themes");
    for entry in std::fs::read_dir(assets_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "kdl") {
            let theme = Themes::from_path(path);
            assert!(theme.is_ok(), "Failed to parse theme");
        }
    }
}
