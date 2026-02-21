# hyprgrid

[![Version](https://img.shields.io/badge/version-0.1.7-blue)](https://github.com/abjoru/hyprgrid)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![CI](https://github.com/abjoru/hyprgrid/actions/workflows/ci.yml/badge.svg)](https://github.com/abjoru/hyprgrid/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange)](https://www.rust-lang.org/)
![Gruvbox](https://img.shields.io/badge/theme-gruvbox-yellow)

A dynamic grid-based application launcher for Hyprland, inspired by XMonad's GridSelect.

## Features

- **YAML-based app config** - Define apps by category
- **Vim-style navigation** - hjkl keys + arrows
- **Gruvbox theming** - Customizable colors with cycling cell accents
- **Layer-shell overlay** - Centered popup on focused monitor
- **Terminal app support** - Launch TUI apps in your preferred terminal
- **Icon support** - Per-app icons via icon name or file path
- **Launch error notifications** - Desktop notifications for missing or failed apps

## Installation

### Dependencies (Arch)

```bash
sudo pacman -S gtk4 gtk4-layer-shell
```

### Build

```bash
cargo build -r
cp target/release/hyprgrid ~/.local/bin/
```

## Usage

```bash
hyprgrid -c <category>
hyprgrid -c favorites
hyprgrid -c system --terminal "alacritty -e"
hyprgrid -c internet --config ~/my/apps.yaml
```

### Keybindings

| Key | Action |
|-----|--------|
| h / ← | Move left |
| l / → | Move right |
| k / ↑ | Move up |
| j / ↓ | Move down |
| Enter | Launch app |
| Escape / q | Close |

## Configuration

### Apps Config

Default locations (in order):
1. `--config` flag
2. `~/.config/hyprgrid/apps.yaml`
3. `~/.config/xmonad/applications.yaml`

```yaml
favorites:
  - firefox:
    name: 'Firefox'
    description: 'Web browser'
    command: 'firefox'
    icon: 'firefox'              # icon name from theme
  - htop:
    name: 'HTop'
    description: 'Process monitor'
    terminal: 'htop'
    icon: '~/.icons/htop.png'    # or absolute/tilde path

system:
  - alacritty:
    name: 'Alacritty'
    command: 'alacritty'
```

### Theme Config

`~/.config/hyprgrid/config.yaml`:

```yaml
theme:
  bg: "#282828"
  bg_selected: "#3c3836"
  border: "#d79921"
  fg: "#ebdbb2"
  fg_dim: "#a89984"
  accents:
    - "#458588"  # blue
    - "#b16286"  # purple
    - "#689d6a"  # aqua
  icons_enabled: true   # enable/disable icons (default: true)
  icon_size: 32         # icon size in pixels (default: 32)
  dim_strength: 0.8     # desaturation of unselected cells, 0.0-1.0 (default: 0.8)
```

Default theme is Gruvbox Dark.

## Hyprland Integration

Add to `~/.config/hypr/hyprland.conf`:

```conf
bind = $mainMod, G, exec, hyprgrid -c favorites
bind = $mainMod SHIFT, G, exec, hyprgrid -c system
```

## Roadmap

- [x] YAML config parsing
- [x] Grid layout with dynamic sizing
- [x] Vim-style keyboard navigation
- [x] Layer-shell overlay
- [x] Gruvbox theming with accent colors
- [x] Terminal app support
- [ ] Window switcher mode (Hyprland IPC)
- [x] .desktop file parsing
- [x] Icon support
- [ ] Mouse support

## License

MIT
