# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Grids can now be rotated by any finite angle, including negative angles and angles
  beyond 90°. Positive angles turn the grid counter-clockwise in image coordinates.

### Changed

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

- The internal `AngleOps` trait and the unused `Vector::rotate_with`, `AddAssign`, `SubAssign`,
  `LineSegment::direction_normalized`, `LineSegment::start` and `Line`'s `Neg`/`Mul` operators.

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

[0.2.0]: https://github.com/sunsided/rotated-grid/releases/tag/0.2.0
[0.1.1]: https://github.com/sunsided/rotated-grid/releases/tag/0.1.1
[0.1.0]: https://github.com/sunsided/rotated-grid/releases/tag/0.1.0
