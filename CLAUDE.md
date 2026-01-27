# CLAUDE.md

Guidance for Claude Code when working with this repository.

## Project Overview

hyprgrid - GTK4 grid-based application launcher for Hyprland with YAML config, vim navigation, gruvbox theming.

## Build Commands

```bash
cargo build          # Debug build
cargo build -r       # Release build
cargo run -- -c favorites  # Run with category
cargo clippy         # Lint
cargo fmt            # Format
```

## Dependencies (Arch)

```bash
sudo pacman -S gtk4 gtk4-layer-shell
```

## Architecture

```
src/
├── main.rs              # CLI (clap), bootstrap
├── app.rs               # GtkApplication, layer-shell, keyboard events
├── launcher.rs          # Process spawning (command/terminal)
├── config/
│   ├── mod.rs
│   ├── types.rs         # AppEntry, LaunchType, FlatEntry
│   └── loader.rs        # YAML loading, XDG path resolution
├── theme/
│   ├── mod.rs
│   └── types.rs         # Theme struct, Gruvbox defaults
├── ui/
│   ├── mod.rs
│   ├── grid.rs          # Grid widget, selection state
│   ├── cell.rs          # Cell rendering with accent colors
│   └── style.rs         # CSS generation from theme
└── input/
    ├── mod.rs
    └── keys.rs          # hjkl/arrows/Enter/Escape handling
```

## Key Files

- `config/types.rs` - YAML parsing, handles flat format: `{ id:, name:, command/terminal: }`
- `ui/grid.rs` - GridState manages selection, movement wrapping
- `ui/style.rs` - Generates GTK CSS from Theme, accent color classes
- `app.rs` - Layer-shell setup, keyboard event routing

## Config Locations

Apps (first found):
1. `--config` CLI flag
2. `~/.config/hyprgrid/apps.yaml`
3. `~/.config/xmonad/applications.yaml`

Theme: `~/.config/hyprgrid/config.yaml`

## Future Work

- Search/filter input field
- Window switcher via Hyprland IPC (`hyprctl clients -j`)
- .desktop file parsing (freedesktop-desktop-entry crate)
- Icon rendering in cells
- Mouse click support
