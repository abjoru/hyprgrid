use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum LaunchType {
    Terminal(String),
    Command(String),
}

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub description: Option<String>,
    pub launch: LaunchType,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    pub command: String,
    #[serde(default)]
    pub terminal: bool,
}

pub type AppsConfig = HashMap<String, Vec<AppDef>>;

#[derive(Debug, Clone)]
pub struct FlatEntry {
    #[allow(dead_code)]
    pub id: String,
    pub entry: AppEntry,
}

impl FlatEntry {
    pub fn flatten(config: &AppsConfig, category: &str) -> Vec<FlatEntry> {
        let Some(apps) = config.get(category) else {
            return vec![];
        };

        apps.iter()
            .map(|app| {
                let launch = if app.terminal {
                    LaunchType::Terminal(app.command.clone())
                } else {
                    LaunchType::Command(app.command.clone())
                };

                FlatEntry {
                    id: app.id.clone(),
                    entry: AppEntry {
                        name: app.name.clone(),
                        description: app.description.clone(),
                        launch,
                        icon: app.icon.clone(),
                    },
                }
            })
            .collect()
    }
}
