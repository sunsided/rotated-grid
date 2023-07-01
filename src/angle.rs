/// An angle expressed in radians.
pub struct Angle<T>(T);

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

impl Angle<f64> {
    /// Constructs the value from an angle specified in degrees.
    pub fn from_degrees(radians: f64) -> Self {
        Self(radians.to_radians())
    }
}

impl<T: Default> Default for Angle<T> {
    fn default() -> Self {
        Self(T::default())
    }
}
