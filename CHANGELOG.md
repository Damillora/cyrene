# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]

### Changed
- Add lockfile switch to toggle disabling major version checks when upgrading.

## [0.2.3] - 2025-10-01

### Fixed
- Prevent `cyrene` from sacrificing herself to the Remembrance if `CYRENE_INSTALL_DIR` is unset and `cyrene` is trying to link binaries of itself.

## [0.2.2] - 2025-10-01

### Fixed
- Stop assuming files in `CYRENE_INSTALL_DIR` are symlinks.

## [0.2.1] - 2025-10-01

### Added
- `cyrene` now supports the `CYRENE_INSTALL_DIR` environment variable, to facillitate managing `cyrene` with `cyrene` itself.
- Preliminary support for generating `cyrene`'s environment using `cyrene env`. For now, this only generates `cyrene`'s default configuration, but there are plans to also insert environment variables needed by `cyrene` plugins.

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

[unreleased]: https://github.com/Damillora/cyrene/compare/v0.2.3...HEAD
[0.2.2]: https://github.com/Damillora/cyrene/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/Damillora/cyrene/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/Damillora/cyrene/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Damillora/cyrene/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Damillora/cyrene/releases/tag/v0.1.0
