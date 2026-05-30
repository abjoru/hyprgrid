use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Frame, Grid};
use std::cell::Cell;
use std::rc::Rc;

use crate::config::FlatEntry;
use crate::layout::{Direction, GridLayout};

use super::cell::{create_cell, set_selected};
use super::style::accent_class;

/// GTK-side adapter over a [`GridLayout`]: holds the cells and the selected
/// index, asks the layout for the next index on each [`Direction`], and
/// repaints the affected cells. Contains no packing or navigation math.
pub struct Selection {
    entries: Vec<FlatEntry>,
    cells: Vec<GtkBox>,
    layout: GridLayout,
    /// Currently selected entry index.
    selected: Cell<usize>,
}

impl Selection {
    fn new(entries: Vec<FlatEntry>, cells: Vec<GtkBox>, layout: GridLayout) -> Self {
        Self {
            entries,
            cells,
            layout,
            selected: Cell::new(0),
        }
    }

    /// Repaint so `idx` becomes the selected cell.
    fn select(&self, idx: usize) {
        let old = self.selected.get();
        if old < self.cells.len() {
            set_selected(&self.cells[old], false);
        }
        if idx < self.cells.len() {
            set_selected(&self.cells[idx], true);
            self.selected.set(idx);
        }
    }

    /// Move the selection one [`Direction`] and repaint.
    pub fn step(&self, dir: Direction) {
        if self.layout.is_empty() {
            return;
        }
        let next = self.layout.step(self.selected.get(), dir);
        self.select(next);
    }

    /// The currently selected entry, if any.
    pub fn current_entry(&self) -> Option<&FlatEntry> {
        self.entries.get(self.selected.get())
    }
}

pub fn build_grid(
    entries: Vec<FlatEntry>,
    num_accents: usize,
    icon_size: u32,
) -> (Frame, Rc<Selection>) {
    let grid = Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);

    let layout = GridLayout::new(entries.len());
    let (min_x, _, min_y, _) = layout.bounds();

    let mut cells = Vec::with_capacity(layout.len());

    for (i, entry) in entries.iter().enumerate() {
        let (x, y) = layout.coord(i);
        let accent = accent_class(layout.layer(i), num_accents);
        let cell = create_cell(entry, &accent, icon_size);

        // Translate to positive grid coordinates.
        grid.attach(&cell, x - min_x, y - min_y, 1, 1);

        cells.push(cell);
    }

    let selection = Rc::new(Selection::new(entries, cells, layout));

    // Select center (first entry).
    if !selection.cells.is_empty() {
        selection.select(0);
    }

    let frame = Frame::new(None);
    frame.add_css_class("grid-container");
    frame.set_child(Some(&grid));

    (frame, selection)
}
