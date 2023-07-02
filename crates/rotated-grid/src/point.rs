use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub struct GridPoint {
    /// The X coordinate along the grid.
    pub x: f64,
    /// The y coordinate along the grid.
    pub y: f64,
}

impl GridPoint {
    /// Creates a new grid coordinate.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Converts this [`GridPoint`] into a tuple of X and Y coordinates, in that order.
    pub const fn into_xy(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl PartialOrd for GridPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.y.partial_cmp(&other.y) {
            None => self.x.partial_cmp(&other.x),
            Some(ordering) => Some(ordering),
        }
    }
}

impl From<(f64, f64)> for GridPoint {
    fn from(value: (f64, f64)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<GridPoint> for (f64, f64) {
    fn from(value: GridPoint) -> Self {
        value.into_xy()
    }
}
