use anyhow::Result;
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

    log::info!("Launching: {}", cmd);

    Command::new("sh").arg("-c").arg(&cmd).spawn()?;

    Ok(())
}
