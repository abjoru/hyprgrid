use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    #[allow(dead_code)]
    pub bg: String,
    #[allow(dead_code)]
    pub bg_selected: String,
    pub border: String,
    pub fg: String,
    pub fg_dim: String,
    // Cell accent colors (cycle through these)
    #[serde(default = "default_accents")]
    pub accents: Vec<String>,
    #[serde(default = "default_icon_size")]
    pub icon_size: u32,
    #[serde(default = "default_icons_enabled")]
    pub icons_enabled: bool,
    /// How much to dim unselected cells (0.0 = no dim, 1.0 = fully desaturated)
    #[serde(default = "default_dim_strength")]
    pub dim_strength: f64,
}

fn default_icon_size() -> u32 {
    32
}

fn default_icons_enabled() -> bool {
    true
}

fn default_dim_strength() -> f64 {
    0.8
}

fn default_accents() -> Vec<String> {
    vec![
        "#043d48".into(), // gruvbox dark blue
        "#562544".into(), // gruvbox dark purple
        "#284935".into(), // gruvbox dark aqua
        "#494608".into(), // gruvbox dark green
        "#6d470c".into(), // gruvbox dark yellow
        "#692302".into(), // gruvbox dark orange
    ]
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: "#282828".into(),
            bg_selected: "#3c3836".into(),
            border: "#d79921".into(), // gruvbox yellow
            fg: "#ebdbb2".into(),
            fg_dim: "#a89984".into(),
            accents: default_accents(),
            icon_size: default_icon_size(),
            icons_enabled: default_icons_enabled(),
            dim_strength: default_dim_strength(),
        }
    }
}

/// Parse a hex color (#RRGGBB) into (r, g, b) components.
fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Desaturate a hex color by `strength` (0.0 = original, 1.0 = fully gray).
pub fn desaturate(hex: &str, strength: f64) -> String {
    let Some((r, g, b)) = parse_hex(hex) else {
        return hex.to_string();
    };
    let gray = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
    let s = strength.clamp(0.0, 1.0);
    let r2 = (r as f64 * (1.0 - s) + gray * s).round() as u8;
    let g2 = (g as f64 * (1.0 - s) + gray * s).round() as u8;
    let b2 = (b as f64 * (1.0 - s) + gray * s).round() as u8;
    format!("#{r2:02x}{g2:02x}{b2:02x}")
}
