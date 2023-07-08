use std::cmp::Ordering;

/// A coordinate on the grid.
#[derive(Debug, Clone, PartialEq)]
pub struct GridCoord {
    /// The X coordinate along the grid.
    pub x: f64,
    /// The y coordinate along the grid.
    pub y: f64,
}

impl GridCoord {
    /// Creates a new grid coordinate.
    #[inline(always)]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Converts this [`GridCoord`] into a tuple of X and Y coordinates, in that order.
    #[inline(always)]
    pub const fn into_xy(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl PartialOrd for GridCoord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.y.partial_cmp(&other.y) {
            None => self.x.partial_cmp(&other.x),
            Some(ordering) => Some(ordering),
        }
    }
}

impl From<(f64, f64)> for GridCoord {
    fn from(value: (f64, f64)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<GridCoord> for (f64, f64) {
    fn from(value: GridCoord) -> Self {
        value.into_xy()
    }
}
