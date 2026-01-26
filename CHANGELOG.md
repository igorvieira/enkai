# Changelog

All notable changes to Murasaki (saki) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI/CD workflows for automated testing
- Test coverage reporting with codecov
- Security audit workflow with cargo-audit
- Multi-platform release workflow (Linux, macOS, Windows)
- Comprehensive unit tests for domain layer (16 new tests)
- Testing guide documentation (docs/TESTING.md)
- CI status badges in README

### Changed
- Renamed binary from `murasaki_rs` to `saki` for easier command-line usage
- Updated all documentation to English
- Improved conflict color scheme:
  - Current (HEAD): Dark blue background
  - Incoming: Dark red background
  - Both selected: Dark purple background
- Renamed project from "enkai" to "murasaki_rs"
- Changed color struct from `EnkaiColors` to `MurasakiColors`
- Increased test coverage from 5 to 21 tests

### Removed
- Removed emojis from documentation and scripts for better accessibility

### Fixed
- Fixed clippy warning in domain tests (clone on Copy type)

### CI/CD
- Tests run on Ubuntu, macOS, and Windows
- Code quality checks (clippy, rustfmt)
- Automated release builds for all platforms
- Coverage tracking and reporting

## [0.1.1] - 2026-01-01

### Fixed
- **CRITICAL**: Fixed unsafe array indexing in `parser.rs` that could cause panics
- **CRITICAL**: Fixed unsafe array indexing in `applier.rs` that could cause panics
- **CRITICAL**: Implemented atomic file writes to prevent data corruption
- **HIGH**: Added panic guard for terminal cleanup to prevent broken terminal state
- **MEDIUM**: Added input validation for file paths to prevent path traversal
- Fixed clippy warnings for collapsible if statements

### Changed
- Replaced direct array accesses with safe `.get()` methods
- Improved error messages with better context
- Preserved original trailing newline behavior in files
- Temp files now use `.{filename}.murasaki_rs.tmp` naming convention

### Security
- Added file path validation to prevent path traversal attacks
- Ensured files are within repository before processing
- Added checks to reject directories and validate file existence

### Testing
- All 5 existing tests passing
- Clean clippy output with no warnings
- Successful release build with no warnings

## [0.1.0] - 2026-01-01

### Added
- Initial release of Murasaki TUI git conflict resolution tool
- Support for merge and rebase conflict resolution
- Split-pane interface with file list and code view
- Syntax highlighting for conflict regions
- Keyboard navigation with vim-style keys (j/k) and arrow keys
- Three resolution strategies: Current (HEAD), Incoming, Both
- Undo functionality for conflict resolutions
- File save with conflict validation
- Rebase action menu (continue/abort/skip)
- Focus indicators for active pane
- Status indicators for resolved/unresolved conflicts

### Features
- Domain layer with pure business logic
- Application state management
- Git integration using git2 library
- Terminal UI using ratatui and crossterm
- Conflict parsing from git markers
- Resolution application with validation

### Documentation
- README with usage instructions
- ARCHITECTURE.md with design decisions
- CONTRIBUTING.md with development guidelines
- Code review summary and fixes documentation

## Future Enhancements

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for planned features:
- Scrolling support for large files
- Custom merge strategies
- Three-way merge view
- Diff visualization with syntax highlighting
- Configuration file for keybindings and colors
- Undo/redo history navigation

---

## Release Notes

### Version 0.1.1 - Stability & Safety Release
Focus: Critical bug fixes for production readiness
- Eliminated all panic risks from array operations
- Implemented atomic file writes
- Added comprehensive input validation
- Terminal cleanup on panic

### Version 0.1.0 - Initial Release
First public release with core functionality for git conflict resolution via TUI.

---

[Unreleased]: https://github.com/igorvieira/murasaki_rs/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/igorvieira/murasaki_rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/igorvieira/murasaki_rs/releases/tag/v0.1.0
