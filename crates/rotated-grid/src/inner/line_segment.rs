//! Contains the [`LineSegment`] type.

use crate::inner::line::Line;
use crate::inner::vector::Vector;

/// A line segment determined by a ray starting at a point of origin with a specified length and direction.
pub struct LineSegment {
    /// The origin point of the line segment.
    pub(crate) start: Vector,
    /// The length and direction vector of the line segment.
    pub(crate) direction: Vector,
}

impl LineSegment {
    /// Constructs a line from an origin point and a direction.
    pub fn new(origin: Vector, length: Vector) -> Self {
        Self {
            start: origin,
            direction: length,
        }
    }

    /// Constructs a line through two points.
    #[inline(always)]
    pub fn from_points(a: Vector, b: &Vector) -> Self {
        Self::new(a, *b - a)
    }

    /// Gets a normalized length version of the line.
    #[inline(always)]
    pub fn normalized(&self) -> Line {
        Line::new(self.start, self.direction)
    }

    pub fn direction_normalized(&self) -> Vector {
        (self.direction - self.start).normalized()
    }

    #[inline(always)]
    pub const fn start(&self) -> &Vector {
        &self.start
    }

    #[inline(always)]
    pub const fn direction(&self) -> &Vector {
        &self.direction
    }
}
