use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rotated_grid::{Angle, Grid};

pub fn criterion_benchmark(c: &mut Criterion) {
    let cases = [
        (16.0, 10.0, 0.0),
        (10240.0, 128.0, 0.0),
        (10240.0, 128.0, 45.0),
        (10240.0, 128.0, 15.0),
        (10240.0, 128.0, 75.0),
        (10240.0, 10240.0, 45.0),
    ];

    for (width, height, angle) in cases {
        let name = format!("Grid {width}×{height} at {angle}°");
        c.bench_function(&name, |b| {
            b.iter(|| {
                Grid::new(black_box(width), black_box(height))
                    .spacing(7.0, 7.0)
                    .angle(Angle::from_degrees(angle))
                    .into_iter()
                    .count()
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
