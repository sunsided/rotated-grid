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
}

impl<T: Default> Default for Angle<T> {
    fn default() -> Self {
        Self(T::default())
    }
}
