//! # Rotated grids for CMYK halftone dithering and more.
//!
//! This crate enumerates the positions of a rotated grid that fall inside an axis-aligned
//! rectangle. Describe the grid with [`Grid`] and iterate over it to get [`GridCoord`]s.
//!
//! ## Order of generated coordinates
//!
//! Do note that the generation order of the coordinates depends on the specific grid parameters
//! and may not be in the most efficient layout when used directly, depending on your use case.
//! For image processing you may want to prefer a top-down order, in which case you should collect
//! the coordinates into a vector and sort them; [`GridCoord`] orders by `y`, then `x`.
//!
//! ## Example
//!
//! ```
//! use rotated_grid::{Angle, Grid, GridCoord};
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
//!     let grid = Grid::new(16.0, 10.0)
//!         .spacing(7.0, 7.0)
//!         .angle(Angle::from_degrees(angle))
//!         .into_iter();
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
mod grid;
mod grid_coord;
mod inner;

use crate::inner::optimal_iterator::OptimalIterator;
use crate::inner::vector::Vector;
pub use angle::Angle;
pub use grid::Grid;
pub use grid_coord::GridCoord;

/// WGSL functions for looking up the grid from shaders.
///
/// The source defines a `RotatedGrid` struct, laid out like [`Grid::to_uniform`], and
/// functions such as `rotated_grid_nearest` and `rotated_grid_neighbors` that find the grid
/// points around a position. They use the same lattice as [`GridPositionIterator`], so a
/// shader can draw on exactly the points the iterator yields without receiving them one by one.
///
/// Prepend it to your shader source, or register it with a shader import system.
pub const WGSL: &str = include_str!("rotated_grid.wgsl");

/// Compiles and runs the code examples in the README as doctests.
#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
struct ReadmeDoctests;

/// An iterator for positions on a rotated grid.
pub struct GridPositionIterator {
    dx: f64,
    dy: f64,
    sin: f64,
    cos: f64,
    inner: OptimalIterator,
}

impl GridPositionIterator {
    /// Creates an iterator over the positions of the specified grid.
    ///
    /// ## Panics
    /// Panics if `width`, `height`, `dx` or `dy` is not positive, or if any value
    /// (including the angle) is not finite.
    pub fn new(grid: Grid) -> Self {
        let Grid {
            width,
            height,
            dx,
            dy,
            x0,
            y0,
            angle,
        } = grid;

        assert!(angle.into_radians().is_finite());
        assert!(width > 0.0 && width.is_finite());
        assert!(height > 0.0 && height.is_finite());
        assert!(dx > 0.0 && dx.is_finite());
        assert!(dy > 0.0 && dy.is_finite());
        assert!(x0.is_finite() && y0.is_finite());

        let tl = Vector::new(0.0, 0.0);
        let tr = Vector::new(width, 0.0);
        let bl = Vector::new(0.0, height);
        let br = Vector::new(width, height);

        let (sin, cos) = angle.sin_cos();

        Self {
            dx,
            dy,
            sin,
            cos,
            inner: OptimalIterator::new(tl, tr, bl, br, angle, dx, dy, x0, y0),
        }
    }
}

impl Iterator for GridPositionIterator {
    type Item = GridCoord;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let point = self.inner.next()?;

