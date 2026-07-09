use anyhow::{Context, Result};
use std::path::PathBuf;

use super::types::{CategoryMap, CommandMap};
use crate::theme::Theme;

#[derive(Debug, Default, serde::Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub theme: Option<Theme>,
    #[serde(default)]
    pub apps: CategoryMap,
    /// Dynamic categories sourced from a command's JSON stdout. A static
    /// `[[apps.<name>]]` of the same name takes precedence.
    #[serde(default)]
    pub commands: CommandMap,
}

pub fn find_config(explicit: Option<&str>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        let expanded = shellexpand::tilde(path);
        return Ok(PathBuf::from(expanded.as_ref()));
    }

    let candidate = dirs::config_dir().map(|p| p.join("hyprgrid/config.toml"));

    if let Some(path) = candidate.filter(|p| p.exists()) {
        return Ok(path);
    }

    anyhow::bail!("No config found. Create ~/.config/hyprgrid/config.toml or use --config")
}

pub fn load_config(path: &PathBuf) -> Result<ConfigFile> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
}
