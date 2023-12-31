use opencv::core::{Mat, Point, Scalar, CV_32FC3};
use opencv::highgui::{imshow, wait_key};
use opencv::imgproc::{circle, FILLED, LINE_AA};
use rotated_grid::{Angle, GridCoord, GridPositionIterator};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 440;

    let grids = [
        ("Cyan", 15.0, (255.0, 255.0, 178.0, 0.0)),
        ("Magenta", 75.0, (255.0, 178.0, 255.0, 0.0)),
        ("Yellow", 0.0, (178.0, 255.0, 255.0, 0.0)),
        ("Key", 45.0, (178.0, 178.0, 178.0, 0.0)),
    ];

    for (name, angle, color) in grids {
        let window_name = format!("{name} at {angle}°");

        let grid = GridPositionIterator::new(
            600 as _,
            400 as _,
            7.0,
            7.0,
            0.0,
            0.0,
            Angle::<f64>::from_degrees(angle),
        );

        let expected_count = grid.size_hint();
        let mut count = 0;

        let mut image =
            Mat::new_rows_cols_with_default(HEIGHT as _, WIDTH as _, CV_32FC3, Scalar::default())?;
        for GridCoord { x, y } in grid {
            count += 1;

            let center = Point::new(x as i32 + 20, y as i32 + 20);
            let radius = 1;
            let color = Scalar::from(color) / 255.0;
            circle(&mut image, center, radius, color, FILLED, LINE_AA, 0)?;
        }

        imshow(&window_name, &image)?;
        println!("{window_name}: Expected count: {expected_count:?}, actual count: {count}");
    }

    wait_key(0)?;
    Ok(())
}
