use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub bg: String,
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
        "#458588".into(), // gruvbox blue
        "#b16286".into(), // gruvbox purple
        "#689d6a".into(), // gruvbox aqua
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
