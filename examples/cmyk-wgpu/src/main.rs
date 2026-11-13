//! CMYK halftone rendered entirely on the GPU.
//!
//! The fragment shader uses `rotated_grid::WGSL` to find the grid points around each pixel.
//! The CPU only uploads the grid parameters, never the grid points themselves.
//!
//! Keys: `1`–`4` toggle C/M/Y/K, `+`/`-` change the dot spacing, `Space` shows the source
//! image, `Esc` quits.

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle, RawDisplayHandle};
use rotated_grid::{Angle, Grid};
use std::error::Error;

const WIDTH: usize = 800;
const HEIGHT: usize = 500;

/// Screen angles for cyan, magenta, yellow and black.
const INK_ANGLES: [f64; 4] = [15.0, 75.0, 0.0, 45.0];
const INK_NAMES: [&str; 4] = ["C", "M", "Y", "K"];
const SHOW_SOURCE: u32 = 1 << 4;

fn main() -> Result<(), Box<dyn Error>> {
    let mut window = Window::new(
        "CMYK halftone (GPU)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;
    window.set_target_fps(60);

    let display = Display(window.display_handle()?.as_raw());
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
        Box::new(display),
    ));
    // SAFETY: The window outlives the surface; both live until the end of `main`,
    // and the surface is declared later, so it is dropped first.
    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window)?)?
    };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;

    let mut config = surface
        .get_default_config(&adapter, WIDTH as u32, HEIGHT as u32)
        .ok_or("surface not supported by the adapter")?;
    // The shader writes the same values as the CPU demo, so skip the sRGB conversion.
    config.format = config.format.remove_srgb_suffix();
    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("halftone"),
        source: wgpu::ShaderSource::Wgsl(
            format!("{}\n{}", rotated_grid::WGSL, include_str!("halftone.wgsl")).into(),
        ),
    });

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("params"),
        size: PARAMS_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("params"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("params"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: params_buffer.as_entire_binding(),
        }],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("halftone"),
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("halftone"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            }),
        ),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(config.format.into())],
        }),
        multiview_mask: None,
        cache: None,
    });

    let mut enabled = [true; 4];
    let mut spacing = 8.0;
    let mut show_source = false;
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
            queue.write_buffer(&params_buffer, 0, &params(&enabled, spacing, show_source));
            window.set_title(&title(&enabled, spacing, show_source));
            dirty = false;
        }

        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                surface.configure(&device, &config);
                window.update();
                continue;
            }
            _ => {
                window.update();
                continue;
            }
        };

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("halftone"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
        queue.present(frame);

        // Processes window events; the image itself is presented by wgpu.
        window.update();
    }

    Ok(())
}

/// The window's display connection, in the owned, thread-safe form wgpu's instance requires.
#[derive(Debug)]
struct Display(RawDisplayHandle);

// SAFETY: wgpu only uses the handle to talk to the display server. The display connection
// belongs to the window, which is created first and dropped after the instance.
unsafe impl Send for Display {}
unsafe impl Sync for Display {}

impl HasDisplayHandle for Display {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // SAFETY: The handle stays valid while the window lives; see above.
        Ok(unsafe { DisplayHandle::borrow_raw(self.0) })
    }
}

/// Size of the `Params` uniform: four grids of eight values, then flags and padding.
const PARAMS_SIZE: u64 = (4 * 8 + 4) * 4;

