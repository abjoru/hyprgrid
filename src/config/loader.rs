use anyhow::{Context, Result};
use std::path::PathBuf;

use super::types::AppsConfig;
use crate::theme::Theme;

#[derive(Debug, Default, serde::Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub theme: Option<Theme>,
}

pub fn find_apps_config(explicit: Option<&str>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        let expanded = shellexpand::tilde(path);
        return Ok(PathBuf::from(expanded.as_ref()));
    }

    let candidates = [
        dirs::config_dir().map(|p| p.join("hyprgrid/apps.yaml")),
        dirs::config_dir().map(|p| p.join("xmonad/applications.yaml")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    anyhow::bail!("No apps config found. Create ~/.config/hyprgrid/apps.yaml or use --config")
}

pub fn find_theme_config() -> Option<PathBuf> {
    dirs::config_dir()
        .map(|p| p.join("hyprgrid/config.yaml"))
        .filter(|p| p.exists())
}

pub fn load_apps(path: &PathBuf) -> Result<AppsConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    serde_yaml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
}

pub fn load_theme() -> Theme {
    find_theme_config()
        .and_then(|path| {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_yaml::from_str::<ConfigFile>(&content).ok())
                .and_then(|cfg| cfg.theme)
        })
        .unwrap_or_default()
}
