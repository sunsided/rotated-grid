# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-11-13

### Added

- `rotated_grid::WGSL` provides WGSL functions (`rotated_grid_nearest`, `rotated_grid_neighbors`
  and more) that find the grid points around a position in a shader, using the same lattice as
  the iterator. `Grid::to_uniform` packs a grid into the matching `RotatedGrid` uniform layout.
- `Grid` describes a grid with named fields and builder methods (`Grid::new(w, h)`,
  `.spacing(dx, dy)`, `.offset(x0, y0)`, `.angle(a)`) and implements `IntoIterator`.
- Grids can now be rotated by any finite angle, including negative angles and angles
  beyond 90°. Positive angles turn the grid counter-clockwise in image coordinates.

### Changed

- **Breaking:** `GridPositionIterator::new` takes a `Grid` instead of seven positional numbers,
  which could be swapped without a compile error.
- **Breaking:** `Angle` is no longer generic; use `Angle` instead of `Angle::<f64>`.
- Iterating grid points is about 3–5× faster: the per-point `next` calls can now be inlined
  into calling crates, and row setup is kept out of the per-point path.

### Fixed

- Grid points near the rectangle's edges were skipped for small rectangles, thin rectangles
  and angles close to 90°.
- `GridPositionIterator::new` now panics on non-positive or non-finite spacings instead of
  looping forever.
- The upper bound returned by `size_hint` is now always respected.
- 90° no longer produces a different grid than angles just below it when `dx` and `dy` differ.
- `GridCoord` now orders by `y`, then `x`; previously points on the same row compared equal.

### Removed

- **Breaking:** The `inner` module (`Vector`, `Line`, `LineSegment`, `OptimalIterator`) and the
  crate-root re-export of `OptimalIterator` are no longer public. They were implementation
  details exposed for the demo application.

## [0.2.0] - 2023-07-08

### Internal

- Reworked the code such that only valid positions are enumerated, speeding up the
  computation on large rotated grids.

## [0.1.1] - 2023-07-08

### Internal

- Explicitly state the use of EUPL-1.2 in `Cargo.toml` to assist crates.io in displaying
  the correct license.

## [0.1.0] - 2023-07-02

### Added

- 🎉 Initial release.

[Unreleased]: https://github.com/sunsided/rotated-grid/compare/1.0.0...HEAD
[1.0.0]: https://github.com/sunsided/rotated-grid/releases/tag/1.0.0
[0.2.0]: https://github.com/sunsided/rotated-grid/releases/tag/0.2.0
[0.1.1]: https://github.com/sunsided/rotated-grid/releases/tag/0.1.1
[0.1.0]: https://github.com/sunsided/rotated-grid/releases/tag/0.1.0
