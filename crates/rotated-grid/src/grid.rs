use crate::{Angle, GridPositionIterator};

/// The parameters of a rotated grid.
///
/// Set the fields directly or start from [`Grid::new`] and chain the builder methods.
/// Iterate over the grid positions with [`IntoIterator`] or [`GridPositionIterator::new`].
///
/// ```
/// use rotated_grid::{Angle, Grid};
///
/// // Builder style.
/// let built = Grid::new(600.0, 400.0)
///     .spacing(7.0, 7.0)
///     .angle(Angle::from_degrees(15.0));
///
/// // Struct literal; every value is named, so none can be swapped by accident.
/// let literal = Grid {
///     width: 600.0,
///     height: 400.0,
///     dx: 7.0,
///     dy: 7.0,
///     x0: 0.0,
///     y0: 0.0,
///     angle: Angle::from_degrees(15.0),
/// };
///
/// assert_eq!(built, literal);
/// assert_eq!(built.into_iter().count(), 4899);
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Grid {
    /// The width of the area to cover. Must be positive.
    pub width: f64,
    /// The height of the area to cover. Must be positive.
    pub height: f64,
    /// The spacing of grid elements along the (rotated) X axis. Must be positive.
    pub dx: f64,
    /// The spacing of grid elements along the (rotated) Y axis. Must be positive.
    pub dy: f64,
    /// The X offset of the grid, along the (rotated) X axis.
    pub x0: f64,
    /// The Y offset of the grid, along the (rotated) Y axis.
    pub y0: f64,
    /// The orientation of the grid. Any finite angle is allowed; positive angles turn
    /// the grid counter-clockwise in image coordinates (i.e., with the Y axis pointing down).
    pub angle: Angle,
}

impl Grid {
    /// Creates an unrotated grid covering `width` × `height` with a spacing of `1.0`
    /// and no offset.
    pub const fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            dx: 1.0,
            dy: 1.0,
            x0: 0.0,
            y0: 0.0,
            angle: Angle::from_radians(0.0),
        }
    }

    /// Sets the spacing of grid elements along the (rotated) X and Y axes.
    pub const fn spacing(self, dx: f64, dy: f64) -> Self {
        Self { dx, dy, ..self }
    }

    /// Sets the offset of the grid along the (rotated) X and Y axes.
    pub const fn offset(self, x0: f64, y0: f64) -> Self {
        Self { x0, y0, ..self }
    }

    /// Sets the orientation of the grid.
    pub const fn angle(self, angle: Angle) -> Self {
        Self { angle, ..self }
    }

    /// Packs the grid into the layout of the `RotatedGrid` struct in [`crate::WGSL`]:
    /// `[width, height, dx, dy, x0, y0, angle_in_radians, 0.0]`.
    ///
    /// Write the values into a uniform or storage buffer to use the grid from a shader.
    pub fn to_uniform(&self) -> [f32; 8] {
        [
            self.width as f32,
            self.height as f32,
            self.dx as f32,
            self.dy as f32,
            self.x0 as f32,
            self.y0 as f32,
            self.angle.into_radians() as f32,
            0.0,
        ]
    }
}

impl IntoIterator for Grid {
    type Item = crate::GridCoord;
    type IntoIter = GridPositionIterator;

    /// Creates the iterator over the grid positions.
    ///
    /// ## Panics
    /// See [`GridPositionIterator::new`].
    fn into_iter(self) -> Self::IntoIter {
        GridPositionIterator::new(self)
    }
}
