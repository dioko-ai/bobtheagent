use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn parses_theme_from_toml() {
    let input = r#"
[colors]
left_top_bg = { r = 1, g = 2, b = 3 }
chat_bg = { r = 4, g = 5, b = 6 }
right_bg = { r = 7, g = 8, b = 9 }
input_bg = { r = 10, g = 11, b = 12 }
status_bg = { r = 13, g = 14, b = 15 }
text_fg = { r = 16, g = 17, b = 18 }
muted_fg = { r = 19, g = 20, b = 21 }
active_fg = { r = 22, g = 23, b = 24 }
"#;

    let theme = Theme::from_toml_str(input).expect("theme should parse");
    assert_eq!(theme.left_top_bg, Color::Rgb(1, 2, 3));
    assert_eq!(theme.chat_bg, Color::Rgb(4, 5, 6));
    assert_eq!(theme.right_bg, Color::Rgb(7, 8, 9));
    assert_eq!(theme.input_bg, Color::Rgb(10, 11, 12));
    assert_eq!(theme.status_bg, Color::Rgb(13, 14, 15));
    assert_eq!(theme.text_fg, Color::Rgb(16, 17, 18));
    assert_eq!(theme.muted_fg, Color::Rgb(19, 20, 21));
    assert_eq!(theme.active_fg, Color::Rgb(22, 23, 24));
}

#[test]
fn uses_default_on_missing_file() {
    let theme = Theme::load_or_default("/definitely-not-a-real-theme-file.toml");
    assert_eq!(theme.left_top_bg, Theme::default().left_top_bg);
}

#[test]
fn load_or_default_round_trips_valid_theme_file() {
    let path = std::env::temp_dir().join(format!(
        "metaagent-theme-{}.toml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos()
    ));
    fs::write(
        &path,
        r#"
[colors]
left_top_bg = { r = 11, g = 12, b = 13 }
chat_bg = { r = 14, g = 15, b = 16 }
right_bg = { r = 17, g = 18, b = 19 }
input_bg = { r = 20, g = 21, b = 22 }
status_bg = { r = 23, g = 24, b = 25 }
text_fg = { r = 26, g = 27, b = 28 }
muted_fg = { r = 29, g = 30, b = 31 }
active_fg = { r = 32, g = 33, b = 34 }
"#,
    )
    .expect("write theme file");

    let theme = Theme::load_or_default(&path);
    assert_eq!(theme.left_top_bg, Color::Rgb(11, 12, 13));
    assert_eq!(theme.active_fg, Color::Rgb(32, 33, 34));

    let _ = fs::remove_file(path);
}

#[test]
fn load_or_default_falls_back_for_malformed_theme_file() {
    let path = std::env::temp_dir().join(format!(
        "metaagent-theme-bad-{}.toml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos()
    ));
    fs::write(&path, "[colors]\nleft_top_bg = { r = \"nope\" }\n").expect("write bad theme");

    let theme = Theme::load_or_default(&path);
    assert_eq!(theme.chat_bg, Theme::default().chat_bg);

    let _ = fs::remove_file(path);
}
