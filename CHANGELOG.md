# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate

### Added

- `cookie-monster-qinled-dig-quad` crate for the QinLED Dig Quad board.

### Changed

- Convert the project from a single crate to a workspace with multiple crates.
- Update the CI workflow to cache cargo dependencies and run fmt and clippy checks.
- Update README with a section on QuinLED Dig Quad.
- Move `Settings` to the `common` crate.
- `Animation` is now an enum in the `common` crate.
- Address some clippy lints.
- Update `defmt` to version 1.0.1.
- Update `rand` to version 0.9.1.
