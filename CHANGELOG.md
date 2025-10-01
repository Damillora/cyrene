# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]

## [0.2.0] - 2025-10-01

### Added
- BREAKING Change: Install, upgrade, and uninstall multiple apps at once with `<plugin>@<version>` syntax.
- `cyrene list`: Show linked versions as well.
- Show progress of downloading sources.
- Pretty CLI output

### Changed
- Rework version handling
- A few refactorings
- `cyrene upgrade`: Upgrade all apps when no apps are specified.
- Remove app from lockfile if candidates for linking are no longer available.

## [0.1.0] - 2025-10-01

### Added
- Install and upgrade multiple versions of tools
- Symlink-based version management
- Extensible with scripts written in [Rune](https://rune-rs.github.io/)
- Synchronize tool versions with `cyrene.toml` lockfiles

[unreleased]: https://github.com/Damillora/cyrene/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Damillora/cyrene/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Damillora/cyrene/releases/tag/v0.1.0