        // Un-rotate the point.
        let point = point.rotate_around_with(self.inner.center(), -self.sin, self.cos);
        Some(GridCoord::new(point.x, point.y))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // At most one point per grid step (plus one) along each axis of the bounding box.
        let extent = self.inner.extent();
        let max_x = (extent.x / self.dx).floor() + 1.0;
        let max_y = (extent.y / self.dy).floor() + 1.0;
        (0, Some((max_x * max_y) as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counts the grid points inside the rectangle by testing every lattice point
    /// of the bounding box, using the same lattice anchoring as the iterator.
    fn brute_force_count(w: f64, h: f64, dx: f64, dy: f64, x0: f64, y0: f64, deg: f64) -> usize {
        let (sin, cos) = deg.to_radians().sin_cos();
        let (cx, cy) = (w * 0.5, h * 0.5);
        let ex = w * cos.abs() + h * sin.abs();
        let ey = w * sin.abs() + h * cos.abs();
        let start_x = cx - ((ex / dx) * 0.5).floor() * dx + x0;
        let start_y = cy - ((ey / dy) * 0.5).floor() * dy + y0;
        let (nx, ny) = ((ex / dx).ceil() as i64 + 2, (ey / dy).ceil() as i64 + 2);
        let eps = 1e-9;

        let mut count = 0;
        for j in -ny..=ny {
            for i in -nx..=nx {
                let (x, y) = (start_x + i as f64 * dx - cx, start_y + j as f64 * dy - cy);
                let ux = x * cos + y * sin + cx;
                let uy = -x * sin + y * cos + cy;
                if ux >= -eps && ux <= w + eps && uy >= -eps && uy <= h + eps {
                    count += 1;
                }
            }
        }
        count
    }

    #[test]
    fn matches_brute_force() {
        let cases = [
            (600.0, 400.0, 7.0, 7.0, 0.0, 0.0),
            (600.0, 400.0, 10.0, 20.0, 3.0, 5.0),
            (272.11, 6.73, 5.22, 2.21, 0.75, 1.37),
            (0.5, 0.5, 0.1, 0.1, 0.0, 0.0),
            (4.24, 77.35, 0.81, 9.89, 0.33, 1.35),
        ];
        let angles = [
            0.0, 15.0, 45.0, 88.79, 90.0, 135.0, 180.0, 270.0, -30.0, -90.0, 400.0,
        ];

        for (w, h, dx, dy, x0, y0) in cases {
            for deg in angles {
                let grid = Grid {
                    width: w,
                    height: h,
                    dx,
                    dy,
                    x0,
                    y0,
                    angle: Angle::from_degrees(deg),
                }
                .into_iter();
                let max = grid.size_hint().1.unwrap();
                let count = grid.count();
                let expected = brute_force_count(w, h, dx, dy, x0, y0, deg);
                let case = format!("w={w} h={h} dx={dx} dy={dy} x0={x0} y0={y0} angle={deg}");
                assert_eq!(count, expected, "{case}");
                assert!(count <= max, "{case}: {count} > size_hint {max}");
            }
        }
    }

    #[test]
    fn rotation_direction() {
        let (dx, deg) = (5.0, 30.0_f64);
        let mut grid = Grid::new(100.0, 100.0)
            .spacing(dx, dx)
            .angle(Angle::from_degrees(deg))
            .into_iter();
        let a = grid.next().unwrap();
        let b = grid.next().unwrap();
        let (sin, cos) = deg.to_radians().sin_cos();

        // Positive angles turn the grid counter-clockwise when Y points down.
        assert!((b.x - a.x - dx * cos).abs() < 1e-9);
        assert!((b.y - a.y + dx * sin).abs() < 1e-9);
    }

    #[test]
    fn wgsl_is_valid() {
        let module = naga::front::wgsl::parse_str(WGSL).expect("WGSL parses");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .expect("WGSL validates");
    }

    #[test]
    fn uniform_layout_matches_wgsl_struct() {
        let grid = Grid::new(600.0, 400.0)
            .spacing(7.0, 8.0)
            .offset(1.0, 2.0)
            .angle(Angle::from_radians(0.5));
        assert_eq!(
            grid.to_uniform(),
            [600.0, 400.0, 7.0, 8.0, 1.0, 2.0, 0.5, 0.0]
        );
    }

    #[test]
    #[should_panic]
    fn zero_dx_panics() {
        Grid::new(10.0, 10.0).spacing(0.0, 1.0).into_iter();
    }

    #[test]
    #[should_panic]
    fn negative_dy_panics() {
        Grid::new(10.0, 10.0).spacing(1.0, -1.0).into_iter();
    }
}
