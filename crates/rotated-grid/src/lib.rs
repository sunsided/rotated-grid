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
mod vector;

pub use angle::Angle;
pub use grid_coord::GridCoord;
pub use line::Line;
pub use line_segment::LineSegment;
pub use vector::Vector;

/// An iterator for positions on a rotated grid.
pub struct GridPositionIterator {
    width: f64,
    height: f64,
    rotated_width: f64,
    rotated_height: f64,
    dx: f64,
    dy: f64,
    x0: f64,
    y0: f64,
    center_x: f64,
    center_y: f64,
    start_x: f64,
    start_y: f64,
    sin_alpha: f64,
    cos_alpha: f64,
    current_x: f64,
    current_y: f64,
    top: Line,
    left: Line,
    bottom: Line,
    right: Line,
    #[cfg(debug_assertions)]
    hits: u64,
    #[cfg(debug_assertions)]
    misses: u64,
    #[cfg(debug_assertions)]
    max_repeats: u64,
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

        let alpha_rad = Self::normalize_angle(alpha.into_radians());
        let (sin_alpha, cos_alpha) = alpha_rad.sin_cos();

        // Calculate the dimensions of the rotated grid
        let rotated_width = (width.abs() * cos_alpha) + (height.abs() * sin_alpha);
        let rotated_height = (width.abs() * sin_alpha) + (height.abs() * cos_alpha);

        // Calculate the center of the rotated grid.
        let center_x = x0 + (width * 0.5);
        let center_y = y0 + (height * 0.5);

        // Calculate the starting point of the rotated grid.
        let start_x = center_x - (rotated_width * 0.5);
        let start_y = center_y - (rotated_height * 0.5);

        // Rotate clockwise in order to swap between mathematical and image coordinate systems.
        // let center = Vector::new(center_x, center_y);
        let offset = Vector::new(x0, y0);

        // let alpha = Angle::from_radians(alpha_rad);
        let tl = (Vector::new(0.0, 0.0) + offset); //.rotate_around(&center, alpha);
        let tr = (Vector::new(width, 0.0) + offset); //.rotate_around(&center, alpha);
        let bl = (Vector::new(0.0, height) + offset); //.rotate_around(&center, alpha);
        let br = (Vector::new(width, height) + offset); //.rotate_around(&center, alpha);

        let top = Line::from_points(tr, &tl);
        let left = Line::from_points(tl, &bl);
        let bottom = Line::from_points(bl, &br);
        let right = Line::from_points(br, &tr);

        let iterator = GridPositionIterator {
            width,
            height,
            rotated_width,
            rotated_height,
            dx,
            dy,
            x0,
            y0,
            center_x,
            center_y,
            start_x,
            start_y,
            sin_alpha,
            cos_alpha,
            top,
            left,
            bottom,
            right,
            current_x: 0.0,
            current_y: 0.0,
            #[cfg(debug_assertions)]
            hits: 0,
            #[cfg(debug_assertions)]
            misses: 0,
            #[cfg(debug_assertions)]
            max_repeats: 0,
        };
        iterator
    }

    /// Provides an estimated upper bound for the number of grid points.
    /// This is only correct for unrotated grids; rotated grids produce smaller values.
    fn estimate_max_grid_points(&self) -> usize {
        let num_points_x = (self.width / self.dx).ceil() as usize;
        let num_points_y = (self.height / self.dy).ceil() as usize;
        num_points_x * num_points_y
    }

    /// Normalizes the specified angle such that it falls into range -PI/2..PI/2.
    fn normalize_angle(mut alpha: f64) -> f64 {
        use std::f64::consts::PI;
        const HALF_PI: f64 = PI * 0.5;
        while alpha >= PI {
            alpha -= PI;
        }
        while alpha >= HALF_PI {
            alpha -= HALF_PI;
        }
        while alpha <= -PI {
            alpha += PI;
        }
        while alpha <= -HALF_PI {
            alpha += HALF_PI;
        }
        alpha
    }
}

fn calculate_intersection_t(
    x0_scanline: f64,
    y0_scanline: f64,
    dx_scanline: f64,
    dy_scanline: f64,
    x0_edge: f64,
    y0_edge: f64,
    dx_edge: f64,
    dy_edge: f64,
) -> Option<f64> {
    let det = dx_scanline * dy_edge - dx_edge * dy_scanline;

    if det.abs() < 1e-6 {
        // Lines are either parallel or coincident
        None
    } else {
        let t = (dx_edge * (y0_scanline - y0_edge) - dy_edge * (x0_scanline - x0_edge)) / det;
        Some(t)
    }
}

