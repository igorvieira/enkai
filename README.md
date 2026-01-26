# Murasaki
> A TUI tool for resolving git conflicts during merge or rebase operations.


[![CI](https://github.com/igorvieira/murasaki_rs/workflows/CI/badge.svg)](https://github.com/igorvieira/murasaki_rs/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/igorvieira/murasaki_rs/branch/main/graph/badge.svg)](https://codecov.io/gh/igorvieira/murasaki_rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)


<img src="https://res.cloudinary.com/dje6m1lab/image/upload/v1769397667/ChatGPT_Image_Jan_26_2026_12_20_52_AM_g6zfhy.png" width="700" height="430"  />

## Installation

```bash
cargo install --path .
```

## Usage

When you have git conflicts:

```bash
saki
```

## Commands

### File List
- `j/k` or `↑/↓` - Navigate
- `Tab` - Switch to code view
- `q` - Quit

### Code View
- `j/k` or `↑/↓` - Scroll line by line
- `Ctrl+d/Ctrl+u` - Scroll half page (fast)
- `n/p` - Next/previous conflict
- `c` - Accept Current (HEAD) for current conflict
- `i` - Accept Incoming for current conflict
- `b` - Accept Both for current conflict
- `u` - Undo resolution of current conflict
- `s` - Save file (after resolving all conflicts)
- `Tab` - Go back to file list
- `q` - Quit

### After Resolving (Rebase)
- `c` - Continue rebase
- `a` - Abort rebase
- `s` - Skip commit

## How It Works

1. Detects git conflicts in the repository
2. Shows list of files with conflicts
3. Use `Tab` to go to code view
4. Use `j/k` to scroll and see the entire file
5. Use `n/p` to navigate between conflicts
6. For each conflict, choose: `c` (Current), `i` (Incoming), or `b` (Both)
7. When all conflicts are resolved, press `s` to save
8. Go to the next file or, if it's a rebase, choose continue/abort/skip

## Features

- Split-pane interface with file list and code view
- Syntax highlighting for conflict regions
- Visual indicators for resolved/unresolved conflicts
- Color-coded conflict backgrounds:
  - Current (HEAD): Blue background
  - Incoming: Red background
  - Both: Purple background
- Atomic file writes to prevent data corruption
- Terminal cleanup on panic for safety
- Input validation to prevent path traversal

## Structure

```
src/
├── domain/     # Data models
├── app/        # Application state
├── git/        # Git integration
└── tui/        # User interface
```

## Development

```bash
# Build
cargo build --release

# Tests
cargo test

# Lint
cargo clippy
```

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history and release notes
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - Architecture and design decisions
- [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) - Contributing guidelines
- [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) - Implementation details and fixes
- [docs/TESTING.md](docs/TESTING.md) - Testing guide and coverage

## Version

Current version: 0.1.1

See [CHANGELOG.md](CHANGELOG.md) for version history.

## License

MIT

## Contributing

See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines on how to contribute to this project.
