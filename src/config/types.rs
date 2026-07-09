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

/// TOML wire shape of a CommandSource: a Category whose Entries are produced
/// by running `command` at startup and parsing its JSON stdout as EntryDefs.
#[derive(Debug, Clone, Deserialize)]
pub struct CommandSource {
    pub command: String,
}

#[derive(Debug, Clone)]
pub enum LaunchType {
    Command(String),
    Terminal(String),
    /// A non-launchable Entry: selecting it in the grid is a no-op. Used for
    /// the error cell a failing CommandSource resolves to.
    Inert,
}

/// Runtime representation of an entry. Usually launchable; an [`Entry::inert`]
/// carries [`LaunchType::Inert`] and only surfaces a failure.
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub launch: LaunchType,
    pub icon: Option<String>,
}

/// On-disk map of every static category to its EntryDefs; startup selects one.
pub type CategoryMap = HashMap<String, Vec<EntryDef>>;

/// On-disk map of every dynamic category to its CommandSource.
pub type CommandMap = HashMap<String, CommandSource>;

/// The outcome of running a CommandSource, produced by the I/O seam:
/// `Ok(stdout)` — the command exited 0; `Err(reason)` — spawn error, non-zero
/// exit, or timeout, with a human-readable reason.
pub type CommandOutput = Result<String, String>;

impl Entry {
    /// A non-launchable error cell whose visible text conveys `message`.
    pub fn inert(message: String) -> Entry {
        Entry {
            id: "__inert__".into(),
            name: message,
            description: None,
            launch: LaunchType::Inert,
            icon: None,
        }
    }
}

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

/// Pure core of CommandSource resolution: map a command's run outcome to
/// Entries, or a single [`Entry::inert`] on any failure — bad outcome (spawn
/// error, non-zero exit, timeout), invalid JSON, or a valid-but-empty array.
/// The failure detail is logged to stderr and mirrored into the inert cell's
/// text. Never returns an empty Vec, so the launcher window always opens.
pub fn entries_from_command(category: &str, output: CommandOutput) -> Vec<Entry> {
    let stdout = match output {
        Ok(stdout) => stdout,
        Err(reason) => {
            log::error!("CommandSource '{}' failed: {}", category, reason);
            return vec![Entry::inert(format!("{}: {}", category, reason))];
        }
    };

    match serde_json::from_str::<Vec<EntryDef>>(&stdout) {
        Ok(defs) if defs.is_empty() => {
            log::error!("CommandSource '{}' produced no entries", category);
            vec![Entry::inert(format!("{}: no entries", category))]
        }
        Ok(defs) => defs.into_iter().map(Entry::from).collect(),
        Err(e) => {
            log::error!("CommandSource '{}' emitted invalid JSON: {}", category, e);
            vec![Entry::inert(format!("{}: invalid JSON: {}", category, e))]
        }
    }
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

    #[test]
    fn command_valid_json_maps_entries_in_order() {
        let json = r#"[
            {"id":"ff","name":"Firefox","command":"firefox"},
            {"id":"top","name":"Shell","command":"htop","terminal":true}
        ]"#;
        let out = entries_from_command("games", Ok(json.into()));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "ff");
        assert!(matches!(out[0].launch, LaunchType::Command(ref c) if c == "firefox"));
        assert_eq!(out[1].id, "top");
        assert!(matches!(out[1].launch, LaunchType::Terminal(ref c) if c == "htop"));
    }

    #[test]
    fn command_honors_all_optional_fields() {
        let json = r#"[{"id":"i","name":"N","command":"c","icon":"ic","description":"d"}]"#;
        let out = entries_from_command("games", Ok(json.into()));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].icon.as_deref(), Some("ic"));
        assert_eq!(out[0].description.as_deref(), Some("d"));
    }

    #[test]
    fn command_failure_outcome_is_single_inert() {
        let out = entries_from_command("games", Err("command not found".into()));
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].launch, LaunchType::Inert));
        assert!(out[0].name.contains("command not found"));
    }

    #[test]
    fn command_invalid_json_is_single_inert() {
        let out = entries_from_command("games", Ok("not json".into()));
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].launch, LaunchType::Inert));
    }

    #[test]
    fn command_missing_required_id_is_single_inert() {
        let out = entries_from_command("games", Ok(r#"[{"name":"N","command":"c"}]"#.into()));
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].launch, LaunchType::Inert));
    }

    #[test]
    fn command_empty_array_is_single_inert() {
        let out = entries_from_command("games", Ok("[]".into()));
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0].launch, LaunchType::Inert));
    }
}
