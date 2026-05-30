use std::process::Command;

use crate::config::LaunchType;

/// A resolved, runnable command derived from an Entry's launch intent: the
/// final tilde-expanded shell string plus the binary name to check on PATH.
///
/// Pure and total — building one cannot fail; only [`run`]ning it can.
pub struct Invocation {
    /// The full shell command passed to `sh -c`.
    pub command: String,
    /// The first whitespace token — the binary to check on PATH.
    pub binary: String,
}

impl Invocation {
    /// Resolve a [`LaunchType`] into a runnable [`Invocation`].
    ///
    /// Owns every command-shaping decision: the `"kitty -e"` terminal
    /// fallback, the `"{term} {cmd}"` format, tilde expansion (with
    /// `home == None` leaving a leading `~` literal), and the first-token
    /// binary rule. Total: it cannot fail.
    ///
    /// For [`LaunchType::Terminal`] the checked binary is the *terminal*, not
    /// the app it hosts — the terminal is what must be on PATH.
    pub fn resolve(
        launch: &LaunchType,
        default_terminal: Option<&str>,
        home: Option<&str>,
    ) -> Invocation {
        let raw = match launch {
            LaunchType::Command(command) => command.clone(),
            LaunchType::Terminal(app) => {
                let term = default_terminal.unwrap_or("kitty -e");
                format!("{} {}", term, app)
            }
        };

        let command = expand_tilde(&raw, home);
        let binary = command
            .split_whitespace()
            .next()
            .unwrap_or(&command)
            .to_owned();

        Invocation { command, binary }
    }
}

/// Why running an [`Invocation`] failed.
pub enum LaunchError {
    /// The binary was not found on PATH.
    NotInstalled(String),
    /// `sh -c` could not be spawned.
    Spawn(std::io::Error),
}

/// Run an [`Invocation`]: check the binary is on PATH, then `sh -c` it.
///
/// The PATH pre-check is load-bearing — `sh -c` always spawns successfully
/// (`sh` exists), so it is the only way to detect a missing binary.
pub fn run(inv: &Invocation) -> Result<(), LaunchError> {
    if which(&inv.binary).is_none() {
        return Err(LaunchError::NotInstalled(inv.binary.clone()));
    }

    log::info!("Launching: {}", inv.command);

    Command::new("sh")
        .arg("-c")
        .arg(&inv.command)
        .spawn()
        .map(|_| ())
        .map_err(LaunchError::Spawn)
}

/// Expand a leading `~/` against `home`. With `home == None` the path is
/// returned unchanged, leaving the `~` literal.
fn expand_tilde(path: &str, home: Option<&str>) -> String {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = home
    {
        return format!("{}/{}", home, rest);
    }
    path.to_owned()
}

fn which(binary: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let full = dir.join(binary);
            full.is_file().then_some(full)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_passes_through_verbatim() {
        let inv = Invocation::resolve(&LaunchType::Command("foo".into()), None, None);
        assert_eq!(inv.command, "foo");
        assert_eq!(inv.binary, "foo");
    }

    #[test]
    fn terminal_uses_default_fallback() {
        let inv = Invocation::resolve(&LaunchType::Terminal("foo".into()), None, None);
        assert_eq!(inv.command, "kitty -e foo");
        assert_eq!(inv.binary, "kitty");
    }

    #[test]
    fn terminal_uses_custom_terminal() {
        let inv = Invocation::resolve(
            &LaunchType::Terminal("foo".into()),
            Some("alacritty -e"),
            None,
        );
        assert_eq!(inv.command, "alacritty -e foo");
        // The terminal is the checked binary, not the hosted app.
        assert_eq!(inv.binary, "alacritty");
    }

    #[test]
    fn tilde_expands_against_home() {
        let inv = Invocation::resolve(&LaunchType::Command("~/x".into()), None, Some("/h/me"));
        assert_eq!(inv.command, "/h/me/x");
        assert_eq!(inv.binary, "/h/me/x");
    }

    #[test]
    fn tilde_left_literal_without_home() {
        let inv = Invocation::resolve(&LaunchType::Command("~/x".into()), None, None);
        assert_eq!(inv.command, "~/x");
    }

    #[test]
    fn binary_is_first_whitespace_token() {
        let inv = Invocation::resolve(&LaunchType::Command("foo --bar baz".into()), None, None);
        assert_eq!(inv.binary, "foo");
    }
}
