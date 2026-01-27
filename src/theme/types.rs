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
        "#076678".into(), // gruvbox faded blue
        "#8f3f71".into(), // gruvbox faded purple
        "#427b58".into(), // gruvbox faded aqua
        "#79740e".into(), // gruvbox faded green
        "#b57614".into(), // gruvbox faded yellow
        "#af3a03".into(), // gruvbox faded orange
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
