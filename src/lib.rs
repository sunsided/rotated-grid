mod angle;

pub use angle::Angle;

pub fn create_grid(
    width: usize,
    height: usize,
    dx: f64,
    dy: f64,
    x0: f64,
    y0: f64,
    alpha: Angle<f64>,
) -> Vec<(f64, f64)> {
    let width = width as f64;
    let height = height as f64;
    let mut grid_positions: Vec<(f64, f64)> = Vec::new();

    // Calculate the dimensions of the rotated grid.
    let alpha_rad = alpha.into_radians();
    let (sin, cos) = alpha_rad.sin_cos();
    let rotated_width = (width * cos) + (height * sin);
    let rotated_height = (width * sin) + (height * cos);

    // Calculate the center of the rotated grid.
    let center_x = x0 + (width * 0.5);
    let center_y = y0 + (height * 0.5);

    // Calculate the starting point of the rotated grid.
    let start_x = center_x - (rotated_width * 0.5);
    let start_y = center_y - (rotated_height * 0.5);

    // Pre-calculate the angles for back-projection.
    let (inv_sin, inv_cos) = (-alpha_rad).sin_cos();

    // Iterate over the rotated grid positions
    let mut y = start_y;
    while y <= start_y + rotated_height {
        let mut x = start_x;
        while x <= start_x + rotated_width {
            // Rotate the grid position back to the original frame.
            let unrotated_x = (x - center_x) * inv_cos - (y - center_y) * inv_sin + center_x;
            let unrotated_y = (x - center_x) * inv_sin + (y - center_y) * inv_cos + center_y;

            // Accept the coordinate if the grid position is within the original rectangle.
            if unrotated_x >= x0
                && unrotated_x <= x0 + width
                && unrotated_y >= y0
                && unrotated_y <= y0 + height
            {
                grid_positions.push((unrotated_x, unrotated_y));
            }

            x += dx;
        }

        y += dy;
    }

    grid_positions
}
