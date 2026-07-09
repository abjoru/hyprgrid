use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::config::{
    CommandOutput, ConfigFile, Entry, entries_for_category, entries_from_command, find_config,
    load_config, resolve_icons,
};
use crate::theme::Theme;

/// Bounded wall-clock budget for a CommandSource; exceeding it kills the child
/// and resolves to the inert error cell.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

/// The resolved startup state: the selected Category's Entries plus the Theme,
/// ready to hand to the GTK app.
pub struct Screen {
    pub entries: Vec<Entry>,
    pub theme: Theme,
}

impl Screen {
    /// I/O outer: resolve the config path and load it, then assemble. The
    /// process spawn for a CommandSource lives here, behind `run_command`.
    pub fn resolve(
        explicit_config: Option<&str>,
        category: &str,
        want_icons: bool,
    ) -> Result<Screen> {
        let path = find_config(explicit_config)?;
        log::info!("Using config: {}", path.display());
        let cfg = load_config(&path)?;
        Screen::from_config(cfg, category, want_icons, run_command)
    }

    /// Testable core: no path/file I/O. Owns theme-default, source precedence
    /// (static apps win over a CommandSource of the same name), the empty-check,
    /// and the icon-policy gate. The CommandSource spawn is injected as
    /// `run_command` so this stays pure under test; the only residual I/O is
    /// `resolve_icons`, gated off when icons are disabled.
    pub fn from_config<F>(
        cfg: ConfigFile,
        category: &str,
        want_icons: bool,
        run_command: F,
    ) -> Result<Screen>
    where
        F: FnOnce(&str) -> CommandOutput,
    {
        let theme = cfg.theme.unwrap_or_default();

        // Static apps win on name collision; only fall back to a CommandSource
        // when the category has no static entries.
        let mut entries = entries_for_category(&cfg.apps, category);
        if entries.is_empty()
            && let Some(source) = cfg.commands.get(category)
        {
            log::info!("Resolving category '{}' from CommandSource", category);
            entries = entries_from_command(category, run_command(&source.command));
        }
        if entries.is_empty() {
            anyhow::bail!("No apps found in category '{}'", category);
        }

        if want_icons && theme.icons_enabled {
            resolve_icons(&mut entries);
        }

        log::info!("Found {} entries in category '{}'", entries.len(), category);
        Ok(Screen { entries, theme })
    }
}

