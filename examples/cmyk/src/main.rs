//! Interactive CMYK halftone demo.
//!
//! A generated color image is separated into cyan, magenta, yellow and black. Each ink is
//! printed as dots on its own rotated grid (C 15°, M 75°, Y 0°, K 45°), with the dot size
//! following the amount of that ink. The inks multiply on white paper, which produces the
//! typical rosette pattern.
//!
//! Keys: `1`–`4` toggle C/M/Y/K, `+`/`-` change the dot spacing, `Space` shows the source
//! image, `Esc` quits.

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rotated_grid::{Angle, Grid, GridCoord};
use std::error::Error;

const WIDTH: usize = 800;
const HEIGHT: usize = 500;

/// An ink: its name, screen angle, the RGB it lets through and how it is separated from RGB.
struct Ink {
    name: &'static str,
    angle: f64,
    color: [f32; 3],
    amount: fn(Cmyk) -> f32,
}

const INKS: [Ink; 4] = [
    Ink {
        name: "C",
        angle: 15.0,
        color: [0.0, 0.68, 0.94],
        amount: |c| c.c,
    },
    Ink {
        name: "M",
        angle: 75.0,
        color: [0.93, 0.0, 0.55],
        amount: |c| c.m,
    },
    Ink {
        name: "Y",
        angle: 0.0,
        color: [1.0, 0.95, 0.0],
        amount: |c| c.y,
    },
    Ink {
        name: "K",
        angle: 45.0,
        color: [0.0, 0.0, 0.0],
        amount: |c| c.k,
    },
];

#[derive(Copy, Clone)]
struct Cmyk {
    c: f32,
    m: f32,
    y: f32,
    k: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut window = Window::new("CMYK halftone", WIDTH, HEIGHT, WindowOptions::default())?;
    window.set_target_fps(30);

    let mut enabled = [true; 4];
    let mut spacing = 8.0;
    let mut show_source = false;
    let mut pixels = vec![0u32; WIDTH * HEIGHT];
    let mut dirty = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for key in window.get_keys_pressed(KeyRepeat::Yes) {
            match key {
                Key::Key1 | Key::Key2 | Key::Key3 | Key::Key4 => {
                    let index = key as usize - Key::Key1 as usize;
                    enabled[index] = !enabled[index];
                }
                Key::Equal | Key::NumPadPlus => spacing = f64::min(spacing + 1.0, 40.0),
                Key::Minus | Key::NumPadMinus => spacing = f64::max(spacing - 1.0, 3.0),
                Key::Space => show_source = !show_source,
                _ => continue,
            }
            dirty = true;
        }

        if dirty {
            if show_source {
                render_source(&mut pixels);
            } else {
                render_halftone(&mut pixels, &enabled, spacing);
            }
            window.set_title(&title(&enabled, spacing, show_source));
            dirty = false;
        }

        window.update_with_buffer(&pixels, WIDTH, HEIGHT)?;
    }

    Ok(())
}

fn title(enabled: &[bool; 4], spacing: f64, show_source: bool) -> String {
    if show_source {
        return "CMYK halftone: source image (Space: halftone)".to_string();
    }

    let inks: String = INKS
        .iter()
        .zip(enabled)
        .map(|(ink, &on)| if on { ink.name } else { "·" })
        .collect();
    format!("CMYK halftone: inks {inks}, spacing {spacing} (1-4, +/-, Space, Esc)")
}

/// The image to print: hues from left to right, fading to white at the top
/// and to black at the bottom.
fn source(x: f64, y: f64) -> [f32; 3] {
    let hue = (x / WIDTH as f64 * 6.0) as f32;
    let t = (y / HEIGHT as f64) as f32;
    let saturation = (2.0 * t).min(1.0);
    let value = (2.0 - 2.0 * t).min(1.0);

    // HSV to RGB.
    let channel = |n: f32| {
        let k = (n + hue) % 6.0;
        let f = (k.min(4.0 - k)).clamp(0.0, 1.0);
        value * (1.0 - saturation * f)
    };
    [channel(5.0), channel(3.0), channel(1.0)]
}

/// Naive RGB to CMYK separation with full black replacement.
fn separate([r, g, b]: [f32; 3]) -> Cmyk {
    let k = 1.0 - r.max(g).max(b);
    if k >= 1.0 {
        return Cmyk {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 1.0,
        };
    }
    Cmyk {
        c: (1.0 - r - k) / (1.0 - k),
        m: (1.0 - g - k) / (1.0 - k),
        y: (1.0 - b - k) / (1.0 - k),
        k,
    }
}

fn render_source(pixels: &mut [u32]) {
    for (i, pixel) in pixels.iter_mut().enumerate() {
        let (x, y) = ((i % WIDTH) as f64 + 0.5, (i / WIDTH) as f64 + 0.5);
        *pixel = to_u32(source(x, y));
    }
}

fn render_halftone(pixels: &mut [u32], enabled: &[bool; 4], spacing: f64) {
    // Start with white paper.
    let mut paper = vec![[1.0f32; 3]; WIDTH * HEIGHT];

    for (ink, _) in INKS.iter().zip(enabled).filter(|(_, &on)| on) {
        let grid = Grid::new(WIDTH as f64, HEIGHT as f64)
            .spacing(spacing, spacing)
            .angle(Angle::from_degrees(ink.angle));

        for GridCoord { x, y } in grid {
            // The dot covers the same share of its grid cell as the ink amount.
            let amount = (ink.amount)(separate(source(x, y)));
            let radius = spacing * (amount as f64 / std::f64::consts::PI).sqrt();
            stamp(&mut paper, x, y, radius, ink.color);
        }
    }

    for (pixel, rgb) in pixels.iter_mut().zip(&paper) {
        *pixel = to_u32(*rgb);
    }
}

/// Prints an anti-aliased dot: the ink multiplies with what is already on the paper.
fn stamp(paper: &mut [[f32; 3]], cx: f64, cy: f64, radius: f64, ink: [f32; 3]) {
    if radius <= 0.0 {
        return;
    }

    let x_range = (cx - radius - 1.0).floor().max(0.0) as usize
        ..=((cx + radius + 1.0) as usize).min(WIDTH - 1);
    for py in (cy - radius - 1.0).floor().max(0.0) as usize
        ..=((cy + radius + 1.0) as usize).min(HEIGHT - 1)
    {
        for px in x_range.clone() {
            let (dx, dy) = (px as f64 + 0.5 - cx, py as f64 + 0.5 - cy);
            let coverage = (radius - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0) as f32;
            if coverage > 0.0 {
                let pixel = &mut paper[py * WIDTH + px];
                for (channel, ink) in pixel.iter_mut().zip(ink) {
                    *channel *= 1.0 - coverage * (1.0 - ink);
                }
            }
        }
    }
}

/// Converts linear 0..1 RGB to minifb's `0RGB` pixel format.
fn to_u32([r, g, b]: [f32; 3]) -> u32 {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    (byte(r) << 16) | (byte(g) << 8) | byte(b)
}
