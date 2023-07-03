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
        Self::new(a, (*b - a))
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
    pub fn intersect_with_line(&self, other: &Self) -> Option<Vector> {
        let dx = self.origin.x - other.origin.x;
        let dy = self.origin.y - other.origin.y;

        let determinant =
            other.direction.x * self.direction.y - other.direction.y * self.direction.x;
        if determinant == 0.0 {
            // The lines are parallel
            return None;
        }

        let t = (other.direction.x * dy - other.direction.y * dx) / determinant;

        let intersection_x = self.origin.x + t * self.direction.x;
        let intersection_y = self.origin.y + t * self.direction.y;
        Some(
            (Vector {
                x: intersection_x,
                y: intersection_y,
            }),
        )
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
        let q = *line_segment.origin();
        let r = self.direction;
        let s = *line_segment.length();

        let q_minus_p = q - p;
        let r_cross_s = r.cross(&s);

        if r_cross_s == 0.0 {
            // The line and line segment are parallel or coincident
            return None;
        }

        let t = q_minus_p.cross(&s) / r_cross_s;
        let u = q_minus_p.cross(&r) / r_cross_s;

        let length_sq = line_segment.length().norm_sq();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersect_with_1() {
        let line1 = Line {
            origin: Vector { x: 2.0, y: 3.0 },
            direction: Vector { x: 2.0, y: 2.0 },
        };
        let line2 = Line {
            origin: Vector { x: 1.0, y: 5.0 },
            direction: Vector { x: 2.0, y: -1.0 },
        };

        let intersection = line1.intersect_with_line(&line2);
        assert_eq!(intersection, Some(Vector { x: 3.0, y: 4.0 }));
    }

    #[test]
    fn test_intersect_with_2() {
        let line1 = Line {
            origin: Vector { x: 1.0, y: 1.0 },
            direction: Vector { x: 2.0, y: 2.0 },
        };
        let line2 = Line {
            origin: Vector { x: 2.0, y: 1.0 },
            direction: Vector { x: -1.0, y: 2.0 },
        };

        let intersection = line1.intersect_with_line(&line2).map(|v| v.round(3));
        assert_eq!(intersection, Some(Vector { x: 1.667, y: 1.667 }));
    }

    #[test]
    fn test_intersect_with_3() {
        let line1 = Line {
            origin: Vector { x: 0.0, y: 0.0 },
            direction: Vector { x: 1.0, y: 0.0 },
        };
        let line2 = Line {
            origin: Vector { x: 0.0, y: 1.0 },
            direction: Vector { x: 1.0, y: 0.0 },
        };

        let intersection = line1.intersect_with_line(&line2);
        assert_eq!(intersection, None);
    }

    #[test]
    fn test_intersect_with_4() {
        let line1 = Line {
            origin: Vector { x: 0.0, y: 0.0 },
            direction: Vector { x: 1.0, y: 0.0 },
        };
        let line2 = Line {
            origin: Vector { x: 1.0, y: 1.0 },
            direction: Vector { x: 0.0, y: -1.0 },
        };

        let intersection = line1.intersect_with_line(&line2);
        assert_eq!(intersection, Some(Vector { x: 1.0, y: 0.0 }));
    }

    #[test]
    fn test_distance() {
        let right = Line {
            origin: Vector { x: 0.0, y: 0.0 },
            direction: Vector { x: 1.0, y: 0.0 },
        };

        assert_eq!(right.distance(&Vector::new(1.0, 1.0)), 1.0);
        assert_eq!(right.distance(&Vector::new(10.0, 0.0)), 0.0);
        assert_eq!(right.distance(&Vector::new(1.0, -2.0)), -2.0);

        let up = Line {
            origin: Vector { x: 1.0, y: 1.0 },
            direction: Vector { x: 0.0, y: 1.0 },
        };
        assert_eq!(up.distance(&Vector::new(0.0, 0.0)), 1.0);
    }
}
