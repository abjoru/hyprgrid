use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, Orientation};

use crate::config::FlatEntry;

pub fn create_cell(entry: &FlatEntry, accent_class: &str, icon_size: u32) -> GtkBox {
    let container = GtkBox::new(Orientation::Horizontal, 8);
    container.add_css_class("grid-cell");
    container.add_css_class(accent_class);

    // Icon (optional)
    if let Some(icon_str) = &entry.entry.icon {
        let expanded = shellexpand::tilde(icon_str);
        let image = if expanded.starts_with('/') {
            Image::from_file(expanded.as_ref())
        } else {
            Image::from_icon_name(icon_str)
        };
        image.set_pixel_size(icon_size as i32);
        image.set_valign(gtk4::Align::Start);
        image.add_css_class("cell-icon");
        container.append(&image);
    }

    // Text column
    let text_box = GtkBox::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let name_label = Label::new(Some(&entry.entry.name));
    name_label.add_css_class("cell-name");
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    text_box.append(&name_label);

    if let Some(desc) = &entry.entry.description {
        let desc_label = Label::new(Some(desc));
        desc_label.add_css_class("cell-desc");
        desc_label.set_halign(gtk4::Align::Start);
        desc_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        desc_label.set_max_width_chars(25);
        text_box.append(&desc_label);
    }

    container.append(&text_box);
    container
}

pub fn set_selected(cell: &GtkBox, selected: bool) {
    if selected {
        cell.add_css_class("selected");
    } else {
        cell.remove_css_class("selected");
    }
}
