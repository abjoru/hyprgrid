use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Frame, Grid};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::config::FlatEntry;

use super::cell::{create_cell, set_selected};
use super::style::accent_class;

/// Generate diamond layer coordinates at distance n from origin
/// Layer 0: [(0,0)]
/// Layer 1: [(0,1), (1,0), (0,-1), (-1,0)]
/// Layer 2: [(0,2), (1,1), (2,0), (1,-1), (0,-2), (-1,-1), (-2,0), (-1,1)]
fn diamond_layer(n: i32) -> Vec<(i32, i32)> {
    if n == 0 {
        return vec![(0, 0)];
    }

    // Top-right quadrant: from (0, n) to (n-1, 1)
    let tr: Vec<(i32, i32)> = (0..n).map(|x| (x, n - x)).collect();

    // Rotate 90° clockwise for bottom-right: (x, y) -> (y, -x)
    let br: Vec<(i32, i32)> = tr.iter().map(|&(x, y)| (y, -x)).collect();

    // Combine right half
    let mut right: Vec<(i32, i32)> = tr.into_iter().chain(br).collect();

    // Mirror for left half: (x, y) -> (-x, -y)
    let left: Vec<(i32, i32)> = right.iter().map(|&(x, y)| (-x, -y)).collect();

    right.extend(left);
    right
}

/// Generate diamond coordinates for n items, returns (coord, layer)
fn diamond_coords(n: usize) -> Vec<((i32, i32), usize)> {
    let mut coords = Vec::with_capacity(n);
    let mut layer = 0;

    while coords.len() < n {
        for coord in diamond_layer(layer) {
            coords.push((coord, layer as usize));
        }
        layer += 1;
    }

    coords.truncate(n);
    coords
}

pub struct GridState {
    pub entries: Vec<FlatEntry>,
    pub cells: Vec<GtkBox>,
    /// Maps (col, row) to entry index
    pub coord_to_idx: HashMap<(i32, i32), usize>,
    /// Maps entry index to (col, row)
    pub idx_to_coord: Vec<(i32, i32)>,
    /// Currently selected entry index
    pub selected: Cell<usize>,
    /// Grid bounds (min_x, max_x, min_y, max_y)
    pub bounds: (i32, i32, i32, i32),
}

impl GridState {
    pub fn new(
        entries: Vec<FlatEntry>,
        cells: Vec<GtkBox>,
        coord_to_idx: HashMap<(i32, i32), usize>,
        idx_to_coord: Vec<(i32, i32)>,
        bounds: (i32, i32, i32, i32),
    ) -> Self {
        Self {
            entries,
            cells,
            coord_to_idx,
            idx_to_coord,
            selected: Cell::new(0),
            bounds,
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
        if self.cells.is_empty() {
            return;
        }

        let current = self.selected.get();
        let (cx, cy) = self.idx_to_coord[current];

        let (min_x, max_x, min_y, max_y) = self.bounds;
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;

        // Try wrapping in the movement direction to find next valid cell
        let mut nx = cx + dx;
        let mut ny = cy + dy;

        // Wrap coordinates
        if nx > max_x {
            nx = min_x;
        } else if nx < min_x {
            nx = max_x;
        }
        if ny > max_y {
            ny = min_y;
        } else if ny < min_y {
            ny = max_y;
        }

        // If target cell exists, move there
        if let Some(&idx) = self.coord_to_idx.get(&(nx, ny)) {
            self.select(idx);
            return;
        }

        // Otherwise scan in movement direction until we find a valid cell
        for _ in 0..(width.max(height) as usize) {
            nx += dx;
            ny += dy;

            // Wrap
            if nx > max_x {
                nx = min_x;
            } else if nx < min_x {
                nx = max_x;
            }
            if ny > max_y {
                ny = min_y;
            } else if ny < min_y {
                ny = max_y;
            }

            if let Some(&idx) = self.coord_to_idx.get(&(nx, ny)) {
                self.select(idx);
                return;
            }
        }
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
    let coords = diamond_coords(count);

    // Calculate bounds
    let min_x = coords.iter().map(|c| (c.0).0).min().unwrap_or(0);
    let max_x = coords.iter().map(|c| (c.0).0).max().unwrap_or(0);
    let min_y = coords.iter().map(|c| (c.0).1).min().unwrap_or(0);
    let max_y = coords.iter().map(|c| (c.0).1).max().unwrap_or(0);

    let mut cells = Vec::with_capacity(count);
    let mut coord_to_idx = HashMap::new();
    let mut idx_to_coord = Vec::with_capacity(count);

    for (i, entry) in entries.iter().enumerate() {
        let ((x, y), layer) = coords[i];
        let accent = accent_class(layer, num_accents);
        let cell = create_cell(entry, &accent);

        // Translate to positive grid coordinates
        let col = x - min_x;
        let row = y - min_y;
        grid.attach(&cell, col, row, 1, 1);

        cells.push(cell);
        coord_to_idx.insert((x, y), i);
        idx_to_coord.push((x, y));
    }

    let state = Rc::new(GridState::new(
        entries,
        cells,
        coord_to_idx,
        idx_to_coord,
        (min_x, max_x, min_y, max_y),
    ));

    // Select center (first entry)
    if !state.cells.is_empty() {
        state.select(0);
    }

    let frame = Frame::new(None);
    frame.add_css_class("grid-container");
    frame.set_child(Some(&grid));

    (frame, state)
}
