use crate::{Angle, Line, LineSegment, Vector};

/// An iterator for grid coordinates in rotated rectangle space.
/// Only coordinates that are guaranteed to lie within the original
/// axis-aligned rectangle are produced.
pub struct OptimalIterator {
    y: f64,
    tl: Vector,
    tr: Vector,
    bl: Vector,
    br: Vector,
    center: Vector,
    extent: Vector,
    delta: Vector,
    offset: Vector,
    rect_width: f64, // TODO: Summarize into vector
    rect_height: f64,
    /// The line segment describing the top edge of the rotated rectangle.
    rect_top: LineSegment,
    /// The line segment describing the left edge of the rotated rectangle.
    rect_left: LineSegment,
    /// The line segment describing the bottom edge of the rotated rectangle.
    rect_bottom: LineSegment,
    /// The line segment describing the right edge of the rotated rectangle.
    rect_right: LineSegment,
    sin: f64,
    cos: f64,
    x_iter: Option<OptimalXIterator>,
}

impl OptimalIterator {
    /// Creates a new iterator from the specified axis-aligned (i.e., unrotated) coordinates.
    pub fn new(
        tl: Vector,
        tr: Vector,
        bl: Vector,
        br: Vector,
        angle: Angle,
        dx: f64,
        dy: f64,
        x0: f64,
        y0: f64,
    ) -> Self {
        let (sin, cos) = angle.sin_cos();

        // Parameters of the axis-aligned rectangle.
        let rect_width = (tr - tl).norm();
        let rect_height = (bl - tl).norm();
        let extent = Vector::new(rect_width, rect_height);
        let center = (tl + tr + bl + br) * 0.25;

        // Calculate the rotated rectangle.
        let tl = tl.rotate_around(&center, angle);
        let tr = tr.rotate_around(&center, angle);
        let bl = bl.rotate_around(&center, angle);
        let br = br.rotate_around(&center, angle);

        // Determine line segments describing the rotated rectangle.
        let rect_top = LineSegment::from_points(tr, &tl);
        let rect_left = LineSegment::from_points(tl, &bl);
        let rect_bottom = LineSegment::from_points(bl, &br);
        let rect_right = LineSegment::from_points(tr, &br);

        // Obtain the Axis-Aligned Bounding Box that wraps the rotated rectangle.
        let extent = Vector::new(
            extent.x * cos + extent.y * sin,
            extent.x * sin + extent.y * cos,
        );
        let tl = center - extent * 0.5;
        let br = center + extent * 0.5;
        let tr = Vector::new(br.x, tl.y);
        let bl = Vector::new(tl.x, br.y);

        // Determine (half) the number and offset of rows in rotated space.
        let y_count_half = ((extent.y / dy) * 0.5).floor();
        let start_y = center.y - (y_count_half * dy) + y0;
        let y = ((tl.y - start_y) / dy).ceil() * dy + start_y;

        Self {
            y,
            tl,
            tr,
            bl,
            br,
            center,
            extent,
            delta: Vector::new(dx, dy),
            offset: Vector::new(x0, y0),
            rect_width,
            rect_height,
            rect_top,
            rect_left,
            rect_bottom,
            rect_right,
            sin,
            cos,
            x_iter: None,
        }
    }

    /// Returns the center of the rectangle.
    pub fn center(&self) -> &Vector {
        &self.center
    }

    /// Finds the intersection point that is furthest from the specified line's origin,
    /// assuming the line's origin already is an intersection point.
    fn find_intersections(&self, ray: &Line) -> Option<(Vector, Vector)> {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        let width = self.extent.x;
        let height = self.extent.y;

        if let Some(t) = ray.calculate_intersection_t(&self.rect_top.normalized(), width) {
            min = min.min(t);
            max = max.max(t);
        }

        if let Some(t) = ray.calculate_intersection_t(&self.rect_bottom.normalized(), width) {
            min = min.min(t);
            max = max.max(t);
        }

        if let Some(t) = ray.calculate_intersection_t(&self.rect_left.normalized(), height) {
            min = min.min(t);
            max = max.max(t);
        }

        if let Some(t) = ray.calculate_intersection_t(&self.rect_right.normalized(), height) {
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
}

impl Iterator for OptimalIterator {
    type Item = Vector;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.y > self.bl.y {
                return None;
            }

            if let Some(iter) = self.x_iter.as_mut() {
                if let Some(x) = iter.next() {
                    return Some(Vector::new(x, self.y));
                }

                self.y += self.delta.y;
            }

            // Obtain the rows.
            let x = self.tl.x;
            let row_start = Vector::new(x, self.y);
            let row_end = Vector::new(x + self.extent.x, self.y);

            // Determine the intersection of the ray from the given row with the rectangle.
            let ray = Line::from_points(row_start, &row_end);
            if let Some((start, end)) = self.find_intersections(&ray) {
                self.x_iter = Some(OptimalXIterator::new(
                    self.y,
                    self.center,
                    self.extent,
                    start,
                    end,
                    self.delta.x,
                    self.offset.x,
                ));
            }
        }
    }
}

/// Iterator for x coordinates along a ray
pub struct OptimalXIterator {
    x: f64,
    y: f64,
    dx: f64,
    row_end: f64,
}

impl OptimalXIterator {
    pub fn new(
        y: f64,
        center: Vector,
        extent: Vector,
        row_start: Vector,
        row_end: Vector,
        dx: f64,
        x0: f64,
    ) -> Self {
        // Determine the first x coordinate along the row that is
        // an integer multiple of dx away from the center and larger
        // than the start coordinate.
        let x_count_half = ((extent.x / dx) * 0.5).floor();
        let start_x = center.x - (x_count_half * dx) + x0;
        let x = ((row_start.x - start_x) / dx).ceil() * dx + start_x;

        Self {
            x,
            y,
            dx,
            row_end: row_end.x,
        }
    }
}

impl Iterator for OptimalXIterator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.x;
        if x > self.row_end {
            return None;
        }

        self.x += self.dx;
        Some(x)
    }
}
