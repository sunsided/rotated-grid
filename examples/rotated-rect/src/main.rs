use opencv::core::{norm, Mat, Point, Scalar, Scalar_, VecN, CV_8UC1, CV_8UC3};
use opencv::highgui::{imshow, wait_key};
use opencv::imgproc::{circle, line, FILLED, LINE_AA};
use opencv::prelude::*;
use rotated_grid::{Angle, Line, LineSegment, Vector};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 480;

    let dx = 10.0;
    let dy = 20.0;

    let bg_color = Scalar::from((255.0, 255.0, 255.0, 0.0));

    let mut image = Mat::new_rows_cols_with_default(HEIGHT as _, WIDTH as _, CV_8UC3, bg_color)?;

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
    loop {
        angle += increment;
        if angle >= 90.0 {
            increment *= -1.0;
            angle = 90.0;
        } else if angle < 0.0 {
            increment *= -1.0;
            angle = 0.0;
        }

        let angle = Angle::from_degrees(angle as _);
        let (sin, cos) = angle.sin_cos();
        image.set(bg_color)?;

        // The center rectangle.
        draw_line(
            &mut image,
            &Vector::new(0.0, center.y),
            &Vector::new(WIDTH as _, center.y),
            Scalar::new(19.0, 44.0, 255.0, 0.0),
        )?;
        draw_line(
            &mut image,
            &Vector::new(center.x, 0.0),
            &Vector::new(center.x, HEIGHT as _),
            Scalar::new(19.0, 44.0, 255.0, 0.0),
        )?;

        // The rotated rectangle.
        let tl = tl.rotate_around(&center, angle);
        let tr = tr.rotate_around(&center, angle);
        let bl = bl.rotate_around(&center, angle);
        let br = br.rotate_around(&center, angle);
        draw_rectangle(&mut image, &tl, &tr, &bl, &br, Scalar::default())?;

        // Determine line segments describing the rotated rectangle.
        let rect_top = LineSegment::from_points(tr, &tl);
        let rect_left = LineSegment::from_points(tl, &bl);
        let rect_bottom = LineSegment::from_points(bl, &br);
        let rect_right = LineSegment::from_points(tr, &br);

        // Draw the Axis-Aligned Bounding Box that wraps the rotated rectangle.
        let extent = Vector::new(
            extent.x * cos + extent.y * sin,
            extent.x * sin + extent.y * cos,
        );
        let tl = (center - extent * 0.5);
        let br = (center + extent * 0.5);
        let tr = Vector::new(br.x, tl.y);
        let bl = Vector::new(tl.x, br.y);
        draw_rectangle(
            &mut image,
            &tl,
            &tr,
            &bl,
            &br,
            Scalar::new(128.0, 128.0, 128.0, 0.0),
        )?;

        // Determine (half) the number and offset of rows in rotated space.
        let y_count_half = ((extent.y / dy) * 0.5).floor();
        let mut y = center.y - y_count_half * dy;
        let x = tl.x;
        while y < bl.y {
            // Draw the rows.
            let start = Vector::new(x, y);
            let end = Vector::new(x + extent.x, y);
            draw_line(
                &mut image,
                &start,
                &end,
                Scalar::new(145.0, 110.0, 69.0, 0.0),
            )?;

            // Determine the intersection of the ray from the given row with the rectangle.
            let ray = Line::from_points(start, &end);
            let segment = &rect_left;

            draw_point(&mut image, &ray.origin(), Scalar::new(0.0, 255.0, 0.0, 0.0))?;
            draw_point(
                &mut image,
                &(*ray.origin() + *ray.direction() * extent.x),
                Scalar::new(0.0, 255.0, 0.0, 0.0),
            )?;

            if let Some((start, end)) = find_intersections(
                &ray,
                &rect_top,
                &rect_left,
                &rect_bottom,
                &rect_right,
                extent.x,
                extent.y,
            ) {
                draw_point(&mut image, &start, Scalar::new(255.0, 0.0, 255.0, 0.0))?;
                draw_line_with_dot(
                    &mut image,
                    &start,
                    &end,
                    Scalar::new(255.0, 0.0, 255.0, 0.0),
                )?;
            }

            y += dy;
        }

        imshow("Rotated Rectangle Test", &image)?;
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

fn render_outside_part(
    mut image: &mut Mat,
    corner: &Vector,
    up_l: &Line,
    intersect: &Vector,
    length_inside_sq: f64,
) -> Result<(), Box<dyn Error>> {
    let dist_to_corner = up_l.dot(&corner);
    let dist_to_corner_sq = dist_to_corner * dist_to_corner;
    if dist_to_corner > 0.0 && dist_to_corner_sq > length_inside_sq {
        let topmost = up_l.clone() * dist_to_corner;

        let color = Scalar::from((0.0, 0.0, 255.0, 0.0));

        line(
            &mut image,
            vec2point(&intersect),
            vec2point(&topmost),
            Scalar::default(),
            1,
            LINE_AA,
            0,
        )?;

        circle(
            &mut image,
            vec2point(&topmost),
            4,
            color,
            FILLED,
            LINE_AA,
            0,
        )?;

        circle(&mut image, vec2point(&corner), 4, color, FILLED, LINE_AA, 0)?;

        line(
            &mut image,
            vec2point(&corner),
            vec2point(&topmost),
            color,
            1,
            LINE_AA,
            0,
        )?;
    }
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

/// Finds the intersection point that is furthest from the specified line's origin,
/// assuming the line's origin already is an intersection point.
fn find_other_intersection(
    orthogonal: &Line,
    top: &LineSegment,
    left: &LineSegment,
    bottom: &LineSegment,
    right: &LineSegment,
) -> Option<Vector> {
    let intersect_top = orthogonal.intersect_with_segment(&top);
    let intersect_bottom = orthogonal.intersect_with_segment(&bottom);
    let intersect_left = orthogonal.intersect_with_segment(&left);
    let intersect_right = orthogonal.intersect_with_segment(&right);

    let mut other_intersect = match intersect_top
        .or(intersect_bottom)
        .or(intersect_left)
        .or(intersect_right)
    {
        None => return None,
        Some(point) => point,
    };

    let current_len_sq = (other_intersect - *orthogonal.origin()).norm_sq();

    if let Some(other) = intersect_top {
        let len_sq = (other - other_intersect).norm_sq();
        if len_sq > current_len_sq {
            return Some(other);
        }
    }

    if let Some(other) = intersect_left {
        let len_sq = (other - other_intersect).norm_sq();
        if len_sq > current_len_sq {
            return Some(other);
        }
    }

    if let Some(other) = intersect_bottom {
        let len_sq = (other - other_intersect).norm_sq();
        if len_sq > current_len_sq {
            return Some(other);
        }
    }

    if let Some(other) = intersect_right {
        let len_sq = (other - other_intersect).norm_sq();
        if len_sq > current_len_sq {
            return Some(other);
        }
    }

    // The points coincide, or there was exactly one result.
    if current_len_sq > 1e-4 {
        Some(other_intersect)
    } else {
        None
    }
}

fn intersect_with_rectangle(
    up_direction: &Line,
    top: &LineSegment,
    left: &LineSegment,
    bottom: &LineSegment,
    right: &LineSegment,
) -> Vector {
    let intersect_top = up_direction.intersect_with_segment(&top);
    let intersect_bottom = up_direction.intersect_with_segment(&bottom);
    let intersect_left = up_direction.intersect_with_segment(&left);
    let intersect_right = up_direction.intersect_with_segment(&right);
    intersect_top
        .or(intersect_left)
        .or(intersect_bottom)
        .or(intersect_right)
        .expect("line intersects with at least one edge")
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
