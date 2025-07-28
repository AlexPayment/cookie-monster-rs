# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate

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