impl Iterator for GridPositionIterator {
    type Item = GridCoord;

    fn next(&mut self) -> Option<Self::Item> {
        let (sin, cos) = (self.sin_alpha, self.cos_alpha);

        let mut repeats = 0;
        loop {
            let x = self.start_x + self.current_x;
            let y = self.start_y + self.current_y;

            // Rotate the grid position back to the unrotated frame.
            let inv_sin = -sin;
            let inv_cos = cos;
            let unrotated_x =
                (x - self.center_x) * inv_cos - (y - self.center_y) * inv_sin + self.center_x;
            let unrotated_y =
                (x - self.center_x) * inv_sin + (y - self.center_y) * inv_cos + self.center_y;

            let center = Vector::new(self.center_x, self.center_y);
            let ray = Line::new(
                Vector::new(self.start_x, y).rotate_around_with(&center, inv_sin, cos),
                Vector::new(1.0, 0.0).rotate_with(inv_sin, cos),
            );

            let left = ray
                .calculate_intersection_t(&self.left, self.height)
                .unwrap_or(f64::NAN);
            let top = ray
                .calculate_intersection_t(&self.top, self.width)
                .unwrap_or(f64::NAN);
            let right = ray
                .calculate_intersection_t(&self.right, self.height)
                .unwrap_or(f64::NAN);
            let bottom = ray
                .calculate_intersection_t(&self.bottom, self.width)
                .unwrap_or(f64::NAN);

            let thresholds = [left, top, right, bottom];
            let mut min = f64::NAN;
            let mut max = f64::NAN;
            for value in thresholds {
                if min.is_nan() || value < min {
                    min = value;
                }
                if max.is_nan() || value > max {
                    max = value;
                }
            }

            // Update the current position.
            self.current_x += self.dx;
            if self.current_x > self.rotated_width {
                self.current_x = 0.0;
                self.current_y += self.dy;
            }

            // TODO: Initialize current_x such that unrotated_x never is <= x0
            // TODO: Initialize current_x such that unrotated_x never is <= x0

            // Check if the grid position is within the original rectangle.
            if unrotated_x >= self.x0
                && unrotated_x <= self.x0 + self.width
                && unrotated_y >= self.y0
                && unrotated_y <= self.y0 + self.height
            {
                debug_assert!(x >= min);
                debug_assert!(x <= max);

                #[cfg(debug_assertions)]
                {
                    self.hits += 1;
                }
                return Some(GridCoord::new(unrotated_x, unrotated_y));
            }

            if x > self.start_x + self.rotated_width || y > self.start_y + self.rotated_height {
                #[cfg(debug_assertions)]
                {
                    debug_assert!(self.hits as usize <= self.estimate_max_grid_points());
                }
                return None;
            }

            #[cfg(debug_assertions)]
            {
                self.misses += 1;
                repeats += 1;
                self.max_repeats = repeats.max(self.max_repeats);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.estimate_max_grid_points()))
    }
}

fn find_t(x0: f64, dx: f64, x1: f64) -> Option<f64> {
    if dx == 0.0 {
        if x1 <= x0 {
            return Some(x0);
        }

        // Since x0 is zero there are no increments that lift
        // the coordinate away from x0, hence there is no solution.
        return None;
    }

    let t = ((x1 - x0) / dx).ceil();
    Some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_t_pos() {
        let x0: f64 = 2.0;
        let dx: f64 = 1.5;
        let x1: f64 = 7.0;

        assert_eq!(find_t(x0, dx, x1), Some(4.0));
    }

    #[test]
    fn test_find_t_neg() {
        let x0: f64 = 2.0;
        let dx: f64 = 1.5;
        let x1: f64 = -7.0;

        assert_eq!(find_t(x0, dx, x1), Some(-6.0));
    }

    #[test]
    fn test_find_t_invalid() {
        let x0: f64 = 2.0;
        let dx: f64 = 0.0;
        let x1: f64 = -7.0;

        assert_eq!(find_t(x0, dx, x1), Some(2.0));
    }

    #[test]
    fn test_find_t_impossible() {
        let x0: f64 = 2.0;
        let dx: f64 = 0.0;
        let x1: f64 = 7.0;

        assert_eq!(find_t(x0, dx, x1), None);
    }
}
