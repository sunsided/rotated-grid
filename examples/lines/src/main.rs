use opencv::core::{Mat, Point, Scalar, CV_8UC1};
use opencv::highgui::{imshow, wait_key};
use opencv::imgproc::{circle, line, FILLED, LINE_AA};
use opencv::prelude::*;
use rotated_grid::{Angle, Line, LineSegment, Vector};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 480;

    let mut image =
        Mat::new_rows_cols_with_default(HEIGHT as _, WIDTH as _, CV_8UC1, Scalar::from(255.0))?;

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
        image.set(Scalar::from(255.0))?;

        let up = up.rotate_around(&center, angle);

        let up_l = Line::from_points(center, &up);
        let intersect_top = up_l.intersect_with_segment(&top);
        let intersect_bottom = up_l.intersect_with_segment(&bottom);
        let intersect_left = up_l.intersect_with_segment(&left);
        let intersect_right = up_l.intersect_with_segment(&right);

        let intersect = intersect_top
            .or(intersect_left)
            .or(intersect_bottom)
            .or(intersect_right)
            .unwrap();

        circle(
            &mut image,
            vec2point(&intersect),
            4,
            Scalar::default(),
            FILLED,
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
            vec2point(&intersect),
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

    imshow("Lines", &image)?;
    wait_key(0)?;
    Ok(())
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
