    use super::*;

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
