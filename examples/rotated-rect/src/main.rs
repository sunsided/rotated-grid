//! Visualizes how the grid is computed: the rectangle is rotated, its rows are intersected
//! with the rotated rectangle, and the resulting grid points are rotated back.
//!
//! Writes one `rotated-rect_<angle>.svg` per angle into the current directory. The left half
//! shows the rotated space, the right half the original (unrotated) space.

use rotated_grid::{
    inner::line::Line, inner::line_segment::LineSegment, inner::vector::Vector, Angle,
};
use std::error::Error;
use std::fmt::Write as _;

const WIDTH: f64 = 640.0;
const HEIGHT: f64 = 480.0;

fn main() -> Result<(), Box<dyn Error>> {
    let dx = 10.0;
    let dy = 20.0;
    let x0 = 5.0;
    let y0 = 5.0;

    for degrees in [-60.0, 0.0, 30.0, 135.0] {
        let svg = render(Angle::from_degrees(degrees), dx, dy, x0, y0);
        let path = format!("rotated-rect_{degrees}.svg");
        std::fs::write(&path, svg)?;
        println!("Wrote {path}");
    }

    Ok(())
}

fn render(angle: Angle, dx: f64, dy: f64, x0: f64, y0: f64) -> String {
    let mut rotated_space = String::new();
    let mut unrotated_space = String::new();

    // Form the regular rectangle.
    let tl = Vector::new(150.0, 130.0);
    let tr = Vector::new(490.0, 130.0);
    let bl = Vector::new(150.0, 350.0);
    let br = Vector::new(490.0, 350.0);
    let center = (tl + tr + bl + br) / 4.0;

    let rect_width = (tr - tl).norm();
    let rect_height = (bl - tl).norm();
    let (sin, cos) = angle.sin_cos();

    // The center lines.
    let red = "rgb(255,44,19)";
    draw_line(
        &mut rotated_space,
        &Vector::new(0.0, center.y),
        &Vector::new(WIDTH, center.y),
        red,
    );
    draw_line(
        &mut rotated_space,
        &Vector::new(center.x, 0.0),
        &Vector::new(center.x, HEIGHT),
        red,
    );

    // Unrotated rectangle.
    draw_rectangle(&mut unrotated_space, &tl, &tr, &bl, &br, "black");

    // The rotated rectangle.
    let tl = tl.rotate_around(&center, angle);
    let tr = tr.rotate_around(&center, angle);
    let bl = bl.rotate_around(&center, angle);
    let br = br.rotate_around(&center, angle);
    draw_rectangle(&mut rotated_space, &tl, &tr, &bl, &br, "black");

    // Determine line segments describing the rotated rectangle.
    let rect_top = LineSegment::from_points(tr, &tl);
    let rect_left = LineSegment::from_points(tl, &bl);
    let rect_bottom = LineSegment::from_points(bl, &br);
    let rect_right = LineSegment::from_points(tr, &br);

    // Draw the Axis-Aligned Bounding Box that wraps the rotated rectangle.
    let (sin_abs, cos_abs) = (sin.abs(), cos.abs());
    let extent = Vector::new(
        rect_width * cos_abs + rect_height * sin_abs,
        rect_width * sin_abs + rect_height * cos_abs,
    );
    let tl = center - extent * 0.5;
    let br = center + extent * 0.5;
    let tr = Vector::new(br.x, tl.y);
    let bl = Vector::new(tl.x, br.y);
    draw_rectangle(&mut rotated_space, &tl, &tr, &bl, &br, "gray");

    // Determine (half) the number and offset of rows in rotated space.
    let y_count_half = ((extent.y / dy) * 0.5).floor();
    let start_y = center.y - (y_count_half * dy) + y0;
    let mut y = ((tl.y - start_y) / dy).ceil() * dy + start_y;

    let magenta = "rgb(255,0,255)";
    let blue = "rgb(69,110,145)";
    while y <= br.y {
        // Determine the intersection of the ray from the given row with the rectangle.
        let ray = Line::from_points(Vector::new(tl.x, y), &Vector::new(tl.x + extent.x, y));
        if let Some((start, end)) = find_intersections(
            &ray,
            &rect_top,
            &rect_left,
            &rect_bottom,
            &rect_right,
            rect_width,
            rect_height,
        ) {
            draw_point(&mut rotated_space, &start, 4.0, magenta);
            draw_point(&mut rotated_space, &end, 4.0, magenta);
            draw_line(&mut rotated_space, &start, &end, magenta);

            // Determine (half) the number and offset of columns in rotated space, along the row.
            let x_count_half = ((extent.x / dx) * 0.5).floor();
            let start_x = center.x - (x_count_half * dx) + x0;
            let mut x = ((start.x - start_x) / dx).ceil() * dx + start_x;
            while x <= end.x {
                let point = Vector::new(x, y);
                draw_point(&mut rotated_space, &point, 2.0, blue);

                // Un-rotate the point for visualization.
                let point = point.rotate_around_with(&center, -sin, cos);
                draw_point(&mut unrotated_space, &point, 2.0, blue);

                x += dx;
            }
        }

        y += dy;
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{HEIGHT}" viewBox="0 0 {w} {HEIGHT}">
<rect width="100%" height="100%" fill="white"/>
<g>{rotated_space}</g>
<g transform="translate({WIDTH} 0)">{unrotated_space}</g>
</svg>
"#,
        w = 2.0 * WIDTH
    )
}

fn draw_line(svg: &mut String, from: &Vector, to: &Vector, color: &str) {
    let _ = writeln!(
        svg,
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{color}"/>"#,
        from.x, from.y, to.x, to.y
    );
}

fn draw_point(svg: &mut String, point: &Vector, radius: f64, color: &str) {
    let _ = writeln!(
        svg,
        r#"<circle cx="{}" cy="{}" r="{radius}" fill="{color}"/>"#,
        point.x, point.y
    );
}

fn draw_rectangle(
    svg: &mut String,
    tl: &Vector,
    tr: &Vector,
    bl: &Vector,
    br: &Vector,
    color: &str,
) {
    let _ = writeln!(
        svg,
        r#"<polygon points="{},{} {},{} {},{} {},{}" fill="none" stroke="{color}"/>"#,
        tl.x, tl.y, tr.x, tr.y, br.x, br.y, bl.x, bl.y
    );
}

/// Finds the first and last intersection points of the ray with the rotated rectangle.
fn find_intersections(
    ray: &Line,
    top: &LineSegment,
    left: &LineSegment,
    bottom: &LineSegment,
    right: &LineSegment,
    width: f64,
    height: f64,
) -> Option<(Vector, Vector)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;

    let hits = [
        ray.calculate_intersection_t(&top.normalized(), width),
        ray.calculate_intersection_t(&bottom.normalized(), width),
        ray.calculate_intersection_t(&left.normalized(), height),
        ray.calculate_intersection_t(&right.normalized(), height),
    ];
    for t in hits.into_iter().flatten() {
        min = min.min(t);
        max = max.max(t);
    }

    if min.is_finite() && max.is_finite() {
        Some((ray.project_out(min), ray.project_out(max)))
    } else {
        None
    }
}
