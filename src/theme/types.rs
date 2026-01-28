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
        }
    }
}
