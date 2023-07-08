use crate::vector::Vector;
use crate::LineSegment;
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
    pub fn from_points(a: Vector, b: &Vector) -> Self {
        Self::new(a, *b - a)
    }

    pub fn dot(&self, point: &Vector) -> f64 {
        self.direction.dot(&(*point - self.origin))
    }

    pub const fn origin(&self) -> &Vector {
        &self.origin
    }

    pub const fn direction(&self) -> &Vector {
        &self.direction
    }

    /// Determines the intersection of this line with another one.
    ///
    /// ## Arguments
    /// * `other` - The other line to test.
    ///
    /// ## Returns
    /// * `Some(Vector)` of the intersection point.
    /// * `None` if the lines are parallel or coincide.
    pub fn intersect_with_segment(&self, line_segment: &LineSegment) -> Option<Vector> {
        let p = self.origin;
        let q = *line_segment.start();
        let r = self.direction;
        let s = *line_segment.end();

        let q_minus_p = q - p;
        let r_cross_s = r.cross(&s);

        if r_cross_s == 0.0 {
            // The line and line segment are parallel or coincident
            return None;
        }

        let t = q_minus_p.cross(&s) / r_cross_s;
        let u = q_minus_p.cross(&r) / r_cross_s;

        let length_sq = line_segment.end().norm_sq();
        let t_sq = t * t;

        if t >= 0.0 && t_sq <= length_sq && u >= 0.0 && u <= 1.0 {
            // Calculate the intersection point
            let intersection_x = p.x + t * r.x;
            let intersection_y = p.y + t * r.y;

            Some(Vector {
                x: intersection_x,
                y: intersection_y,
            })
        } else {
            // The line and line segment do not intersect within the line segment boundaries
            None
        }
    }

    pub fn calculate_intersection_t(&self, other: &Self, max_u: f64) -> Option<f64> {
        let det = self.direction.x * other.direction.y - other.direction.x * self.direction.y;
        if det.abs() < 1e-6 {
            // Lines are either parallel or coincident
            return None;
        }
        // Length along self to the point of intersection.
        let t = (other.direction.x * (self.origin.y - other.origin.y)
            - other.direction.y * (self.origin.x - other.origin.x))
            / det;

        // Length along other to the point of intersection.
        let u = ((self.origin.x + t * self.direction.x - other.origin.x) * other.direction.x
            + (self.origin.y + t * self.direction.y - other.origin.y) * other.direction.y)
            / max_u;

        if t >= 0.0 && u >= 0.0 && u <= max_u {
            Some(t)
        } else {
            None
        }
    }

    /// Determines the distance of the line to a point.
    /// If the returned distance is positive, the point lies to the left of the line.
    pub fn distance(&self, point: &Vector) -> f64 {
        let v1 = self.direction;
        let v2 = Vector {
            x: point.x - self.origin.x,
            y: point.y - self.origin.y,
        };

        let cross_product = v1.x * v2.y - v1.y * v2.x;
        cross_product
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
