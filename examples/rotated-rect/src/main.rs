use opencv::core::{Mat, Point, Scalar, Scalar_, VecN, CV_8UC3};
use opencv::highgui::{imshow, wait_key};
use opencv::imgproc::{circle, line, FILLED, LINE_AA};
use opencv::prelude::*;
use rotated_grid::{Angle, Line, LineSegment, OptimalIterator, OptimalXIterator, Vector};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 480;

    let dx = 10.0;
    let dy = 20.0;

    let x0_ = 5.0;
    let y0_ = 5.0;

    let bg_color = Scalar::from((255.0, 255.0, 255.0, 0.0));

    let mut rotated_space =
        Mat::new_rows_cols_with_default(HEIGHT as _, WIDTH as _, CV_8UC3, bg_color)?;
    let mut unrotated_space =
        Mat::new_rows_cols_with_default(HEIGHT as _, WIDTH as _, CV_8UC3, bg_color)?;

    // Form the regular rectangle.
    let tl = Vector::new(150.0, 130.0);
    let tr = Vector::new(490.0, 130.0);
    let bl = Vector::new(150.0, 350.0);
    let br = Vector::new(490.0, 350.0);

    let center = (tl + tr + bl + br) / 4.0;

    let rect_width = (tr - tl).norm();
    let rect_height = (bl - tl).norm();
    let extent = Vector::new(rect_width, rect_height);

    // Note that the calculations are defined for the rotation angles of 0..90 degrees!
    let mut angle = 0.0;
    let mut increment = 0.3;
    let min_angle = 0.0; // TODO: Allow for negative angles (e.g. -90).
    let max_angle = 90.0;

    let mut offset_rotation_angle = 0.0f64;
    loop {
        // Rotate the grid.
        angle += increment;
        if angle >= max_angle {
            increment *= -1.0;
            angle = max_angle;
        } else if angle < min_angle {
            increment *= -1.0;
            angle = min_angle;
        }

        // Rotate the x0/y0 offset for the effect.
        let (offset_sin, offset_cos) = offset_rotation_angle.sin_cos();
        let x0 = x0_ * offset_cos + y0_ * offset_sin;
        let y0 = x0_ * offset_sin - y0_ * offset_cos;
        offset_rotation_angle += 0.1;

        // Initialize the iterator up here to use original values.
        let mut iter =
            OptimalIterator::new(tl, tr, bl, br, Angle::from_degrees(angle), dx, dy, x0, y0);

        // The center rectangle.
        draw_line(
            &mut rotated_space,
            &Vector::new(0.0, center.y),
            &Vector::new(WIDTH as _, center.y),
            Scalar::new(19.0, 44.0, 255.0, 0.0),
        )?;
        draw_line(
            &mut rotated_space,
            &Vector::new(center.x, 0.0),
            &Vector::new(center.x, HEIGHT as _),
            Scalar::new(19.0, 44.0, 255.0, 0.0),
        )?;

        let angle = Angle::from_degrees(angle as _);
        let (sin, cos) = angle.sin_cos();

        // Unrotated rectangle.
        unrotated_space.set(bg_color)?;
        draw_rectangle(&mut unrotated_space, &tl, &tr, &bl, &br, Scalar::default())?;

        // The rotated rectangle.
        rotated_space.set(bg_color)?;
        let tl = tl.rotate_around(&center, angle);
        let tr = tr.rotate_around(&center, angle);
        let bl = bl.rotate_around(&center, angle);
        let br = br.rotate_around(&center, angle);
        draw_rectangle(&mut rotated_space, &tl, &tr, &bl, &br, Scalar::default())?;

        // Draw the Axis-Aligned Bounding Box that wraps the rotated rectangle.
        let extent = Vector::new(
            extent.x * cos + extent.y * sin,
            extent.x * sin + extent.y * cos,
        );
        let tl = center - extent * 0.5;
        let br = center + extent * 0.5;
        let tr = Vector::new(br.x, tl.y);
        let bl = Vector::new(tl.x, br.y);
        draw_rectangle(
            &mut rotated_space,
            &tl,
            &tr,
            &bl,
            &br,
            Scalar::new(128.0, 128.0, 128.0, 0.0),
        )?;

        while let Some((point, grid_coord)) = iter.next() {
            // Draw point in rotated space.
            draw_point_small(
                &mut rotated_space,
                &point,
                Scalar::new(145.0, 110.0, 69.0, 0.0),
            )?;

            // Draw point in unrotated space.
            draw_point_small(
                &mut unrotated_space,
                &Vector::new(grid_coord.x, grid_coord.y),
                Scalar::new(145.0, 110.0, 69.0, 0.0),
            )?;
        }

        imshow("Rotated Rectangle", &rotated_space)?;
        imshow("Unrotated Rectangle", &unrotated_space)?;
        if wait_key(33)? > 1 {
            return Ok(());
        }
    }
}

