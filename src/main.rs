mod app;
mod config;
mod input;
mod launcher;
mod theme;
mod ui;

use anyhow::Result;
use clap::Parser;

use config::{FlatEntry, find_apps_config, load_apps};

#[derive(Parser)]
#[command(name = "hyprgrid")]
#[command(about = "Dynamic grid-based launcher for Hyprland")]
struct Cli {
    /// Category to display
    #[arg(short, long)]
    category: String,

    /// Path to apps config file
    #[arg(long)]
    config: Option<String>,

    /// Terminal command for terminal apps
    #[arg(short, long)]
    terminal: Option<String>,
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    let config_path = find_apps_config(cli.config.as_deref())?;
    log::info!("Using config: {}", config_path.display());

    let apps = load_apps(&config_path)?;

    let entries = FlatEntry::flatten(&apps, &cli.category);
    if entries.is_empty() {
        anyhow::bail!("No apps found in category '{}'", cli.category);
    }

    log::info!(
        "Found {} apps in category '{}'",
        entries.len(),
        cli.category
    );

    app::run(app::AppConfig {
        entries,
        terminal: cli.terminal,
    })
}
