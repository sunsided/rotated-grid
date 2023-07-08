//! # Rotated grids for CMYK halftone dithering and more.
//!
//! This crate provides the [`GridPositionIterator`] type that creates
//! spaced grid positions along a rotated grid.
//!
//! ## Order of generated coordinates
//!
//! Do note that the generation order of the coordinates depends on the specific grid parameters
//! and may not be in the most efficient layout when used directly, depending on your use case.
//! For image processing you may want to prefer a top-down order, in which case you should collect
//! the coordinates into a vector and sort by `y` coordinate first.
//!
//! ## Example
//!
//! ```
//! use rotated_grid::{Angle, GridCoord, GridPositionIterator};
//!
//! const WIDTH: usize = 16;
//! const HEIGHT: usize = 10;
//!
//! let halftone_grids = [
//!     ("Cyan", 15.0),
//!     ("Magenta", 75.0),
//!     ("Yellow", 0.0),
//!     ("Black", 45.0),
//! ];
//!
//! for (name, angle) in halftone_grids {
//!     println!("{name} at {angle}°");
//!
//!     let grid = GridPositionIterator::new(
//!         WIDTH as _,
//!         HEIGHT as _,
//!         7.0,
//!         7.0,
//!         0.0,
//!         0.0,
//!         Angle::<f64>::from_degrees(angle),
//!     );
//!
//!     let (_, expected_max) = grid.size_hint();
//!     let mut count = 0;
//!
//!     for GridCoord { x, y } in grid {
//!         println!("{x}, {y}");
//!         count += 1;
//!     }
//!
//!     assert!(count <= expected_max.unwrap())
//! }
//! ```

mod angle;
mod grid_coord;
mod line;
mod line_segment;
mod optimal_iterator;
mod vector;

use crate::angle::AngleOps;
use crate::optimal_iterator::OptimalIterator;
pub use angle::Angle;
pub use grid_coord::GridCoord;
pub use line::Line;
pub use line_segment::LineSegment;
pub use vector::Vector;

/// An iterator for positions on a rotated grid.
pub struct GridPositionIterator {
    width: f64,
    height: f64,
    dx: f64,
    dy: f64,
    inv_sin: f64,
    inv_cos: f64,
    inner: OptimalIterator,
}

impl GridPositionIterator {
    /// Creates a new iterator.
    ///
    /// ## Arguments
    /// * `width` - The width of the grid. Must be positive.
    /// * `height` - The height of the grid. Must be positive.
    /// * `dx` - The spacing of grid elements along the (rotated) X axis.
    /// * `dy` - The spacing of grid elements along the (rotated) Y axis.
    /// * `x0` - The X offset of the first grid element.
    /// * `x1` - The Y offset of the first grid element.
    /// * `alpha` - The orientation of the grid.
    pub fn new(
        width: f64,
        height: f64,
        dx: f64,
        dy: f64,
        x0: f64,
        y0: f64,
        alpha: Angle<f64>,
    ) -> Self {
        assert!(width > 0.0);
        assert!(height > 0.0);

        let tl = Vector::new(0.0, 0.0);
        let tr = Vector::new(width, 0.0);
        let bl = Vector::new(0.0, height);
        let br = Vector::new(width, height);

        let alpha = alpha.normalize();
        let (sin, cos) = alpha.sin_cos();

        Self {
            width,
            height,
            dx,
            dy,
            inv_sin: -sin,
            inv_cos: cos,
            inner: OptimalIterator::new(tl, tr, bl, br, alpha, dx, dy, x0, y0),
        }
    }

    /// Provides an estimated upper bound for the number of grid points.
    /// This is only correct for unrotated grids; rotated grids produce smaller values.
    fn estimate_max_grid_points(&self) -> usize {
        let num_points_x = (self.width + self.dx) / self.dx;
        let num_points_y = (self.height + self.dy) / self.dy;
        (num_points_x * num_points_y).ceil() as _
    }
}

impl Iterator for GridPositionIterator {
    type Item = GridCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(point) = self.inner.next() {
            let x = point.x;
            let y = point.y;
            let center = self.inner.center();

            // Un-rotate the point.
            let unrotated_x =
                (x - center.x) * self.inv_cos - (y - center.y) * self.inv_sin + center.x;
            let unrotated_y =
                (x - center.x) * self.inv_sin + (y - center.y) * self.inv_cos + center.y;

            Some(GridCoord::new(unrotated_x, unrotated_y))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.estimate_max_grid_points()))
    }
}
