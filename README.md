# murasaki_rs
> A TUI tool for resolving git conflicts during merge or rebase operations.


[![CI](https://github.com/igorvieira/murasaki_rs/workflows/CI/badge.svg)](https://github.com/igorvieira/murasaki_rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)


<img src="https://res.cloudinary.com/dje6m1lab/image/upload/v1769398314/ChatGPT_Image_Jan_26_2026_12_31_41_AM_f1xewn.png" width="700" height="430"  />

<img width="1702" height="462" alt="image" src="https://github.com/user-attachments/assets/50f21ee6-8029-479d-adeb-9f4ad740831f" />


## Installation


### Manual Installation

```bash
git clone https://github.com/igorvieira/murasaki_rs.git ~/.config/murasaki_rs
cd ~/.config/murasaki_rs
cargo install --path .
```

This will:
- Clone the repository to `~/.config/murasaki_rs`
- Build and install using `cargo install --path .`
- Install the `saki` binary to `~/.cargo/bin/`


## Usage

### When You Have Git Conflicts

```bash
saki
```

### When There Are No Conflicts

If you run `saki` without any active merge/rebase conflicts, it will show an interactive status view with:

**Layout:**
- **Left Panel (25%)**: Menu with options
  - View Changes
  - Quit
- **Right Panel (75%)**: Dynamic content based on menu selection
  - Default: ANSI art banner (murasaki in purple, centered)
  - View Changes: File list organized by status (Staged, Both, Unstaged) with inline operations

## Commands

### Status View (No Conflicts)

**When on Menu (Banner):**
- `j/k` or `↑/↓` - Navigate menu
- `Enter` - Select menu option
- `q` - Quit

**When in View Changes:**
- `j/k` or `↑/↓` - Navigate files
- `a` - Stage selected file (git add)
- `s` - Unstage selected file (git restore --staged)
- `r` - Restore selected file to last committed state (git restore)
- `Esc` - Return to menu
- `q` - Quit

**File Status Display:**
- Files are shown with two-character status: `[XY]`
  - First character (X): Staged status (index)
  - Second character (Y): Unstaged status (working directory)
  - Examples: `[M ]` = staged modification, `[ M]` = unstaged modification, `[MM]` = both

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

### Conflict Resolution
- Split-pane interface with file list and code view
- Syntax highlighting for conflict regions
- Visual indicators for resolved/unresolved conflicts
- Color-coded conflict backgrounds:
  - Current (HEAD): Blue background
  - Incoming: Red background
  - Both: Purple background

### Status View (New!)
- Split layout: 25% left menu, 75% right content panel
- ANSI art banner (murasaki in purple, centered, borderless)
- Always-visible menu on the left
- Dynamic right panel that changes based on menu selection
- Interactive git status display with **staged vs unstaged** separation
- Files organized into three sections:
  - **Staged Changes** (green): Ready to commit
  - **Staged + Unstaged Changes** (yellow): Partially staged files
  - **Unstaged Changes** (red): Working directory modifications
- Two-character status display (e.g., `[M ]`, `[ M]`, `[MM]`)
- Restore files functionality (both staged and unstaged)
- Clean, VS Code/Cursor-like interface

### Safety Features
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