fn draw_line_with_dot(
    mut image: &mut Mat,
    from: &Vector,
    to: &Vector,
    color: Scalar_<f64>,
) -> Result<(), Box<dyn Error>> {
    draw_point(&mut image, to, color)?;
    draw_line(&mut image, from, to, color)?;
    Ok(())
}

fn draw_line(
    mut image: &mut Mat,
    start: &Vector,
    end: &Vector,
    color: Scalar_<f64>,
) -> Result<(), Box<dyn Error>> {
    line(
        &mut image,
        vec2point(&start),
        vec2point(&end),
        color,
        1,
        LINE_AA,
        0,
    )?;
    Ok(())
}

fn draw_point(
    mut image: &mut Mat,
    point: &Vector,
    color: VecN<f64, 4>,
) -> Result<(), Box<dyn Error>> {
    circle(&mut image, vec2point(point), 4, color, FILLED, LINE_AA, 0)?;
    Ok(())
}

fn draw_point_small(
    mut image: &mut Mat,
    point: &Vector,
    color: VecN<f64, 4>,
) -> Result<(), Box<dyn Error>> {
    circle(&mut image, vec2point(point), 2, color, FILLED, LINE_AA, 0)?;
    Ok(())
}

/// Finds the intersection point that is furthest from the specified line's origin,
/// assuming the line's origin already is an intersection point.
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

    if let Some(t) = ray.calculate_intersection_t(&top.normalized(), width) {
        min = min.min(t);
        max = max.max(t);
    }

    if let Some(t) = ray.calculate_intersection_t(&bottom.normalized(), width) {
        min = min.min(t);
        max = max.max(t);
    }

    if let Some(t) = ray.calculate_intersection_t(&left.normalized(), height) {
        min = min.min(t);
        max = max.max(t);
    }

    if let Some(t) = ray.calculate_intersection_t(&right.normalized(), height) {
        min = min.min(t);
        max = max.max(t);
    }

    if min.is_finite() && max.is_finite() {
        Some((
            *ray.origin() + *ray.direction() * min,
            *ray.origin() + *ray.direction() * max,
        ))
    } else {
        None
    }
}

fn draw_rectangle(
    mut image: &mut Mat,
    tl: &Vector,
    tr: &Vector,
    bl: &Vector,
    br: &Vector,
    color: Scalar,
) -> Result<(), Box<dyn Error>> {
    // top side
    line(
        &mut image,
        vec2point(&tl),
        vec2point(&tr),
        color,
        1,
        LINE_AA,
        0,
    )?;

    // left side
    line(
        &mut image,
        vec2point(&tl),
        vec2point(&bl),
        color,
        1,
        LINE_AA,
        0,
    )?;

    // bottom side
    line(
        &mut image,
        vec2point(&bl),
        vec2point(&br),
        color,
        1,
        LINE_AA,
        0,
    )?;

    // right side
    line(
        &mut image,
        vec2point(&br),
        vec2point(&tr),
        color,
        1,
        LINE_AA,
        0,
    )?;
    Ok(())
}

fn vec2point(vector: &Vector) -> Point {
    Point::new(vector.x as _, vector.y as _)
}
