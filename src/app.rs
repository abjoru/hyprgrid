use anyhow::Result;
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::rc::Rc;

use crate::config::{FlatEntry, load_theme};
use crate::input::{Action, parse_key};
use crate::launcher::launch;
use crate::ui::{build_grid, generate_css};

pub struct AppConfig {
    pub entries: Vec<FlatEntry>,
    pub terminal: Option<String>,
    pub icon_size: u32,
}

pub fn run(config: AppConfig) -> Result<()> {
    let app = Application::builder()
        .application_id("com.github.hyprgrid")
        .build();

    let entries = Rc::new(config.entries);
    let terminal = config.terminal;
    let icon_size = config.icon_size;

    app.connect_activate(move |app| {
        let theme = load_theme();
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
        let (container, state) = build_grid(entries.as_ref().clone(), num_accents, icon_size);
        window.set_child(Some(&container));

        // Keyboard handling
        let key_controller = gtk4::EventControllerKey::new();
        let state_clone = Rc::clone(&state);
        let window_clone = window.clone();
        let terminal_clone = terminal.clone();

        key_controller.connect_key_pressed(move |_, key, _, _| {
            match parse_key(key) {
                Action::MoveLeft => state_clone.move_selection(-1, 0),
                Action::MoveRight => state_clone.move_selection(1, 0),
                Action::MoveUp => state_clone.move_selection(0, -1),
                Action::MoveDown => state_clone.move_selection(0, 1),
                Action::Launch => {
                    if let Some(entry) = state_clone.current_entry() {
                        match launch(entry, terminal_clone.as_deref()) {
                            Ok(_) => window_clone.close(),
                            Err(e) => log::error!("Launch failed: {}", e),
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
