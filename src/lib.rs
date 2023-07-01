mod angle;

pub use angle::Angle;

pub struct GridIterator {
    width: f64,
    height: f64,
    rotated_width: f64,
    rotated_height: f64,
    dx: f64,
    dy: f64,
    x0: f64,
    y0: f64,
    center_x: f64,
    center_y: f64,
    start_x: f64,
    start_y: f64,
    sin_alpha: f64,
    cos_alpha: f64,
    current_x: f64,
    current_y: f64,
}

impl GridIterator {
    pub fn new(
        width: f64,
        height: f64,
        dx: f64,
        dy: f64,
        x0: f64,
        y0: f64,
        alpha: Angle<f64>,
    ) -> Self {
        let (sin_alpha, cos_alpha) = alpha.into_radians().sin_cos();

        // Calculate the dimensions of the rotated grid
        let rotated_width = (width.abs() * cos_alpha) + (height.abs() * sin_alpha);
        let rotated_height = (width.abs() * sin_alpha) + (height.abs() * cos_alpha);

        // Calculate the center of the rotated grid.
        let center_x = x0 + (width * 0.5);
        let center_y = y0 + (height * 0.5);

        // Calculate the starting point of the rotated grid.
        let start_x = center_x - (rotated_width * 0.5);
        let start_y = center_y - (rotated_height * 0.5);

        let iterator = GridIterator {
            width,
            height,
            rotated_width,
            rotated_height,
            dx,
            dy,
            x0,
            y0,
            center_x,
            center_y,
            start_x,
            start_y,
            sin_alpha,
            cos_alpha,
            current_x: 0.0,
            current_y: 0.0,
        };
        iterator
    }
}

impl Iterator for GridIterator {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let (sin, cos) = (self.sin_alpha, self.cos_alpha);

        loop {
            let x = self.start_x + self.current_x;
            let y = self.start_y + self.current_y;

            // Rotate the grid position back to the unrotated frame.
            let inv_sin = -sin;
            let inv_cos = cos;
            let unrotated_x =
                (x - self.center_x) * inv_cos - (y - self.center_y) * inv_sin + self.center_x;
            let unrotated_y =
                (x - self.center_x) * inv_sin + (y - self.center_y) * inv_cos + self.center_y;

            // Update the current position.
            self.current_x += self.dx;
            if self.current_x > self.rotated_width {
                self.current_x = 0.0;
                self.current_y += self.dy;
            }

            // Check if the grid position is within the original rectangle.
            if unrotated_x >= self.x0
                && unrotated_x <= self.x0 + self.width
                && unrotated_y >= self.y0
                && unrotated_y <= self.y0 + self.height
            {
                return Some((unrotated_x, unrotated_y));
            }

            if x > self.start_x + self.rotated_width || y > self.start_y + self.rotated_height {
                return None;
            }
        }
    }
}
