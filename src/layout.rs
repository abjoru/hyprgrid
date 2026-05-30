use std::collections::HashMap;

/// A single navigation step across the [`GridLayout`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    /// The (dx, dy) increment this direction applies to a coordinate.
    fn delta(self) -> (i32, i32) {
        match self {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
        }
    }
}

/// Generate diamond layer coordinates at distance n from origin.
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

/// Generate diamond coordinates for n items, returns (coord, layer).
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

/// Wrap a single-step coordinate component back into `[lo, hi]`.
fn wrap(v: i32, lo: i32, hi: i32) -> i32 {
    if v > hi {
        lo
    } else if v < lo {
        hi
    } else {
        v
    }
}

/// The pure geometry of a launcher screen: a diamond packing of N entries
/// onto integer coordinates plus the wrap-and-scan navigation between them.
///
/// Knows only the count of entries, never their contents — no GTK.
pub struct GridLayout {
    /// Entry index -> coordinate.
    idx_to_coord: Vec<(i32, i32)>,
    /// Entry index -> layer (distance from centre).
    layers: Vec<usize>,
    /// Coordinate -> entry index.
    coord_to_idx: HashMap<(i32, i32), usize>,
    /// Grid bounds (min_x, max_x, min_y, max_y).
    bounds: (i32, i32, i32, i32),
}

impl GridLayout {
    /// Build the diamond packing for `count` entries.
    pub fn new(count: usize) -> Self {
        let coords = diamond_coords(count);

        let min_x = coords.iter().map(|c| (c.0).0).min().unwrap_or(0);
        let max_x = coords.iter().map(|c| (c.0).0).max().unwrap_or(0);
        let min_y = coords.iter().map(|c| (c.0).1).min().unwrap_or(0);
        let max_y = coords.iter().map(|c| (c.0).1).max().unwrap_or(0);

        let mut idx_to_coord = Vec::with_capacity(count);
        let mut layers = Vec::with_capacity(count);
        let mut coord_to_idx = HashMap::with_capacity(count);

        for (i, &(coord, layer)) in coords.iter().enumerate() {
            idx_to_coord.push(coord);
            layers.push(layer);
            coord_to_idx.insert(coord, i);
        }

        Self {
            idx_to_coord,
            layers,
            coord_to_idx,
            bounds: (min_x, max_x, min_y, max_y),
        }
    }

    /// Number of entries packed into the layout.
    pub fn len(&self) -> usize {
        self.idx_to_coord.len()
    }

    /// Whether the layout holds no entries.
    pub fn is_empty(&self) -> bool {
        self.idx_to_coord.is_empty()
    }

    /// Coordinate assigned to `idx`, for GTK placement.
    pub fn coord(&self, idx: usize) -> (i32, i32) {
        self.idx_to_coord[idx]
    }

    /// Layer assigned to `idx`, for accent-colour assignment.
    pub fn layer(&self, idx: usize) -> usize {
        self.layers[idx]
    }

    /// Grid bounds (min_x, max_x, min_y, max_y).
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        self.bounds
    }

    /// Step the selection one `Direction` from `current`, wrapping at the grid
    /// edges and scanning past empty coordinates to the next occupied one.
    ///
    /// Returns `current` unchanged when no other occupied cell is reachable.
    pub fn step(&self, current: usize, dir: Direction) -> usize {
        if self.idx_to_coord.is_empty() {
            return current;
        }

        let (dx, dy) = dir.delta();
        let (cx, cy) = self.idx_to_coord[current];
        let (min_x, max_x, min_y, max_y) = self.bounds;
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let max_steps = width.max(height) as usize;

        let (mut nx, mut ny) = (cx, cy);
        // One initial step plus a scan of up to max_steps further hops.
        for _ in 0..=max_steps {
            nx = wrap(nx + dx, min_x, max_x);
            ny = wrap(ny + dy, min_y, max_y);

            if let Some(&idx) = self.coord_to_idx.get(&(nx, ny)) {
                return idx;
            }
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn packing_produces_n_unique_coords_with_non_decreasing_layers() {
        for count in [1usize, 2, 5, 9, 13, 25] {
            let layout = GridLayout::new(count);
            assert_eq!(layout.len(), count);

            let coords: HashSet<(i32, i32)> = (0..count).map(|i| layout.coord(i)).collect();
            assert_eq!(coords.len(), count, "all coordinates unique for n={count}");

            let mut prev = 0;
            for i in 0..count {
                let layer = layout.layer(i);
                assert!(
                    layer >= prev,
                    "layers non-decreasing at idx {i} for n={count}"
                );
                prev = layer;
            }
        }
    }

    #[test]
    fn single_entry_step_returns_itself() {
        let layout = GridLayout::new(1);
        for dir in [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ] {
            assert_eq!(layout.step(0, dir), 0);
        }
    }

    #[test]
    fn basic_moves_on_full_diamond() {
        // 5 entries fills layer 0 + layer 1: centre + 4 arms.
        // Coords: 0=(0,0), 1=(0,1), 2=(1,0), 3=(0,-1), 4=(-1,0).
        let layout = GridLayout::new(5);
        assert_eq!(layout.coord(0), (0, 0));

        // From centre, each direction lands on the matching arm.
        assert_eq!(layout.step(0, Direction::Right), 2); // (1,0)
        assert_eq!(layout.step(0, Direction::Left), 4); // (-1,0)
        assert_eq!(layout.step(0, Direction::Up), 3); // (0,-1)
        assert_eq!(layout.step(0, Direction::Down), 1); // (0,1)
    }

    #[test]
    fn wrap_from_edge_to_opposite_edge() {
        // 5-entry diamond, bounds x in [-1,1]. Right arm (1,0) wrapping right
        // crosses to x=-1, the left arm (-1,0).
        let layout = GridLayout::new(5);
        let right_arm = 2; // (1,0)
        assert_eq!(layout.coord(right_arm), (1, 0));
        assert_eq!(layout.step(right_arm, Direction::Right), 4); // wraps to (-1,0)

        let left_arm = 4; // (-1,0)
        assert_eq!(layout.step(left_arm, Direction::Left), 2); // wraps to (1,0)
    }

    #[test]
    fn step_scans_past_empty_coords_on_sparse_last_layer() {
        // 7 entries: full layer 0+1 (5 cells) plus two cells of the sparse
        // layer 2 at (0,2)=idx5 and (1,1)=idx6.
        let layout = GridLayout::new(7);
        assert_eq!(layout.coord(2), (1, 0));
        assert_eq!(layout.coord(6), (1, 1));

        // No entry occupies (1,-1): it is an empty hole in column x=1.
        let occupied: HashSet<(i32, i32)> = (0..7).map(|i| layout.coord(i)).collect();
        assert!(!occupied.contains(&(1, -1)), "(1,-1) is an empty hole");

        // Stepping Up from (1,0) targets the empty (1,-1) first, so it must
        // scan onward (wrapping through the column) to the occupied (1,1).
        assert_eq!(layout.step(2, Direction::Up), 6);
    }

    #[test]
    fn step_never_returns_out_of_range_index() {
        for count in [1usize, 3, 5, 6, 7, 12, 30] {
            let layout = GridLayout::new(count);
            for start in 0..count {
                for dir in [
                    Direction::Left,
                    Direction::Right,
                    Direction::Up,
                    Direction::Down,
                ] {
                    let next = layout.step(start, dir);
                    assert!(next < count, "step out of range for n={count}");
                }
            }
        }
    }
}
