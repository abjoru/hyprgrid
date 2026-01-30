use std::path::Path;

use freedesktop_desktop_entry::{DesktopEntry, Iter as DesktopIter, default_paths};

use super::types::FlatEntry;

/// Resolve missing icons from .desktop files. Returns number of .desktop entries scanned.
pub fn resolve_icons(entries: &mut [FlatEntry]) -> usize {
    // Skip scanning if all entries already have icons
    if entries.iter().all(|e| e.entry.icon.is_some()) {
        return 0;
    }

    let desktop = scan_desktop_files();
    let count = desktop.len();

    for entry in entries.iter_mut() {
        if entry.entry.icon.is_some() {
            continue;
        }

        let id = entry.id.to_lowercase();

        let icon = desktop
            .iter()
            .find(|d| d.short_name == id)
            .or_else(|| desktop.iter().find(|d| d.exec_name.as_deref() == Some(&id)))
            .map(|d| d.icon.clone());

        entry.entry.icon = icon;
    }

    count
}

struct DesktopInfo {
    icon: String,
    short_name: String,
    exec_name: Option<String>,
}

fn scan_desktop_files() -> Vec<DesktopInfo> {
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

    entries
}
