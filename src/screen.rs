use anyhow::Result;

use crate::config::{
    ConfigFile, Entry, entries_for_category, find_config, load_config, resolve_icons,
};
use crate::theme::Theme;

/// The resolved startup state: the selected Category's Entries plus the Theme,
/// ready to hand to the GTK app.
pub struct Screen {
    pub entries: Vec<Entry>,
    pub theme: Theme,
}

impl Screen {
    /// I/O outer: resolve the config path and load it, then assemble.
    pub fn resolve(
        explicit_config: Option<&str>,
        category: &str,
        want_icons: bool,
    ) -> Result<Screen> {
        let path = find_config(explicit_config)?;
        log::info!("Using config: {}", path.display());
        let cfg = load_config(&path)?;
        Screen::from_config(cfg, category, want_icons)
    }

    /// Testable core: no path/file I/O. Owns theme-default, category select,
    /// empty-check, and the icon-policy gate. The only residual I/O is
    /// `resolve_icons`, gated off when icons are disabled.
    pub fn from_config(cfg: ConfigFile, category: &str, want_icons: bool) -> Result<Screen> {
        let theme = cfg.theme.unwrap_or_default();

        let mut entries = entries_for_category(&cfg.apps, category);
        if entries.is_empty() {
            anyhow::bail!("No apps found in category '{}'", category);
        }

        if want_icons && theme.icons_enabled {
            resolve_icons(&mut entries);
        }

        log::info!("Found {} apps in category '{}'", entries.len(), category);
        Ok(Screen { entries, theme })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{CategoryMap, EntryDef};

    fn entry_def(id: &str) -> EntryDef {
        EntryDef {
            id: id.into(),
            name: id.into(),
            description: None,
            icon: Some("preset".into()),
            command: "cmd".into(),
            terminal: false,
        }
    }

    fn config_with(category: &str, ids: &[&str]) -> ConfigFile {
        let mut apps: CategoryMap = CategoryMap::new();
        apps.insert(
            category.into(),
            ids.iter().map(|id| entry_def(id)).collect(),
        );
        ConfigFile { theme: None, apps }
    }

    #[test]
    fn unknown_category_errs() {
        let cfg = config_with("favorites", &["a"]);
        assert!(Screen::from_config(cfg, "missing", false).is_err());
    }

    #[test]
    fn empty_category_errs() {
        let cfg = config_with("favorites", &[]);
        assert!(Screen::from_config(cfg, "favorites", false).is_err());
    }

    #[test]
    fn known_category_resolves_entries_in_order() {
        let cfg = config_with("favorites", &["a", "b", "c"]);
        let screen = Screen::from_config(cfg, "favorites", false).unwrap();
        let ids: Vec<&str> = screen.entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }

    #[test]
    fn theme_absent_is_defaulted() {
        let cfg = config_with("favorites", &["a"]);
        let screen = Screen::from_config(cfg, "favorites", false).unwrap();
        assert_eq!(screen.theme.bg, Theme::default().bg);
    }

    #[test]
    fn theme_present_is_carried_through() {
        let mut cfg = config_with("favorites", &["a"]);
        cfg.theme = Some(Theme {
            bg: "#123456".into(),
            ..Theme::default()
        });
        let screen = Screen::from_config(cfg, "favorites", false).unwrap();
        assert_eq!(screen.theme.bg, "#123456");
    }

    #[test]
    fn want_icons_false_leaves_icons_untouched() {
        let mut cfg = config_with("favorites", &["a"]);
        cfg.apps.get_mut("favorites").unwrap()[0].icon = None;
        let screen = Screen::from_config(cfg, "favorites", false).unwrap();
        assert_eq!(screen.entries[0].icon, None);
    }

    #[test]
    fn icons_disabled_in_theme_skips_scan() {
        let mut cfg = config_with("favorites", &["a"]);
        cfg.apps.get_mut("favorites").unwrap()[0].icon = None;
        cfg.theme = Some(Theme {
            icons_enabled: false,
            ..Theme::default()
        });
        let screen = Screen::from_config(cfg, "favorites", true).unwrap();
        assert_eq!(screen.entries[0].icon, None);
    }
}
