mod app;
mod config;
mod input;
mod launcher;
mod layout;
mod screen;
mod theme;
mod ui;

use anyhow::Result;
use clap::Parser;

use screen::Screen;

#[derive(Parser)]
#[command(name = "hyprgrid")]
#[command(about = "Dynamic grid-based launcher for Hyprland")]
struct Cli {
    /// Category to display
    #[arg(short, long)]
    category: String,

    /// Path to config file
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

    let screen = Screen::resolve(cli.config.as_deref(), &cli.category, !cli.no_icons)?;

    app::run(app::AppConfig {
        entries: screen.entries,
        terminal: cli.terminal,
        theme: screen.theme,
    })
}
