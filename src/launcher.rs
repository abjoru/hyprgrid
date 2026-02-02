use anyhow::{Result, bail};
use notify_rust::Notification;
use std::process::Command;

use crate::config::{FlatEntry, LaunchType};

pub fn launch(entry: &FlatEntry, default_terminal: Option<&str>) -> Result<()> {
    let cmd = match &entry.entry.launch {
        LaunchType::Command(command) => command.clone(),
        LaunchType::Terminal(terminal) => {
            let term = default_terminal.unwrap_or("kitty -e");
            format!("{} {}", term, terminal)
        }
    };

    // Extract the binary name to check availability
    let binary = cmd.split_whitespace().next().unwrap_or(&cmd);

    if which(binary).is_none() {
        let msg = format!("'{}' is not installed or not in PATH", binary);
        notify_error(&entry.entry.name, &msg);
        bail!(msg);
    }

    log::info!("Launching: {}", cmd);

    match Command::new("sh").arg("-c").arg(&cmd).spawn() {
        Ok(_) => Ok(()),
        Err(e) => {
            let msg = format!("Failed to launch: {}", e);
            notify_error(&entry.entry.name, &msg);
            bail!(msg);
        }
    }
}

fn which(binary: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let full = dir.join(binary);
            full.is_file().then_some(full)
        })
    })
}

fn notify_error(app_name: &str, message: &str) {
    if let Err(e) = Notification::new()
        .summary(&format!("hyprgrid: {}", app_name))
        .body(message)
        .icon("dialog-error")
        .timeout(5000)
        .show()
    {
        log::error!("Failed to send notification: {}", e);
    }
}