/// Packs the `Params` uniform of `halftone.wgsl`.
fn params(enabled: &[bool; 4], spacing: f64, show_source: bool) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(PARAMS_SIZE as usize);
    for angle in INK_ANGLES {
        let grid = Grid::new(WIDTH as f64, HEIGHT as f64)
            .spacing(spacing, spacing)
            .angle(Angle::from_degrees(angle));
        for value in grid.to_uniform() {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
    }

    let mut flags = if show_source { SHOW_SOURCE } else { 0 };
    for (bit, &on) in enabled.iter().enumerate() {
        flags |= u32::from(on) << bit;
    }
    for value in [flags, 0, 0, 0] {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes
}

fn title(enabled: &[bool; 4], spacing: f64, show_source: bool) -> String {
    if show_source {
        return "CMYK halftone (GPU): source image (Space: halftone)".to_string();
    }

    let inks: String = INK_NAMES
        .iter()
        .zip(enabled)
        .map(|(name, &on)| if on { *name } else { "·" })
        .collect();
    format!("CMYK halftone (GPU): inks {inks}, spacing {spacing} (1-4, +/-, Space, Esc)")
}

#[cfg(test)]
mod tests {
    use rotated_grid::{Angle, Grid, GridCoord};
    use wgpu::util::DeviceExt;

    /// Runs `rotated_grid_nearest` from `rotated_grid::WGSL` on the GPU for each input point.
    /// Returns `None` if no GPU adapter (not even a software one) is available.
    fn gpu_nearest(grid: &Grid, points: &[[f32; 2]]) -> Option<Vec<[f32; 2]>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}
                    @group(0) @binding(0) var<uniform> grid: RotatedGrid;
                    @group(0) @binding(1) var<storage, read> input: array<vec2<f32>>;
                    @group(0) @binding(2) var<storage, read_write> output: array<vec2<f32>>;

                    @compute @workgroup_size(64)
                    fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
                        if id.x < arrayLength(&input) {{
                            output[id.x] = rotated_grid_nearest(grid, input[id.x]);
                        }}
                    }}",
                    rotated_grid::WGSL
                )
                .into(),
            ),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let bytes = |values: &mut dyn Iterator<Item = f32>| -> Vec<u8> {
            values.flat_map(f32::to_ne_bytes).collect()
        };
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &bytes(&mut grid.to_uniform().into_iter()),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let input = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &bytes(&mut points.iter().flatten().copied()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let size = input.size();
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output.as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((points.len() as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, size);
        queue.submit([encoder.finish()]);

        readback.map_async(wgpu::MapMode::Read, .., |result| result.unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data = readback.get_mapped_range(..).unwrap();
        Some(
            data.as_chunks::<4>()
                .0
                .iter()
                .map(|bytes| f32::from_ne_bytes(*bytes))
                .collect::<Vec<_>>()
                .as_chunks::<2>()
                .0
                .to_vec(),
        )
    }

    /// The shader's lattice must be the iterator's lattice: each grid point the CPU yields is
    /// its own nearest grid point on the GPU, and points nudged off it snap back to it.
    #[test]
    fn shader_lattice_matches_iterator() {
        let grids = [
            Grid::new(800.0, 500.0)
                .spacing(8.0, 8.0)
                .angle(Angle::from_degrees(15.0)),
            Grid::new(272.11, 6.73)
                .spacing(5.22, 2.21)
                .offset(0.75, 1.37)
                .angle(Angle::from_degrees(88.79)),
            Grid::new(600.0, 400.0)
                .spacing(10.0, 20.0)
                .offset(3.0, 5.0)
                .angle(Angle::from_degrees(-135.0)),
        ];

        for grid in grids {
            let expected: Vec<[f32; 2]> = grid
                .into_iter()
                .map(|GridCoord { x, y }| [x as f32, y as f32])
                .collect();
            // Nudge each point by a quarter of the smaller spacing in a varying direction.
            let nudge = grid.dx.min(grid.dy) as f32 * 0.25;
            let input: Vec<[f32; 2]> = expected
                .iter()
                .enumerate()
                .map(|(i, [x, y])| {
                    let a = i as f32 * 2.4;
                    [x + nudge * a.cos(), y + nudge * a.sin()]
                })
                .collect();

            let Some(actual) = gpu_nearest(&grid, &input) else {
                eprintln!("skipping: no GPU adapter available");
                return;
            };

            for (i, (e, a)) in expected.iter().zip(&actual).enumerate() {
                let error = ((e[0] - a[0]).powi(2) + (e[1] - a[1]).powi(2)).sqrt();
                assert!(
                    error < 1e-2,
                    "{grid:?}: point {i} expected {e:?}, shader gave {a:?}"
                );
            }
        }
    }
}
