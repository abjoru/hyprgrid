use serde::Deserialize;
use std::collections::HashMap;

/// TOML wire shape of an Entry: a flat `command` + `terminal` bool that
/// `From<EntryDef>` fuses into a `LaunchType`. Deserialization-only.
#[derive(Debug, Clone, Deserialize)]
pub struct EntryDef {
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

#[derive(Debug, Clone)]
pub enum LaunchType {
    Command(String),
    Terminal(String),
}

/// Runtime representation of a launchable entry.
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub launch: LaunchType,
    pub icon: Option<String>,
}

/// On-disk map of every category to its EntryDefs; startup selects one.
pub type CategoryMap = HashMap<String, Vec<EntryDef>>;

impl From<EntryDef> for Entry {
    fn from(def: EntryDef) -> Self {
        let launch = if def.terminal {
            LaunchType::Terminal(def.command)
        } else {
            LaunchType::Command(def.command)
        };
        Entry {
            id: def.id,
            name: def.name,
            description: def.description,
            launch,
            icon: def.icon,
        }
    }
}

/// Resolve the Entries for a given category, in declared order.
/// Unknown category yields an empty Vec.
pub fn entries_for_category(config: &CategoryMap, category: &str) -> Vec<Entry> {
    config
        .get(category)
        .map(|defs| defs.iter().cloned().map(Entry::from).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(terminal: bool) -> EntryDef {
        EntryDef {
            id: "id1".into(),
            name: "Name".into(),
            description: Some("desc".into()),
            icon: Some("icon".into()),
            command: "cmd".into(),
            terminal,
        }
    }

    #[test]
    fn from_def_command_when_not_terminal() {
        let e = Entry::from(def(false));
        assert!(matches!(e.launch, LaunchType::Command(ref c) if c == "cmd"));
    }

    #[test]
    fn from_def_terminal_when_terminal() {
        let e = Entry::from(def(true));
        assert!(matches!(e.launch, LaunchType::Terminal(ref c) if c == "cmd"));
    }

    #[test]
    fn from_def_carries_fields() {
        let e = Entry::from(def(false));
        assert_eq!(e.id, "id1");
        assert_eq!(e.name, "Name");
        assert_eq!(e.icon.as_deref(), Some("icon"));
        assert_eq!(e.description.as_deref(), Some("desc"));
    }

    #[test]
    fn entries_for_category_unknown_is_empty() {
        let map: CategoryMap = HashMap::new();
        assert!(entries_for_category(&map, "nope").is_empty());
    }

    #[test]
    fn entries_for_category_resolves_in_order() {
        let mut map: CategoryMap = HashMap::new();
        let mut a = def(false);
        a.id = "a".into();
        let mut b = def(false);
        b.id = "b".into();
        map.insert("cat".into(), vec![a, b]);
        let out = entries_for_category(&map, "cat");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "a");
        assert_eq!(out[1].id, "b");
    }
}
