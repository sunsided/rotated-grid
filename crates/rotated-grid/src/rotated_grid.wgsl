// Rotated grid lookups for shaders, matching `rotated_grid::Grid` on the CPU.
//
// Where the CPU iterator lists every grid point, these functions answer the inverse question
// per pixel: which grid points are near this position? The lattice is derived from the same
// parameters and anchored the same way, so both sides agree on where the points are.
//
// `RotatedGrid` matches the layout of `Grid::to_uniform()`: eight `f32` values.

struct RotatedGrid {
    /// Width and height of the covered rectangle.
    size: vec2<f32>,
    /// Spacing of grid points along the rotated X and Y axes.
    spacing: vec2<f32>,
    /// Offset of the grid along the rotated X and Y axes.
    offset: vec2<f32>,
    /// Orientation in radians; positive angles turn the grid counter-clockwise
    /// in image coordinates (Y pointing down).
    angle: f32,
    _padding: f32,
}

/// Center of the rectangle; the grid is rotated around it.
fn rotated_grid_center(g: RotatedGrid) -> vec2<f32> {
    return 0.5 * g.size;
}

/// The lattice point at index (0, 0), in rotated space.
fn rotated_grid_origin(g: RotatedGrid) -> vec2<f32> {
    let cs = abs(vec2<f32>(cos(g.angle), sin(g.angle)));
    // Size of the axis-aligned bounding box around the rotated rectangle.
    let extent = vec2<f32>(
        g.size.x * cs.x + g.size.y * cs.y,
        g.size.x * cs.y + g.size.y * cs.x,
    );
    return rotated_grid_center(g) - floor(extent / g.spacing * 0.5) * g.spacing + g.offset;
}

/// Maps an image-space position into rotated space, where the grid is axis-aligned.
fn rotated_grid_to_rotated(g: RotatedGrid, p: vec2<f32>) -> vec2<f32> {
    let c = cos(g.angle);
    let s = sin(g.angle);
    let d = p - rotated_grid_center(g);
    return vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c) + rotated_grid_center(g);
}

/// Maps a rotated-space position back into image space.
fn rotated_grid_from_rotated(g: RotatedGrid, r: vec2<f32>) -> vec2<f32> {
    let c = cos(g.angle);
    let s = sin(g.angle);
    let d = r - rotated_grid_center(g);
    return vec2<f32>(d.x * c + d.y * s, -d.x * s + d.y * c) + rotated_grid_center(g);
}

/// The lattice index of the grid cell whose lower corner is the nearest grid point
/// at or before `p` along both rotated axes.
fn rotated_grid_cell(g: RotatedGrid, p: vec2<f32>) -> vec2<i32> {
    let r = rotated_grid_to_rotated(g, p);
    return vec2<i32>(floor((r - rotated_grid_origin(g)) / g.spacing));
}

/// The image-space position of the grid point with the given lattice index.
fn rotated_grid_point(g: RotatedGrid, index: vec2<i32>) -> vec2<f32> {
    let r = rotated_grid_origin(g) + vec2<f32>(index) * g.spacing;
    return rotated_grid_from_rotated(g, r);
}

/// The grid point nearest to `p`, in image space.
fn rotated_grid_nearest(g: RotatedGrid, p: vec2<f32>) -> vec2<f32> {
    let r = rotated_grid_to_rotated(g, p);
    let origin = rotated_grid_origin(g);
    let index = round((r - origin) / g.spacing);
    return rotated_grid_from_rotated(g, origin + index * g.spacing);
}

/// The four grid points around `p` (the corners of its grid cell), in image space.
/// Use these when shapes centered on grid points can be larger than half the spacing,
/// such as halftone dots in dark areas.
fn rotated_grid_neighbors(g: RotatedGrid, p: vec2<f32>) -> array<vec2<f32>, 4> {
    let cell = rotated_grid_cell(g, p);
    return array<vec2<f32>, 4>(
        rotated_grid_point(g, cell),
        rotated_grid_point(g, cell + vec2<i32>(1, 0)),
        rotated_grid_point(g, cell + vec2<i32>(0, 1)),
        rotated_grid_point(g, cell + vec2<i32>(1, 1)),
    );
}

/// Whether an image-space position lies inside the covered rectangle, as the CPU
/// iterator's points do.
fn rotated_grid_contains(g: RotatedGrid, q: vec2<f32>) -> bool {
    return all(q >= vec2<f32>(0.0)) && all(q <= g.size);
}
