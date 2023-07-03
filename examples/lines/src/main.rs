use opencv::core::{norm, Mat, Point, Scalar, CV_8UC1, CV_8UC3};
use opencv::highgui::{imshow, wait_key};
use opencv::imgproc::{circle, line, FILLED, LINE_AA};
use opencv::prelude::*;
use rotated_grid::{Angle, Line, LineSegment, Vector};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 480;

    let bg_color = Scalar::from((255.0, 255.0, 255.0, 0.0));

    let mut image = Mat::new_rows_cols_with_default(HEIGHT as _, WIDTH as _, CV_8UC3, bg_color)?;

    let tl = Vector::new(100.0, 100.0);
    let tr = Vector::new(540.0, 100.0);
    let bl = Vector::new(100.0, 380.0);
    let br = Vector::new(540.0, 380.0);
    let center = (tl + tr + bl + br) / 4.0;

    let up = Vector::new(0.0, 100.0) + center;

    let top = LineSegment::from_points(tr, &tl);
    let left = LineSegment::from_points(tl, &bl);
    let bottom = LineSegment::from_points(bl, &br);
    let right = LineSegment::from_points(br, &tr);

    let mut angle = 0.0;
    loop {
        angle += 1.0;
        if angle > 360.0 {
            angle -= 360.0;
        }

        let angle = Angle::from_degrees(angle as _);
        image.set(bg_color)?;

        let up = up.rotate_around(&center, angle);
        let up_l = Line::from_points(center, &up);

        let top_intersect = intersect_with_rectangle(&up_l, &top, &left, &bottom, &right);
        let length_inside_sq = (top_intersect - center).norm_sq();

        let orthogonal = Line::new(top_intersect, up_l.direction().orthogonal());
        let other_top_intersect =
            find_other_intersection(orthogonal.clone(), &top, &left, &bottom, &right)
                .or_else(|| find_other_intersection(-orthogonal, &top, &left, &bottom, &right));

        render_outside_part(&mut image, &tl, &up_l, &top_intersect, length_inside_sq)?;
        render_outside_part(&mut image, &tr, &up_l, &top_intersect, length_inside_sq)?;
        render_outside_part(&mut image, &bl, &up_l, &top_intersect, length_inside_sq)?;
        render_outside_part(&mut image, &br, &up_l, &top_intersect, length_inside_sq)?;

        let orthogonal = Line::new(center, up_l.direction().orthogonal());
        let base_intersect =
            find_other_intersection(orthogonal.clone(), &top, &left, &bottom, &right)
                .expect("expect a line to one side");
        let other_base_intersect =
            find_other_intersection(-orthogonal.clone(), &top, &left, &bottom, &right)
                .expect("expect a line to one side");

        circle(
            &mut image,
            vec2point(&top_intersect),
            4,
            Scalar::default(),
            FILLED,
            LINE_AA,
            0,
        )?;

        if let Some(other_intersect) = other_top_intersect {
            circle(
                &mut image,
                vec2point(&other_intersect),
                4,
                Scalar::from((255.0, 0.0, 0.0, 0.0)),
                FILLED,
                LINE_AA,
                0,
            )?;

            line(
                &mut image,
                vec2point(&top_intersect),
                vec2point(&other_intersect),
                Scalar::from((255.0, 0.0, 0.0, 0.0)),
                1,
                LINE_AA,
                0,
            )?;
        }

        circle(
            &mut image,
            vec2point(&base_intersect),
            4,
            Scalar::from((0.0, 255.0, 0.0, 0.0)),
            FILLED,
            LINE_AA,
            0,
        )?;
        line(
            &mut image,
            vec2point(&center),
            vec2point(&base_intersect),
            Scalar::from((0.0, 255.0, 0.0, 0.0)),
            1,
            LINE_AA,
            0,
        )?;

        circle(
            &mut image,
            vec2point(&other_base_intersect),
            4,
            Scalar::from((0.0, 255.0, 0.0, 0.0)),
            FILLED,
            LINE_AA,
            0,
        )?;
        line(
            &mut image,
            vec2point(&center),
            vec2point(&other_base_intersect),
            Scalar::from((0.0, 255.0, 0.0, 0.0)),
            1,
            LINE_AA,
            0,
        )?;

        line(
            &mut image,
            vec2point(&center),
            vec2point(&up),
            Scalar::default(),
            4,
            LINE_AA,
            0,
        )?;

        line(
            &mut image,
            vec2point(&center),
            vec2point(&top_intersect),
            Scalar::default(),
            1,
            LINE_AA,
            0,
        )?;

        // center
        circle(
            &mut image,
            vec2point(&center),
            1,
            Scalar::default(),
            FILLED,
            LINE_AA,
            0,
        )?;

        draw_rectangle(&mut image, &tl, &tr, &bl, &br)?;

        imshow("Lines", &image)?;
        if wait_key(33)? > 1 {
            return Ok(());
        }
    }
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
fn find_other_intersection(
    orthogonal: Line,
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
) -> Result<(), Box<dyn Error>> {
    // top side
    line(
        &mut image,
        vec2point(&tl),
        vec2point(&tr),
        Scalar::default(),
        1,
        LINE_AA,
        0,
    )?;

    // left side
    line(
        &mut image,
        vec2point(&tl),
        vec2point(&bl),
        Scalar::default(),
        1,
        LINE_AA,
        0,
    )?;

    // bottom side
    line(
        &mut image,
        vec2point(&bl),
        vec2point(&br),
        Scalar::default(),
        1,
        LINE_AA,
        0,
    )?;

    // right side
    line(
        &mut image,
        vec2point(&br),
        vec2point(&tr),
        Scalar::default(),
        1,
        LINE_AA,
        0,
    )?;
    Ok(())
}

fn vec2point(vector: &Vector) -> Point {
    Point::new(vector.x as _, vector.y as _)
}
