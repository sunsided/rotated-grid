//! Visualizes how the grid is computed: the rectangle is rotated, its rows are intersected
//! with the rotated rectangle, and the resulting grid points are rotated back.
//!
//! Opens a window that animates the rotation. The left half shows the rotated space,
//! the right half the original (unrotated) space. Press Escape to quit.

use minifb::{Key, Window, WindowOptions};
mod geometry;

use geometry::{Line, Vector};
use rotated_grid::Angle;
use std::error::Error;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

const WHITE: u32 = 0xFF_FF_FF;
const BLACK: u32 = 0x00_00_00;
const GRAY: u32 = 0x80_80_80;
const RED: u32 = 0xFF_2C_13;
const MAGENTA: u32 = 0xFF_00_FF;
const BLUE: u32 = 0x45_6E_91;

fn main() -> Result<(), Box<dyn Error>> {
    let dx = 10.0;
    let dy = 20.0;
    let x0_ = 5.0;
    let y0_ = 5.0;

    let mut window = Window::new(
        "Rotated Rectangle (Esc to quit)",
        2 * WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;
    window.set_target_fps(30);

    let mut canvas = Canvas::new(2 * WIDTH, HEIGHT);
    let mut angle = 0.0f64;
    let mut increment = 0.3;
    let mut offset_rotation_angle = 0.0f64;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Sweep the angle back and forth.
        angle += increment;
        if !(-180.0..=180.0).contains(&angle) {
            increment = -increment;
            angle = angle.clamp(-180.0, 180.0);
        }

        // Rotate the x0/y0 offset for the effect.
        let (offset_sin, offset_cos) = offset_rotation_angle.sin_cos();
        let x0 = x0_ * offset_cos + y0_ * offset_sin;
        let y0 = x0_ * offset_sin - y0_ * offset_cos;
        offset_rotation_angle += 0.1;

        render(&mut canvas, Angle::from_degrees(angle), dx, dy, x0, y0);
        window.update_with_buffer(&canvas.pixels, 2 * WIDTH, HEIGHT)?;
    }

    Ok(())
}

fn render(canvas: &mut Canvas, angle: Angle, dx: f64, dy: f64, x0: f64, y0: f64) {
    canvas.pixels.fill(WHITE);

    // The right half of the window shows the unrotated space.
    let unrotated = Vector::new(WIDTH as f64, 0.0);

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
    let (w, h) = (WIDTH as f64, HEIGHT as f64);
    canvas.line(&Vector::new(0.0, center.y), &Vector::new(w, center.y), RED);
    canvas.line(&Vector::new(center.x, 0.0), &Vector::new(center.x, h), RED);

    // Unrotated rectangle.
    canvas.rectangle(
        &(tl + unrotated),
        &(tr + unrotated),
        &(bl + unrotated),
        &(br + unrotated),
        BLACK,
    );

    // The rotated rectangle.
    let tl = tl.rotate_around(&center, sin, cos);
    let tr = tr.rotate_around(&center, sin, cos);
    let bl = bl.rotate_around(&center, sin, cos);
    let br = br.rotate_around(&center, sin, cos);
    canvas.rectangle(&tl, &tr, &bl, &br, BLACK);

    // Determine line segments describing the rotated rectangle.
    let rect_top = Line::from_points(tr, tl);
    let rect_left = Line::from_points(tl, bl);
    let rect_bottom = Line::from_points(bl, br);
    let rect_right = Line::from_points(tr, br);

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
    canvas.rectangle(&tl, &tr, &bl, &br, GRAY);

    // Determine (half) the number and offset of rows in rotated space.
    let y_count_half = ((extent.y / dy) * 0.5).floor();
    let start_y = center.y - (y_count_half * dy) + y0;
    let mut y = ((tl.y - start_y) / dy).ceil() * dy + start_y;

    while y <= br.y {
        // Determine the intersection of the ray from the given row with the rectangle.
        let ray = Line::from_points(Vector::new(tl.x, y), Vector::new(tl.x + extent.x, y));
        if let Some((start, end)) = find_intersections(
            &ray,
            &rect_top,
            &rect_left,
            &rect_bottom,
            &rect_right,
            rect_width,
            rect_height,
        ) {
            canvas.point(&start, 4, MAGENTA);
            canvas.point(&end, 4, MAGENTA);
            canvas.line(&start, &end, MAGENTA);

            // Determine (half) the number and offset of columns in rotated space, along the row.
            let x_count_half = ((extent.x / dx) * 0.5).floor();
            let start_x = center.x - (x_count_half * dx) + x0;
            let mut x = ((start.x - start_x) / dx).ceil() * dx + start_x;
            while x <= end.x {
                let point = Vector::new(x, y);
                canvas.point(&point, 2, BLUE);

                // Un-rotate the point for visualization.
                let point = point.rotate_around(&center, -sin, cos);
                canvas.point(&(point + unrotated), 2, BLUE);

                x += dx;
            }
        }

        y += dy;
    }
}

/// A minimal software framebuffer in minifb's `0RGB` pixel format.
struct Canvas {
    pixels: Vec<u32>,
    width: usize,
    height: usize,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![WHITE; width * height],
            width,
            height,
        }
    }

    fn set(&mut self, x: i64, y: i64, color: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = color;
        }
    }

    fn line(&mut self, from: &Vector, to: &Vector, color: u32) {
        let delta = *to - *from;
        let steps = delta.x.abs().max(delta.y.abs()).ceil().max(1.0);
        for i in 0..=steps as i64 {
            let p = *from + delta * (i as f64 / steps);
            self.set(p.x.round() as i64, p.y.round() as i64, color);
        }
    }

    fn point(&mut self, center: &Vector, radius: i64, color: u32) {
        let (cx, cy) = (center.x.round() as i64, center.y.round() as i64);
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= radius * radius {
                    self.set(cx + x, cy + y, color);
                }
            }
        }
    }

    fn rectangle(&mut self, tl: &Vector, tr: &Vector, bl: &Vector, br: &Vector, color: u32) {
        self.line(tl, tr, color);
        self.line(tr, br, color);
        self.line(br, bl, color);
        self.line(bl, tl, color);
    }
}

/// Finds the first and last intersection points of the ray with the rotated rectangle.
fn find_intersections(
    ray: &Line,
    top: &Line,
    left: &Line,
    bottom: &Line,
    right: &Line,
    width: f64,
    height: f64,
) -> Option<(Vector, Vector)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;

    let hits = [
        ray.intersect(top, width),
        ray.intersect(bottom, width),
        ray.intersect(left, height),
        ray.intersect(right, height),
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
