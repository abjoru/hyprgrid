mod app;
mod config;
mod input;
mod launcher;
mod theme;
mod ui;

use anyhow::Result;
use clap::Parser;

use config::{FlatEntry, find_apps_config, load_apps, load_theme, resolve_icons};

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

    /// Disable icon rendering
    #[arg(long)]
    no_icons: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let theme = load_theme();

    let config_path = find_apps_config(cli.config.as_deref())?;
    log::info!("Using config: {}", config_path.display());

    let apps = load_apps(&config_path)?;

    let mut entries = FlatEntry::flatten(&apps, &cli.category);
    if entries.is_empty() {
        anyhow::bail!("No apps found in category '{}'", cli.category);
    }

    // Resolve icons from .desktop files
    let icons_enabled = !cli.no_icons && theme.icons_enabled;
    if icons_enabled {
        let scanned = resolve_icons(&mut entries);
        log::info!("Resolved icons from {} .desktop entries", scanned);
    }

    log::info!(
        "Found {} apps in category '{}'",
        entries.len(),
        cli.category
    );

    app::run(app::AppConfig {
        entries,
        terminal: cli.terminal,
        icon_size: theme.icon_size,
    })
}
