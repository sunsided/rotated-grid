//! Renders the four CMYK halftone grids side by side into `cmyk.svg` in the current directory.

use rotated_grid::{Angle, GridCoord, GridPositionIterator};
use std::error::Error;
use std::fmt::Write as _;

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: f64 = 640.0;
    const HEIGHT: f64 = 440.0;

    let grids = [
        ("Cyan", 15.0, "rgb(0,178,255)"),
        ("Magenta", 75.0, "rgb(255,0,178)"),
        ("Yellow", 0.0, "rgb(255,255,0)"),
        ("Key", 45.0, "rgb(178,178,178)"),
    ];

    let mut svg = String::new();
    for (i, (name, angle, color)) in grids.into_iter().enumerate() {
        let grid = GridPositionIterator::new(
            600.0,
            400.0,
            7.0,
            7.0,
            0.0,
            0.0,
            Angle::<f64>::from_degrees(angle),
        );

        let expected_count = grid.size_hint();
        let mut count = 0;

        // Lay out the panels in a 2×2 grid, with a 20 pixel margin around each grid.
        let (px, py) = ((i % 2) as f64 * WIDTH, (i / 2) as f64 * HEIGHT);
        let _ = writeln!(
            svg,
            r#"<g transform="translate({px} {py})" fill="{color}">"#
        );
        let _ = writeln!(
            svg,
            r#"<text x="20" y="15" font-size="12">{name} at {angle}°</text>"#
        );
        for GridCoord { x, y } in grid {
            count += 1;
            let _ = writeln!(
                svg,
                r#"<circle cx="{}" cy="{}" r="1"/>"#,
                x + 20.0,
                y + 20.0
            );
        }
        svg.push_str("</g>\n");

        println!("{name} at {angle}°: Expected count: {expected_count:?}, actual count: {count}");
    }

    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<rect width="100%" height="100%" fill="black"/>
{svg}</svg>
"#,
        w = 2.0 * WIDTH,
        h = 2.0 * HEIGHT
    );
    std::fs::write("cmyk.svg", svg)?;
    println!("Wrote cmyk.svg");
    Ok(())
}
