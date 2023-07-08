//! Contains the [`Line`] type.

use crate::inner::vector::Vector;
use std::ops::{Mul, Neg};

/// A line determined by a ray starting at a point of origin.
#[derive(Debug, Clone)]
pub struct Line {
    /// The origin point of the line.
    origin: Vector,
    /// The direction vector of the line.
    direction: Vector,
}

impl Line {
    /// Constructs a line from an origin point and a direction.
    pub fn new(origin: Vector, direction: Vector) -> Self {
        Self {
            origin,
            direction: direction.normalized(),
        }
    }

    /// Constructs a line through two points.
    #[inline(always)]
    pub fn from_points(a: Vector, b: &Vector) -> Self {
        Self::new(a, *b - a)
    }

    #[inline(always)]
    pub const fn origin(&self) -> &Vector {
        &self.origin
    }

    #[inline(always)]
    pub const fn direction(&self) -> &Vector {
        &self.direction
    }

    /// Projects a vector at a given distance alongside a direction
    /// from the current origin.
    #[inline(always)]
    pub fn project_out(&self, t: f64) -> Vector {
        Vector {
            x: self.origin.x + self.direction.x * t,
            y: self.origin.y + self.direction.y * t,
        }
    }

    pub fn calculate_intersection_t(&self, other: &Self, max_u: f64) -> Option<f64> {
        let det = self.direction.cross(other.direction());
        if det.abs() < 1e-6 {
            // Lines are either parallel or coincident
            return None;
        }

        let delta = self.origin - other.origin;

        // Length along self to the point of intersection.
        let t = other.direction.cross(&delta) / det;

        // Project the intersection point out.
        let projected = delta.project_out(&self.direction, t);

        // Squared length along other to the point of intersection.
        let u = projected.dot(&other.direction);

        if t >= 0.0 && u >= 0.0 && u <= max_u * max_u {
            Some(t)
        } else {
            None
        }
    }
}

impl Neg for Line {
    type Output = Line;

    fn neg(self) -> Self::Output {
        Self {
            origin: self.origin,
            direction: -self.direction,
        }
    }
}

impl Mul<f64> for Line {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        self.origin + rhs * self.direction
    }
}
