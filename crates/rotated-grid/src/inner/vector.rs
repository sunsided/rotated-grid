//! Contains the [`Vector`] type.

use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    /// Constructs a new vector from the specified coordinates.
    #[inline(always)]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Rounds the coordinates to the specified number of decimals.
    /// This simplifies testing.
    #[cfg(test)]
    pub fn round(&self, decimals: u32) -> Self {
        let scale = 10_f64.powi(decimals as i32);
        Self {
            x: (self.x * scale).round() / scale,
            y: (self.y * scale).round() / scale,
        }
    }

    /// Calculates the squared euclidean norm of the vector.
    #[inline(always)]
    pub fn norm_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Calculates the euclidean norm of the vector.
    #[inline(always)]
    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// Returns the vector scaled to unit length.
    #[inline(always)]
    pub fn normalized(&self) -> Self {
        *self / self.norm()
    }

    /// Rotates the vector counterclockwise by the specified angle expressed as its sine and cosine.
    pub fn rotate_around_with(&self, pivot: &Self, sin: f64, cos: f64) -> Self {
        let x0 = self.x - pivot.x;
        let y0 = self.y - pivot.y;

        let x = x0 * cos - y0 * sin;
        let y = x0 * sin + y0 * cos;

        Self {
            x: x + pivot.x,
            y: y + pivot.y,
        }
    }

    /// Calculates the dot product of two vectors.
    #[inline(always)]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Calculates the 2D cross product of two vectors.
    #[inline(always)]
    pub fn cross(&self, other: &Vector) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// Projects a vector at a given distance alongside a direction
    /// from the current origin.
    #[inline(always)]
    pub fn project_out(&self, direction: &Vector, t: f64) -> Self {
        Self {
            x: self.x + direction.x * t,
            y: self.y + direction.y * t,
        }
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //noinspection RsApproxConstant
    #[allow(clippy::approx_constant)]
    #[test]
    fn test_normalize() {
        assert_eq!(
            Vector { x: 2.0, y: 2.0 }.normalized().round(4),
            Vector {
                x: 0.7071,
                y: 0.7071
            }
        );
    }

    #[test]
    fn test_dot() {
        assert_eq!(
            Vector { x: 1.0, y: 0.0 }.dot(&Vector { x: 1.0, y: 0.0 }),
            1.0
        );

        assert_eq!(
            Vector { x: 2.0, y: 3.0 }.dot(&Vector { x: 4.0, y: -1.0 }),
            5.0
        );
    }
}
