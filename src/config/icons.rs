use std::path::Path;

use freedesktop_desktop_entry::{DesktopEntry, Iter as DesktopIter, default_paths};

use super::types::Entry;

/// Icons declared by the system's `.desktop` files, normalized for matching:
/// `short_name`/`exec_name` are lowercased by `scan`, so `icon_for` only
/// lowercases the query.
pub struct DesktopIcons(Vec<DesktopInfo>);

struct DesktopInfo {
    icon: String,
    short_name: String,
    exec_name: Option<String>,
}

impl DesktopIcons {
    /// IO adapter: walk `default_paths()` and parse every `.desktop` file.
    pub fn scan() -> Self {
        let mut entries = Vec::new();

        for path in DesktopIter::new(default_paths()) {
            let Ok(bytes) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(de) = DesktopEntry::from_str(&path, &bytes, None::<&[&str]>) else {
                continue;
            };

            let Some(icon_raw) = de.icon() else { continue };
            let icon = icon_raw.to_string();

            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_lowercase();
            if stem.is_empty() {
                continue;
            }

            let short_name = stem.rsplit('.').next().unwrap_or(&stem).to_string();

            let exec_name = de.exec().map(|e| {
                let first = e.split_whitespace().next().unwrap_or(e);
                Path::new(first)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(first)
                    .to_lowercase()
            });

            entries.push(DesktopInfo {
                icon,
                short_name,
                exec_name,
            });
        }

        DesktopIcons(entries)
    }

    /// Pure matcher: lowercase `id`, match `short_name`, else fall back to
    /// `exec_name`. `short_name` wins. Total.
    pub fn icon_for(&self, id: &str) -> Option<String> {
        let id = id.to_lowercase();
        self.0
            .iter()
            .find(|d| d.short_name == id)
            .or_else(|| self.0.iter().find(|d| d.exec_name.as_deref() == Some(&id)))
            .map(|d| d.icon.clone())
    }
}

/// Resolve missing icons from `.desktop` files, in place. Skips the
/// filesystem scan when every entry already has an icon.
pub fn resolve_icons(entries: &mut [Entry]) {
    if entries.iter().all(|e| e.icon.is_some()) {
        return;
    }

    let icons = DesktopIcons::scan();
    log::info!("Resolved icons from {} .desktop entries", icons.0.len());

    for entry in entries.iter_mut().filter(|e| e.icon.is_none()) {
        entry.icon = icons.icon_for(&entry.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(icon: &str, short_name: &str, exec_name: Option<&str>) -> DesktopInfo {
        DesktopInfo {
            icon: icon.into(),
            short_name: short_name.into(),
            exec_name: exec_name.map(Into::into),
        }
    }

    #[test]
    fn icon_for_matches_short_name() {
        let icons = DesktopIcons(vec![info("firefox-icon", "firefox", None)]);
        assert_eq!(icons.icon_for("firefox").as_deref(), Some("firefox-icon"));
    }

    #[test]
    fn icon_for_falls_back_to_exec_name() {
        let icons = DesktopIcons(vec![info("editor-icon", "code", Some("nvim"))]);
        assert_eq!(icons.icon_for("nvim").as_deref(), Some("editor-icon"));
    }

    #[test]
    fn icon_for_short_name_wins_over_exec_name() {
        let icons = DesktopIcons(vec![
            info("exec-match", "other", Some("target")),
            info("short-match", "target", Some("ignored")),
        ]);
        assert_eq!(icons.icon_for("target").as_deref(), Some("short-match"));
    }

    #[test]
    fn icon_for_is_case_insensitive() {
        let icons = DesktopIcons(vec![info("ff", "firefox", None)]);
        assert_eq!(icons.icon_for("Firefox").as_deref(), Some("ff"));
    }

    #[test]
    fn icon_for_no_match_is_none() {
        let icons = DesktopIcons(vec![info("ff", "firefox", Some("firefox-bin"))]);
        assert_eq!(icons.icon_for("chromium"), None);
    }

    #[test]
    fn icon_for_empty_is_none() {
        let icons = DesktopIcons(Vec::new());
        assert_eq!(icons.icon_for("anything"), None);
    }
}
