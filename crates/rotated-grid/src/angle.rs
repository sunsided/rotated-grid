use std::ops::Neg;

/// An angle expressed in radians.
#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
pub struct Angle(f64);

impl Angle {
    /// Constructs the value from an angle specified in radians.
    pub const fn from_radians(radians: f64) -> Self {
        Self(radians)
    }

    /// Constructs the value from an angle specified in degrees.
    pub fn from_degrees(degrees: f64) -> Self {
        Self(degrees.to_radians())
    }

    /// Converts the value back into radians.
    pub const fn into_radians(self) -> f64 {
        self.0
    }

    /// Determines the sine and cosine of the angle.
    pub fn sin_cos(&self) -> (f64, f64) {
        self.0.sin_cos()
    }
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}
