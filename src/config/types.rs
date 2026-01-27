use serde_yaml::Value;
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
}

/// YAML structure: { "category": [ { "id": null, "name": "...", "command"/"terminal": "..." } ] }
pub type AppsConfig = HashMap<String, Vec<HashMap<String, Value>>>;

#[derive(Debug, Clone)]
pub struct FlatEntry {
    pub id: String,
    pub entry: AppEntry,
}

impl FlatEntry {
    pub fn flatten(config: &AppsConfig, category: &str) -> Vec<FlatEntry> {
        let Some(apps) = config.get(category) else {
            return vec![];
        };

        apps.iter()
            .filter_map(|item| Self::parse_item(item))
            .collect()
    }

    fn parse_item(item: &HashMap<String, Value>) -> Option<FlatEntry> {
        // Find the ID key (the one with null/empty value or first key that's not a known field)
        let known_fields = ["name", "description", "command", "terminal"];
        let id = item
            .iter()
            .find(|(k, _)| !known_fields.contains(&k.as_str()))
            .map(|(k, _)| k.clone())?;

        let name = item.get("name").and_then(|v| v.as_str())?.to_string();
        let description = item.get("description").and_then(|v| v.as_str().map(String::from));

        let launch = if let Some(cmd) = item.get("terminal").and_then(|v| v.as_str()) {
            LaunchType::Terminal(cmd.to_string())
        } else if let Some(cmd) = item.get("command").and_then(|v| v.as_str()) {
            LaunchType::Command(cmd.to_string())
        } else {
            return None;
        };

        Some(FlatEntry {
            id,
            entry: AppEntry {
                name,
                description,
                launch,
            },
        })
    }
}