/// I/O seam: run a CommandSource's shell command with a bounded timeout,
/// capturing stdout. Maps spawn error, non-zero exit, and timeout to an
/// `Err(reason)`; JSON parsing of a successful stdout is the pure core's job.
fn run_command(command: &str) -> CommandOutput {
    let expanded = shellexpand::tilde(command).into_owned();

    let mut child = match Command::new("sh")
        .arg("-c")
        .arg(&expanded)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => return Err(format!("failed to spawn: {}", e)),
    };

    let deadline = Instant::now() + COMMAND_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = match child.wait_with_output() {
                    Ok(output) => output,
                    Err(e) => return Err(format!("failed to read output: {}", e)),
                };
                if status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                let detail = stderr.trim();
                return Err(if detail.is_empty() {
                    format!("exited with {}", status)
                } else {
                    format!("exited with {}: {}", status, detail)
                });
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("timed out after {}s", COMMAND_TIMEOUT.as_secs()));
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(e) => return Err(format!("wait failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LaunchType;
    use crate::config::types::{CategoryMap, CommandSource, EntryDef};

    /// A `run_command` stub that must never be invoked (static-only configs).
    fn no_command(_: &str) -> CommandOutput {
        panic!("run_command should not be called for a static category");
    }

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
        ConfigFile {
            apps,
            ..ConfigFile::default()
        }
    }

    #[test]
    fn unknown_category_errs() {
        let cfg = config_with("favorites", &["a"]);
        assert!(Screen::from_config(cfg, "missing", false, no_command).is_err());
    }

    #[test]
    fn empty_category_errs() {
        let cfg = config_with("favorites", &[]);
        assert!(Screen::from_config(cfg, "favorites", false, no_command).is_err());
    }

    #[test]
    fn known_category_resolves_entries_in_order() {
        let cfg = config_with("favorites", &["a", "b", "c"]);
        let screen = Screen::from_config(cfg, "favorites", false, no_command).unwrap();
        let ids: Vec<&str> = screen.entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }

    #[test]
    fn theme_absent_is_defaulted() {
        let cfg = config_with("favorites", &["a"]);
        let screen = Screen::from_config(cfg, "favorites", false, no_command).unwrap();
        assert_eq!(screen.theme.bg, Theme::default().bg);
    }

    #[test]
    fn theme_present_is_carried_through() {
        let mut cfg = config_with("favorites", &["a"]);
        cfg.theme = Some(Theme {
            bg: "#123456".into(),
            ..Theme::default()
        });
        let screen = Screen::from_config(cfg, "favorites", false, no_command).unwrap();
        assert_eq!(screen.theme.bg, "#123456");
    }

    #[test]
    fn want_icons_false_leaves_icons_untouched() {
        let mut cfg = config_with("favorites", &["a"]);
        cfg.apps.get_mut("favorites").unwrap()[0].icon = None;
        let screen = Screen::from_config(cfg, "favorites", false, no_command).unwrap();
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
        let screen = Screen::from_config(cfg, "favorites", true, no_command).unwrap();
        assert_eq!(screen.entries[0].icon, None);
    }

    fn with_command(category: &str, command: &str) -> ConfigFile {
        let mut commands = crate::config::types::CommandMap::new();
        commands.insert(
            category.into(),
            CommandSource {
                command: command.into(),
            },
        );
        ConfigFile {
            commands,
            ..ConfigFile::default()
        }
    }

    #[test]
    fn command_source_resolves_entries() {
        let cfg = with_command("games", "list-games");
        let json = r#"[{"id":"ff","name":"Firefox","command":"firefox"}]"#;
        let screen = Screen::from_config(cfg, "games", false, |cmd| {
            assert_eq!(cmd, "list-games");
            Ok(json.into())
        })
        .unwrap();
        assert_eq!(screen.entries.len(), 1);
        assert_eq!(screen.entries[0].id, "ff");
    }

    #[test]
    fn static_wins_over_command_of_same_name() {
        // Both a static [[apps.x]] and a [commands.x] exist; static must win
        // and the command must never run.
        let mut cfg = config_with("x", &["static-a"]);
        cfg.commands.insert(
            "x".into(),
            CommandSource {
                command: "should-not-run".into(),
            },
        );
        let screen = Screen::from_config(cfg, "x", false, no_command).unwrap();
        let ids: Vec<&str> = screen.entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, ["static-a"]);
    }

    #[test]
    fn command_failure_resolves_to_inert_cell() {
        let cfg = with_command("games", "nope");
        let screen =
            Screen::from_config(cfg, "games", false, |_| Err("command not found".into())).unwrap();
        assert_eq!(screen.entries.len(), 1);
        assert!(matches!(screen.entries[0].launch, LaunchType::Inert));
    }

    #[test]
    fn run_command_captures_stdout_on_success() {
        let out = run_command("printf '%s' hello").unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn run_command_errors_on_nonzero_exit() {
        assert!(run_command("exit 3").is_err());
    }

    #[test]
    fn run_command_errors_on_missing_binary() {
        // `sh -c` finds no such command -> non-zero exit -> Err.
        assert!(run_command("this-binary-does-not-exist-xyz").is_err());
    }

    // Slow (waits out the 5s budget); run explicitly with `--ignored`.
    #[test]
    #[ignore]
    fn run_command_times_out_on_hang() {
        let err = run_command("sleep 3600").unwrap_err();
        assert!(err.contains("timed out"));
    }
}
