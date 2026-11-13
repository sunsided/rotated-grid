//! Minimal 2D geometry for drawing the intermediate steps of the grid computation.
//! The library keeps its own copy private, so this demo carries its own.

use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(&self) -> Self {
        *self / self.norm()
    }

    /// Rotates the vector counterclockwise around `pivot` by the angle given as its sine and cosine.
    pub fn rotate_around(&self, pivot: &Self, sin: f64, cos: f64) -> Self {
        let (x, y) = (self.x - pivot.x, self.y - pivot.y);
        Self::new(x * cos - y * sin + pivot.x, x * sin + y * cos + pivot.y)
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(&self, other: &Self) -> f64 {
        self.x * other.y - self.y * other.x
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vector {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f64> for Vector {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

/// A ray starting at `origin`, with a unit-length `direction`.
pub struct Line {
    origin: Vector,
    direction: Vector,
}

impl Line {
    /// Constructs a ray from `a` through `b`.
    pub fn from_points(a: Vector, b: Vector) -> Self {
        Self {
            origin: a,
            direction: (b - a).normalized(),
        }
    }

    /// The point at distance `t` along the ray.
    pub fn project_out(&self, t: f64) -> Vector {
        self.origin + self.direction * t
    }

    /// The distance along `self` at which it crosses `other`, if the crossing lies
    /// within the first `max_u` units of `other`.
    pub fn intersect(&self, other: &Self, max_u: f64) -> Option<f64> {
        let det = self.direction.cross(&other.direction);
        if det.abs() < 1e-6 {
            // Parallel or coincident.
            return None;
        }

        let delta = self.origin - other.origin;
        let t = other.direction.cross(&delta) / det;
        let u = (delta + self.direction * t).dot(&other.direction);

        (t >= 0.0 && u >= 0.0 && u <= max_u + 1e-9).then_some(t)
    }
}
