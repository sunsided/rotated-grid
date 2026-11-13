// CMYK halftone, rendered per pixel. Appended to `rotated_grid::WGSL`, which provides
// `RotatedGrid` and the `rotated_grid_*` functions.
//
// Matches the CPU `cmyk` demo: the same source gradient, separation, ink colors and dot sizes.

struct Params {
    /// One grid per ink: cyan, magenta, yellow, black.
    inks: array<RotatedGrid, 4>,
    /// Bits 0-3: ink enabled. Bit 4: show the source image instead.
    flags: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

@group(0) @binding(0) var<uniform> params: Params;

const PI: f32 = 3.14159265;
const SHOW_SOURCE: u32 = 16u;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    // One triangle that covers the whole viewport.
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

/// The image to print: hues from left to right, fading to white at the top
/// and to black at the bottom.
fn source(p: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
    let hue = p.x / size.x * 6.0;
    let t = p.y / size.y;
    let saturation = min(2.0 * t, 1.0);
    let value = min(2.0 - 2.0 * t, 1.0);

    // HSV to RGB.
    let k = (vec3<f32>(5.0, 3.0, 1.0) + hue) % 6.0;
    let f = clamp(min(k, 4.0 - k), vec3<f32>(0.0), vec3<f32>(1.0));
    return value * (1.0 - saturation * f);
}

/// Naive RGB to CMYK separation with full black replacement.
fn separate(rgb: vec3<f32>) -> vec4<f32> {
    let k = 1.0 - max(rgb.r, max(rgb.g, rgb.b));
    if k >= 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return vec4<f32>((1.0 - rgb - k) / (1.0 - k), k);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = position.xy;
    let size = params.inks[0].size;

    if (params.flags & SHOW_SOURCE) != 0u {
        return vec4<f32>(source(p, size), 1.0);
    }

    var ink_colors = array<vec3<f32>, 4>(
        vec3<f32>(0.0, 0.68, 0.94),
        vec3<f32>(0.93, 0.0, 0.55),
        vec3<f32>(1.0, 0.95, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
    );

    // Start with white paper; every dot multiplies its ink onto it.
    var paper = vec3<f32>(1.0);
    for (var ink = 0u; ink < 4u; ink++) {
        if (params.flags & (1u << ink)) == 0u {
            continue;
        }

        let grid = params.inks[ink];
        // Dots in dark areas grow past half the spacing, so check all four cell corners.
        var dots = rotated_grid_neighbors(grid, p);
        for (var i = 0u; i < 4u; i++) {
            let dot = dots[i];
            // Like the CPU iterator, only print dots inside the image.
            if !rotated_grid_contains(grid, dot) {
                continue;
            }

            // The dot covers the same share of its grid cell as the ink amount.
            let amount = separate(source(dot, size))[ink];
            if amount <= 0.0 {
                continue;
            }
            let radius = grid.spacing.x * sqrt(amount / PI);
            let coverage = clamp(radius - distance(p, dot) + 0.5, 0.0, 1.0);
            paper *= 1.0 - coverage * (1.0 - ink_colors[ink]);
        }
    }

    return vec4<f32>(paper, 1.0);
}
