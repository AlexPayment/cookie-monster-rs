# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate

## [2.0.0] - 2026-07-20

### Added

- AnimationKind enum to track the state transitions between animations.
- `Shimmer` and `MultiColorFrontToBack` animations.
- Default render method.

### Changed

- Avoid a copy of the vertical slices on the stack.
- Stop initializing all the animations when the application starts. Instead, initialize them only when they are needed.
- Simplify the signature of the render methods to accept a broader range of SmartLedsWrite trait implementations.
- Split the LEDS writes into two. The first section has 384 LEDs, and the second section has 576. Each section has a
  dedicated SPI and the writes are done in parallel. Before this change, it took about 28 milliseconds to write. Now
  it takes about 16 milliseconds, with the second section contributing to the higher duration due to its larger LED
  count.
- Move the following animations to the end of the rotation:
  - `MultiColorSolid`
  - `MultiColorSolidRandom`
  - `UniColorSolid`

### Removed

- Duplicated constants for the potentiometer values.
- Reference to the LedData in every animation struct. Instead, pass to the methods a reference or a mutable reference to
  the LedData. This removes the need of RefCell and lifetimes.
- Remove jitter handling when reading the potentiometer values. It caused the starting value to be incorrect, and it was
  bringing little value.

## [1.1.0] - 2026-07-04

### Changed

- Handle jitter and errors when reading the potentiometer values.
- Update the CI workflow:
  - Set the cache path to match the example provided on their repo.
  - Split the caches by target and step.
- Update the CI workflow to use:
  - `actions/cache` version `6.*`.
  - `actions/checkout` version `7.*`.
  - `esp-rs/xtensa-toolchain` version `1.7.*`.
- Update `defmt` to version `1.1.1`.
- Update `defmt-rtt` to version `1.3.0`.
- Update `embassy-executor` to version `0.10.0`.
- Update `embassy-nrf` to version `0.7.0`.
- Update `embassy-sync` to version `0.8.0`.
- Update `embassy-time` to version `0.5.1`.
- Update `esp-backtrace` to version `0.19.0`.
- Update `esp-bootloader-esp-idf` to version `0.5.0`.
- Update `esp-hal` to version `1.1.1`.
- Update `esp-println` to version `0.17.0`.
- Update `rand` to version `0.10.2`.
- Update `smart-leds-trait` to version `0.3.2`.
- Update all transitive dependencies.

### Fixed

- Address some clippy lints.
- Update the wiring diagram for the Micro:bit v2.

### Removed

- The cache from the CI workflow `fmt` job.
- Rust version for `qinled-dig-quad`.

## [1.0.1] - 2025-07-28

### Changed

- Update `embassy-nrf` to version 0.5.0.
- Update `esp-backtrace` to version 0.17.0.
- Update `esp-hal` to version 1.0.0-rc.0.
- Update `esp-hal-embassy` to version 0.9.0.
- Update `esp-println` to version 0.15.0.
- Update `rand` to version 0.9.2.
- Set rust version for `qinled-dig-quad` to `1.88.0`.

## [1.0.0] - 2025-06-20

### Added

- `cookie-monster-qinled-dig-quad` crate for the QinLED Dig Quad board.

### Changed

- Convert the project from a single crate to a workspace with multiple crates.
- Update the CI workflow to cache cargo dependencies and run fmt and clippy checks.
- Update README with a section on QuinLED Dig Quad.
- Move `Settings` to the `common` crate.
- `Animation` is now an enum in the `common` crate.
- Migrate the micro:bit v2 implementation to Embassy.
- Address some clippy lints.
- Update `defmt` to version 1.0.1.
- Update `rand` to version 0.9.1.
