use ratatui::style::Color;
use serde_derive::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct TomlColors {
    accent: Option<String>,
    cursor: Option<String>,
    foreground: Option<String>,
    background: Option<String>,
    selection_foreground: Option<String>,
    selection_background: Option<String>,
    color0: Option<String>,
    color1: Option<String>,
    color2: Option<String>,
    color3: Option<String>,
    color4: Option<String>,
    color5: Option<String>,
    color6: Option<String>,
    color7: Option<String>,
    color8: Option<String>,
    color9: Option<String>,
    color10: Option<String>,
    color11: Option<String>,
    color12: Option<String>,
    color13: Option<String>,
    color14: Option<String>,
    color15: Option<String>,
}

#[derive(Clone)]
pub struct OmarchyTheme {
    pub accent: Color,
    pub cursor: Color,
    pub foreground: Color,
    pub background: Color,
    pub selection_foreground: Color,
    pub selection_background: Color,

    // Normal colors
    pub color0: Color,
    pub color1: Color,
    pub color2: Color,
    pub color3: Color,
    pub color4: Color,
    pub color5: Color,
    pub color6: Color,
    pub color7: Color,

    // Bright colors
    pub color8: Color,
    pub color9: Color,
    pub color10: Color,
    pub color11: Color,
    pub color12: Color,
    pub color13: Color,
    pub color14: Color,
    pub color15: Color,
}

impl Default for OmarchyTheme {
    fn default() -> Self {
        // Fallback to Tokyo Night style if Omarchy file is missing
        Self {
            accent: Color::Rgb(122, 162, 247),     // #7aa2f7
            cursor: Color::Rgb(192, 202, 245),     // #c0caf5
            foreground: Color::Rgb(169, 177, 214), // #a9b1d6
            background: Color::Rgb(26, 27, 38),    // #1a1b26
            selection_foreground: Color::Rgb(192, 202, 245),
            selection_background: Color::Rgb(122, 162, 247),

            color0: Color::Rgb(50, 52, 74),    // #32344a
            color1: Color::Rgb(247, 118, 142), // #f7768e
            color2: Color::Rgb(158, 206, 106), // #9ece6a
            color3: Color::Rgb(224, 175, 104), // #e0af68
            color4: Color::Rgb(122, 162, 247), // #7aa2f7
            color5: Color::Rgb(173, 142, 230), // #ad8ee6
            color6: Color::Rgb(68, 157, 171),  // #449dab
            color7: Color::Rgb(120, 124, 153), // #787c99

            color8: Color::Rgb(68, 75, 106),    // #444b6a
            color9: Color::Rgb(255, 122, 147),  // #ff7a93
            color10: Color::Rgb(185, 242, 124), // #b9f27c
            color11: Color::Rgb(255, 158, 100), // #ff9e64
            color12: Color::Rgb(125, 166, 255), // #7da6ff
            color13: Color::Rgb(187, 154, 247), // #bb9af7
            color14: Color::Rgb(13, 185, 215),  // #0db9d7
            color15: Color::Rgb(172, 176, 208), // #acb0d0
        }
    }
}

impl OmarchyTheme {
    pub fn load() -> Self {
        let mut default = Self::default();

        let path = dirs::home_dir().map(|mut p| {
            p.push(".config");
            p.push("omarchy");
            p.push("current");
            p.push("theme");
            p.push("colors.toml");
            p
        });

        if let Some(p) = path
            && let Ok(content) = fs::read_to_string(p)
                && let Ok(toml_data) = toml::from_str::<TomlColors>(&content) {
                    Self::apply_if_some(&mut default.accent, toml_data.accent);
                    Self::apply_if_some(&mut default.cursor, toml_data.cursor);
                    Self::apply_if_some(&mut default.foreground, toml_data.foreground);
                    Self::apply_if_some(&mut default.background, toml_data.background);
                    Self::apply_if_some(
                        &mut default.selection_foreground,
                        toml_data.selection_foreground,
                    );
                    Self::apply_if_some(
                        &mut default.selection_background,
                        toml_data.selection_background,
                    );

                    Self::apply_if_some(&mut default.color0, toml_data.color0);
                    Self::apply_if_some(&mut default.color1, toml_data.color1);
                    Self::apply_if_some(&mut default.color2, toml_data.color2);
                    Self::apply_if_some(&mut default.color3, toml_data.color3);
                    Self::apply_if_some(&mut default.color4, toml_data.color4);
                    Self::apply_if_some(&mut default.color5, toml_data.color5);
                    Self::apply_if_some(&mut default.color6, toml_data.color6);
                    Self::apply_if_some(&mut default.color7, toml_data.color7);

                    Self::apply_if_some(&mut default.color8, toml_data.color8);
                    Self::apply_if_some(&mut default.color9, toml_data.color9);
                    Self::apply_if_some(&mut default.color10, toml_data.color10);
                    Self::apply_if_some(&mut default.color11, toml_data.color11);
                    Self::apply_if_some(&mut default.color12, toml_data.color12);
                    Self::apply_if_some(&mut default.color13, toml_data.color13);
                    Self::apply_if_some(&mut default.color14, toml_data.color14);
                    Self::apply_if_some(&mut default.color15, toml_data.color15);
                }

        default
    }

    fn apply_if_some(color: &mut Color, hex: Option<String>) {
        if let Some(h) = hex
            && let Some(parsed) = Self::parse_hex(&h) {
                *color = parsed;
            }
    }

    fn parse_hex(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }
}
