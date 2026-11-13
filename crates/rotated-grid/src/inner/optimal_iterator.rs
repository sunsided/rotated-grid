use crate::inner::line::Line;
use crate::inner::vector::Vector;
use crate::Angle;

/// An iterator for grid coordinates in rotated rectangle space.
/// Only coordinates that are guaranteed to lie within the original
/// axis-aligned rectangle are produced.
pub struct OptimalIterator {
    /// The next x coordinate on the current row.
    x: f64,
    /// The last x coordinate on the current row.
    row_end_x: f64,
    /// The y coordinate of the current row.
    row_y: f64,
    /// The y coordinate of the next row.
    next_y: f64,
    /// The x coordinate of the grid column that all rows are aligned to.
    start_x: f64,
    min_x: f64,
    max_y: f64,
    center: Vector,
    /// The extent of the axis-aligned bounding box around the rotated rectangle.
    extent: Vector,
    /// The width and height of the original rectangle, i.e. its edge lengths.
    size: Vector,
    delta: Vector,
    /// The line segment describing the top edge of the rotated rectangle.
    rect_top: Line,
    /// The line segment describing the left edge of the rotated rectangle.
    rect_left: Line,
    /// The line segment describing the bottom edge of the rotated rectangle.
    rect_bottom: Line,
    /// The line segment describing the right edge of the rotated rectangle.
    rect_right: Line,
}

impl OptimalIterator {
    /// Creates a new iterator from the specified axis-aligned (i.e., unrotated) coordinates.
    #[allow(clippy::too_many_arguments)]
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
        let size = Vector::new(rect_width, rect_height);
        let center = (tl + tr + bl + br) * 0.25;

        // Calculate the rotated rectangle.
        let tl = tl.rotate_around_with(&center, sin, cos);
        let tr = tr.rotate_around_with(&center, sin, cos);
        let bl = bl.rotate_around_with(&center, sin, cos);
        let br = br.rotate_around_with(&center, sin, cos);

        // Determine line segments describing the rotated rectangle.
        let rect_top = Line::from_points(tr, &tl);
        let rect_left = Line::from_points(tl, &bl);
        let rect_bottom = Line::from_points(bl, &br);
        let rect_right = Line::from_points(tr, &br);

        // Obtain the Axis-Aligned Bounding Box that wraps the rotated rectangle.
        // The absolute values make this valid for rotations in any direction.
        let (sin_abs, cos_abs) = (sin.abs(), cos.abs());
        let extent = Vector::new(
            size.x * cos_abs + size.y * sin_abs,
            size.x * sin_abs + size.y * cos_abs,
        );
        let tl = center - extent * 0.5;
        let br = center + extent * 0.5;

        // Determine (half) the number and offset of rows in rotated space.
        let y_count_half = ((extent.y / dy) * 0.5).floor();
        let start_y = center.y - (y_count_half * dy) + y0;
        let y = ((tl.y - start_y) / dy).ceil() * dy + start_y;

        // Determine (half) the number and offset of columns in rotated space.
        let x_count_half = ((extent.x / dx) * 0.5).floor();
        let start_x = center.x - (x_count_half * dx) + x0;

        Self {
            // An empty current row; the first call to `next` sets up the first row.
            x: f64::INFINITY,
            row_end_x: f64::NEG_INFINITY,
            row_y: y,
            next_y: y,
            start_x,
            min_x: tl.x,
            max_y: br.y,
            center,
            extent,
            size,
            delta: Vector::new(dx, dy),
            rect_top,
            rect_left,
            rect_bottom,
            rect_right,
        }
    }

    /// Returns the center of the rectangle.
    #[inline(always)]
    pub const fn center(&self) -> &Vector {
        &self.center
    }

    /// Returns the extent of the axis-aligned bounding box that is iterated over.
    #[inline(always)]
    pub(crate) const fn extent(&self) -> &Vector {
        &self.extent
    }

    /// Finds the first and last intersection points of the ray with the rotated rectangle.
    fn find_intersections(&self, ray: &Line) -> Option<(Vector, Vector)> {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        let width = self.size.x;
        let height = self.size.y;

        let top = ray.calculate_intersection_t(&self.rect_top, width);
        let bottom = ray.calculate_intersection_t(&self.rect_bottom, width);
        let left = ray.calculate_intersection_t(&self.rect_left, height);
        let right = ray.calculate_intersection_t(&self.rect_right, height);

        for t in [top, bottom, left, right].into_iter().flatten() {
            min = min.min(t);
            max = max.max(t);
        }

        if min.is_finite() && max.is_finite() {
            Some((ray.project_out(min), ray.project_out(max)))
        } else {
            None
        }
    }
}

impl OptimalIterator {
    /// Advances to the next row that intersects the rectangle and returns its first point.
    /// Runs once per row, so it is kept out of the inlined per-point path.
    fn next_row(&mut self) -> Option<Vector> {
        loop {
            let y = self.next_y;
            if y > self.max_y {
                return None;
            }
            self.next_y += self.delta.y;

            // Determine the intersection of the ray along the row with the rectangle.
            let row_start = Vector::new(self.min_x, y);
            let row_end = Vector::new(self.min_x + self.extent.x, y);
            let ray = Line::from_points(row_start, &row_end);
            let Some((start, end)) = self.find_intersections(&ray) else {
                continue;
            };

            // The first x coordinate along the row that lies on the grid
            // and is not smaller than the start coordinate.
            let dx = self.delta.x;
            let x = ((start.x - self.start_x) / dx).ceil() * dx + self.start_x;
            if x > end.x {
                continue;
            }

            self.x = x + dx;
            self.row_end_x = end.x;
            self.row_y = y;
            return Some(Vector::new(x, y));
        }
    }
}

impl Iterator for OptimalIterator {
    type Item = Vector;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let x = self.x;
        if x <= self.row_end_x {
            self.x += self.delta.x;
            return Some(Vector::new(x, self.row_y));
        }

        self.next_row()
    }
}
