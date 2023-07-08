//! Contains the [`LineSegment`] type.

use crate::inner::line::Line;
use crate::inner::vector::Vector;

/// A line segment determined by a ray starting at a point of origin with a specified length and direction.
pub struct LineSegment {
    /// The origin point of the line segment.
    pub(crate) start: Vector,
    /// The length and direction vector of the line segment.
    pub(crate) end: Vector,
}

impl LineSegment {
    /// Constructs a line from an origin point and a direction.
    pub fn new(origin: Vector, length: Vector) -> Self {
        Self {
            start: origin,
            end: length,
        }
    }

    /// Constructs a line through two points.
    pub fn from_points(a: Vector, b: &Vector) -> Self {
        Self::new(a, *b - a)
    }

    /// Gets a normalized length version of the line.
    pub fn normalized(&self) -> Line {
        Line::new(self.start, self.end)
    }

    pub fn direction(&self) -> Vector {
        (self.end - self.start).normalized()
    }

    pub const fn start(&self) -> &Vector {
        &self.start
    }

    pub const fn end(&self) -> &Vector {
        &self.end
    }
}
