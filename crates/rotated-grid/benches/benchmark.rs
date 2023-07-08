use criterion::{criterion_group, criterion_main, Criterion};
use rotated_grid::{Angle, GridPositionIterator};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Grid 16×16 at 0°", |b| {
        b.iter(|| {
            const WIDTH: f64 = 16.0;
            const HEIGHT: f64 = 10.0;
            const ANGLE: f64 = 0.0;

            let grid = GridPositionIterator::new(
                WIDTH as _,
                HEIGHT as _,
                7.0,
                7.0,
                0.0,
                0.0,
                Angle::<f64>::from_degrees(ANGLE),
            );

            let mut count = 0;
            for _ in grid.into_iter() {
                count += 1;
            }

            count
        })
    });

    c.bench_function("Grid 10240×128 at 0°", |b| {
        b.iter(|| {
            const WIDTH: f64 = 10240.0;
            const HEIGHT: f64 = 128.0;
            const ANGLE: f64 = 0.0;

            let grid = GridPositionIterator::new(
                WIDTH as _,
                HEIGHT as _,
                7.0,
                7.0,
                0.0,
                0.0,
                Angle::<f64>::from_degrees(ANGLE),
            );

            let mut count = 0;
            for _ in grid.into_iter() {
                count += 1;
            }

            count
        })
    });

    c.bench_function("Grid 10240×128 at 45°", |b| {
        b.iter(|| {
            const WIDTH: f64 = 10240.0;
            const HEIGHT: f64 = 128.0;
            const ANGLE: f64 = 45.0;

            let grid = GridPositionIterator::new(
                WIDTH as _,
                HEIGHT as _,
                7.0,
                7.0,
                0.0,
                0.0,
                Angle::<f64>::from_degrees(ANGLE),
            );

            let mut count = 0;
            for _ in grid.into_iter() {
                count += 1;
            }

            count
        })
    });

    c.bench_function("Grid 10240×128 at 15°", |b| {
        b.iter(|| {
            const WIDTH: f64 = 10240.0;
            const HEIGHT: f64 = 128.0;
            const ANGLE: f64 = 15.0;

            let grid = GridPositionIterator::new(
                WIDTH as _,
                HEIGHT as _,
                7.0,
                7.0,
                0.0,
                0.0,
                Angle::<f64>::from_degrees(ANGLE),
            );

            let mut count = 0;
            for _ in grid.into_iter() {
                count += 1;
            }

            count
        })
    });

    c.bench_function("Grid 10240×128 at 75°", |b| {
        b.iter(|| {
            const WIDTH: f64 = 10240.0;
            const HEIGHT: f64 = 128.0;
            const ANGLE: f64 = 75.0;

            let grid = GridPositionIterator::new(
                WIDTH as _,
                HEIGHT as _,
                7.0,
                7.0,
                0.0,
                0.0,
                Angle::<f64>::from_degrees(ANGLE),
            );

            let mut count = 0;
            for _ in grid.into_iter() {
                count += 1;
            }

            count
        })
    });

    c.bench_function("Grid 10240×10240 at 45°", |b| {
        b.iter(|| {
            const WIDTH: f64 = 10240.0;
            const HEIGHT: f64 = 10240.0;
            const ANGLE: f64 = 45.0;

            let grid = GridPositionIterator::new(
                WIDTH as _,
                HEIGHT as _,
                7.0,
                7.0,
                0.0,
                0.0,
                Angle::<f64>::from_degrees(ANGLE),
            );

            let mut count = 0;
            for _ in grid.into_iter() {
                count += 1;
            }

            count
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
