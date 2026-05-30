use anyhow::Result;
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use notify_rust::Notification;
use std::rc::Rc;

use crate::config::FlatEntry;
use crate::input::{Action, parse_key};
use crate::launcher::{self, Invocation, LaunchError};
use crate::layout::Direction;
use crate::theme::Theme;
use crate::ui::{build_grid, generate_css};

pub struct AppConfig {
    pub entries: Vec<FlatEntry>,
    pub terminal: Option<String>,
    pub theme: Theme,
}

pub fn run(config: AppConfig) -> Result<()> {
    let app = Application::builder()
        .application_id("com.github.hyprgrid")
        .build();

    let entries = Rc::new(config.entries);
    let terminal = config.terminal;
    let theme = config.theme;

    app.connect_activate(move |app| {
        let num_accents = theme.accents.len().max(1);
        let css = generate_css(&theme);

        // Load CSS
        let provider = CssProvider::new();
        provider.load_from_data(&css);
        gtk4::style_context_add_provider_for_display(
            &Display::default().expect("Could not get display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Create window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("hyprgrid")
            .build();

        // Layer shell setup
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        // Build grid
        let icon_size = theme.icon_size;
        let (container, state) = build_grid(entries.as_ref().clone(), num_accents, icon_size);
        window.set_child(Some(&container));

        // Keyboard handling
        let key_controller = gtk4::EventControllerKey::new();
        let state_clone = Rc::clone(&state);
        let window_clone = window.clone();
        let terminal_clone = terminal.clone();

        key_controller.connect_key_pressed(move |_, key, _, _| {
            match parse_key(key) {
                Action::MoveLeft => state_clone.step(Direction::Left),
                Action::MoveRight => state_clone.step(Direction::Right),
                Action::MoveUp => state_clone.step(Direction::Up),
                Action::MoveDown => state_clone.step(Direction::Down),
                Action::Launch => {
                    if let Some(entry) = state_clone.current_entry() {
                        let home = std::env::var("HOME").ok();
                        let inv = Invocation::resolve(
                            &entry.entry.launch,
                            terminal_clone.as_deref(),
                            home.as_deref(),
                        );
                        match launcher::run(&inv) {
                            Ok(_) => window_clone.close(),
                            Err(e) => report_launch_error(&entry.entry.name, &e),
                        }
                    }
                }
                Action::Close => window_clone.close(),
                Action::None => {}
            }
            gtk4::glib::Propagation::Stop
        });

        window.add_controller(key_controller);
        window.present();
    });

    app.run_with_args::<&str>(&[]);
    Ok(())
}

/// Map a [`LaunchError`] to a desktop notification plus a log line. Owns all
/// user-facing failure reporting so the launcher seam stays pure.
fn report_launch_error(app_name: &str, err: &LaunchError) {
    let msg = match err {
        LaunchError::NotInstalled(binary) => {
            format!("'{}' is not installed or not in PATH", binary)
        }
        LaunchError::Spawn(e) => format!("Failed to launch: {}", e),
    };

    log::error!("Launch failed for '{}': {}", app_name, msg);

    if let Err(e) = Notification::new()
        .summary(&format!("hyprgrid: {}", app_name))
        .body(&msg)
        .icon("dialog-error")
        .timeout(5000)
        .show()
    {
        log::error!("Failed to send notification: {}", e);
    }
}
