use crate::Angle;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

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

    /// Calculates the euclidean norm of the vector.
    #[inline(always)]
    pub fn normalized(&self) -> Self {
        *self / self.norm()
    }

    /// Rotates the vector counterclockwise by the specified angle.
    pub fn rotate(&self, angle: Angle) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    /// Rotates the vector counterclockwise by the specified angle expressed as its sine and cosine.
    pub fn rotate_with(&self, sin: f64, cos: f64) -> Self {
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    /// Rotates the vector counterclockwise by the specified angle.
    pub fn rotate_around(&self, pivot: &Self, angle: Angle) -> Self {
        let (sin, cos) = angle.sin_cos();

        let x0 = self.x - pivot.x;
        let y0 = self.y - pivot.y;

        let x = x0 * cos - y0 * sin;
        let y = x0 * sin + y0 * cos;

        Self {
            x: x + pivot.x,
            y: y + pivot.y,
        }
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

    /// Rotates the vector counterclockwise by the specified angle.
    pub fn rotate_around_screenspace(&self, pivot: &Self, angle: Angle) -> Self {
        let (sin, cos) = angle.sin_cos();

        let x0 = self.x - pivot.x;
        let y0 = self.y - pivot.y;

        let x = x0 * cos + y0 * sin;
        let y = -x0 * sin + y0 * cos;

        Self {
            x: x + pivot.x,
            y: y + pivot.y,
        }
    }

    /// Provides a vector orthogonal to the specified one by rotating the vector
    /// 90° counterclockwise.
    pub fn orthogonal(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    /// Calculates the dot product of two vectors.
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(&self, other: &Vector) -> f64 {
        self.x * other.y - self.y * other.x
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

impl AddAssign<Vector> for Vector {
    fn add_assign(&mut self, rhs: Vector) {
        self.x += rhs.x;
        self.y += rhs.y;
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

impl SubAssign<Vector> for Vector {
    fn sub_assign(&mut self, rhs: Vector) {
        self.x -= rhs.x;
        self.y -= rhs.y;
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

impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        rhs * self
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

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //noinspection RsApproxConstant
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
    fn test_rotate() {
        let vector = Vector { x: 1.0, y: 0.0 };
        assert_eq!(
            vector.rotate(Angle::from_degrees(0.0)).round(3),
            Vector { x: 1.0, y: 0.0 }
        );
        assert_eq!(
            vector.rotate(Angle::from_degrees(90.0)).round(3),
            Vector { x: 0.0, y: 1.0 }
        );
        assert_eq!(
            vector.rotate(Angle::from_degrees(180.0)).round(3),
            Vector { x: -1.0, y: 0.0 }
        );
        assert_eq!(
            vector.rotate(Angle::from_degrees(-90.0)).round(3),
            Vector { x: 0.0, y: -1.0 }
        );
        assert_eq!(
            vector.rotate(Angle::from_degrees(45.0)).round(3),
            Vector { x: 1.0, y: 1.0 }.normalized().round(3)
        );
    }

    #[test]
    fn test_rotate_around() {
        let vector = Vector { x: 1.0, y: 0.0 };

        // Zero rotation (around any point) results in no change.
        assert_eq!(
            vector
                .rotate_around(&vector, Angle::from_degrees(0.0))
                .round(3),
            Vector { x: 1.0, y: 0.0 }
        );

        // Any rotation around the point itself results in no change.
        assert_eq!(
            vector
                .rotate_around(&vector, Angle::from_degrees(45.0))
                .round(3),
            Vector { x: 1.0, y: 0.0 }
        );

        // Rotate around the specified pivot vector.
        assert_eq!(
            vector
                .rotate_around(&Vector { x: 1.0, y: 1.0 }, Angle::from_degrees(90.0))
                .round(3),
            Vector { x: 2.0, y: 1.0 }
        );
    }

    #[test]
    fn test_orthogonal() {
        assert_eq!(
            Vector { x: 1.0, y: 0.0 }.orthogonal(),
            Vector { x: 0.0, y: 1.0 }
        );

        assert_eq!(
            Vector { x: 0.0, y: 1.0 }.orthogonal(),
            Vector { x: -1.0, y: 0.0 }
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
