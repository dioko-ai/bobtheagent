use std::fs;
use std::path::Path;

use ratatui::style::Color;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Theme {
    pub left_top_bg: Color,
    pub chat_bg: Color,
    pub right_bg: Color,
    pub input_bg: Color,
    pub status_bg: Color,
    pub text_fg: Color,
    pub muted_fg: Color,
    pub active_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            left_top_bg: Color::Rgb(44, 44, 44),
            chat_bg: Color::Rgb(54, 54, 54),
            right_bg: Color::Rgb(48, 48, 48),
            input_bg: Color::Rgb(62, 62, 62),
            status_bg: Color::Rgb(36, 36, 36),
            text_fg: Color::Rgb(225, 225, 225),
            muted_fg: Color::Rgb(185, 185, 185),
            active_fg: Color::Rgb(255, 255, 255),
        }
    }
}

impl Theme {
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        let path_ref = path.as_ref();
        match fs::read_to_string(path_ref) {
            Ok(contents) => match Self::from_toml_str(&contents) {
                Ok(theme) => theme,
                Err(err) => {
                    eprintln!(
                        "Failed to parse theme file '{}': {err}. Using defaults.",
                        path_ref.display()
                    );
                    Self::default()
                }
            },
            Err(err) => {
                eprintln!(
                    "Failed to read theme file '{}': {err}. Using defaults.",
                    path_ref.display()
                );
                Self::default()
            }
        }
    }

    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        let cfg: ThemeToml = toml::from_str(s)?;
        Ok(Self {
            left_top_bg: cfg.colors.left_top_bg.to_color(),
            chat_bg: cfg.colors.chat_bg.to_color(),
            right_bg: cfg.colors.right_bg.to_color(),
            input_bg: cfg.colors.input_bg.to_color(),
            status_bg: cfg.colors.status_bg.to_color(),
            text_fg: cfg.colors.text_fg.to_color(),
            muted_fg: cfg.colors.muted_fg.to_color(),
            active_fg: cfg.colors.active_fg.to_color(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct ThemeToml {
    colors: ThemeColorsToml,
}

#[derive(Debug, Deserialize)]
struct ThemeColorsToml {
    left_top_bg: RgbToml,
    chat_bg: RgbToml,
    right_bg: RgbToml,
    input_bg: RgbToml,
    status_bg: RgbToml,
    text_fg: RgbToml,
    muted_fg: RgbToml,
    active_fg: RgbToml,
}

#[derive(Debug, Deserialize)]
struct RgbToml {
    r: u8,
    g: u8,
    b: u8,
}

impl RgbToml {
    fn to_color(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }
}

#[cfg(test)]
#[path = "../tests/unit/theme_tests.rs"]
mod tests;
