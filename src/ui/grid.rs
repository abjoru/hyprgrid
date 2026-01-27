use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Frame, Grid};
use std::cell::Cell;
use std::rc::Rc;

use crate::config::FlatEntry;

use super::cell::{create_cell, set_selected};
use super::style::accent_class;

pub struct GridState {
    pub entries: Vec<FlatEntry>,
    pub cells: Vec<GtkBox>,
    pub cols: usize,
    pub selected: Cell<usize>,
}

impl GridState {
    pub fn new(entries: Vec<FlatEntry>, cells: Vec<GtkBox>, cols: usize) -> Self {
        Self {
            entries,
            cells,
            cols,
            selected: Cell::new(0),
        }
    }

    pub fn select(&self, idx: usize) {
        let old = self.selected.get();
        if old < self.cells.len() {
            set_selected(&self.cells[old], false);
        }
        if idx < self.cells.len() {
            set_selected(&self.cells[idx], true);
            self.selected.set(idx);
        }
    }

    pub fn move_selection(&self, dx: i32, dy: i32) {
        let count = self.cells.len();
        if count == 0 {
            return;
        }

        let current = self.selected.get();
        let cols = self.cols as i32;
        let row = current as i32 / cols;
        let col = current as i32 % cols;

        let new_col = (col + dx).rem_euclid(cols);
        let new_row = (row + dy).rem_euclid((count as i32 + cols - 1) / cols);
        let mut new_idx = (new_row * cols + new_col) as usize;

        // Handle last row having fewer items
        if new_idx >= count {
            new_idx = count - 1;
        }

        self.select(new_idx);
    }

    pub fn current_entry(&self) -> Option<&FlatEntry> {
        self.entries.get(self.selected.get())
    }
}

pub fn build_grid(entries: Vec<FlatEntry>, num_accents: usize) -> (Frame, Rc<GridState>) {
    let grid = Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);

    let count = entries.len();
    let cols = (count as f64).sqrt().ceil() as usize;

    let mut cells = Vec::with_capacity(count);

    for (i, entry) in entries.iter().enumerate() {
        let accent = accent_class(i, num_accents);
        let cell = create_cell(entry, &accent);
        let row = i / cols;
        let col = i % cols;
        grid.attach(&cell, col as i32, row as i32, 1, 1);
        cells.push(cell);
    }

    let state = Rc::new(GridState::new(entries, cells, cols));

    // Select first cell
    if !state.cells.is_empty() {
        state.select(0);
    }

    // Wrap grid in a styled frame
    let frame = Frame::new(None);
    frame.add_css_class("grid-container");
    frame.set_child(Some(&grid));

    (frame, state)
}
