use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};

use crate::config::FlatEntry;

pub fn create_cell(entry: &FlatEntry, accent_class: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 4);
    container.add_css_class("grid-cell");
    container.add_css_class(accent_class);

    let name_label = Label::new(Some(&entry.entry.name));
    name_label.add_css_class("cell-name");
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    container.append(&name_label);

    if let Some(desc) = &entry.entry.description {
        let desc_label = Label::new(Some(desc));
        desc_label.add_css_class("cell-desc");
        desc_label.set_halign(gtk4::Align::Start);
        desc_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        desc_label.set_max_width_chars(25);
        container.append(&desc_label);
    }

    container
}

pub fn set_selected(cell: &GtkBox, selected: bool) {
    if selected {
        cell.add_css_class("selected");
    } else {
        cell.remove_css_class("selected");
    }
}
