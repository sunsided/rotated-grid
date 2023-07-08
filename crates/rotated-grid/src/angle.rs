use std::ops::Neg;

/// An angle expressed in radians.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Angle<T = f64>(T);

impl<T> Angle<T> {
    /// Constructs the value from an angle specified in radians.
    pub fn from_radians(radians: T) -> Self {
        Self(radians)
    }

    /// Converts the value back into radians.
    pub fn into_radians(self) -> T {
        self.0
    }
}

pub trait AngleOps<T> {
    /// Determines the sine and cosine of the angle.
    fn sin_cos(&self) -> (T, T);

    /// Normalizes the specified angle such that it falls into range -PI/2..PI/2.
    fn normalize(&self) -> Self;
}

impl Angle<f64> {
    /// Constructs the value from an angle specified in degrees.
    pub fn from_degrees(radians: f64) -> Self {
        Self(radians.to_radians())
    }

    /// Determines the sine and cosine of the angle.
    pub fn sin_cos(&self) -> (f64, f64) {
        self.0.sin_cos()
    }
}

impl AngleOps<f64> for Angle<f64> {
    /// Determines the sine and cosine of the angle.
    fn sin_cos(&self) -> (f64, f64) {
        self.0.sin_cos()
    }

    /// Normalizes the specified angle such that it falls into range -PI/2..PI/2.
    fn normalize(&self) -> Self {
        use std::f64::consts::PI;
        const HALF_PI: f64 = PI * 0.5;
        let mut alpha = self.0;
        while alpha >= PI {
            alpha -= PI;
        }
        while alpha >= HALF_PI {
            alpha -= HALF_PI;
        }
        while alpha <= -PI {
            alpha += PI;
        }
        while alpha <= -HALF_PI {
            alpha += HALF_PI;
        }
        Angle(alpha)
    }
}

impl<T: Default> Default for Angle<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl Neg for Angle<f64> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}
