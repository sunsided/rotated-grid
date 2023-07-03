use crate::vector::Vector;
use crate::Line;

/// A line segment determined by a ray starting at a point of origin with a specified length and direction.
pub struct LineSegment {
    /// The origin point of the line segment.
    pub origin: Vector,
    /// The length and direction vector of the line segment.
    pub length: Vector,
}

impl LineSegment {
    /// Constructs a line from an origin point and a direction.
    pub fn new(origin: Vector, length: Vector) -> Self {
        Self { origin, length }
    }

    /// Constructs a line through two points.
    pub fn from_points(a: Vector, b: &Vector) -> Self {
        Self::new(a, (*b - a))
    }

    /// Gets a normalized length version of the line.
    pub fn normalized(&self) -> Line {
        Line::new(self.origin, self.length)
    }
}
