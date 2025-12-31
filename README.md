# Enkai

A beautiful TUI (Terminal User Interface) tool for handling git conflicts during merge or rebase operations.

## Features

- 🎯 **Intuitive Interface**: Navigate conflicts with vim-style keybindings or arrow keys
- 📝 **Visual Conflict Resolution**: See current and incoming changes side-by-side
- 🔄 **Rebase Integration**: Seamlessly continue, skip, or abort rebases after resolving conflicts
- ⚡ **Fast & Efficient**: Built in Rust with ratatui for a smooth experience
- 🎨 **Clear Visual Feedback**: Color-coded status indicators and progress tracking

## Installation

### Quick Install (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/enkai/main/install.sh | sh
```

### From Source

Requires Rust toolchain (1.70+):

```bash
git clone https://github.com/YOUR_USERNAME/enkai.git
cd enkai
cargo install --path .
```

### Manual Binary Download

Download the latest release for your platform from the [releases page](https://github.com/YOUR_USERNAME/enkai/releases).

## Usage

When you have git conflicts during a merge or rebase:

```bash
# Automatically detect and show all conflicted files
enkai

# Or specify specific files
enkai src/main.rs src/lib.rs
```

## Keybindings

### File List View

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Open selected file |
| `q` | Quit |

### Conflict Resolution View

| Key | Action |
|-----|--------|
| `c` | Accept **Current** (HEAD) changes |
| `i` | Accept **Incoming** changes |
| `b` | Accept **Both** changes |
| `j` / `↓` | Next conflict |
| `k` / `↑` | Previous conflict |
| `s` | **Save** file (only when all conflicts resolved) |
| `Esc` | Back to file list |
| `q` | Quit |

### Rebase Actions View

(Appears after resolving all conflicts during a rebase)

| Key | Action |
|-----|--------|
| `c` | **Continue** rebase (`git rebase --continue`) |
| `a` | **Abort** rebase (`git rebase --abort`) |
| `s` | **Skip** current commit (`git rebase --skip`) |
| `q` / `Esc` | Exit without action |

## Workflow Example

1. Start a merge or rebase that results in conflicts:
   ```bash
   git merge feature-branch
   # or
   git rebase main
   ```

2. Launch enkai:
   ```bash
   enkai
   ```

3. Navigate through files and resolve conflicts:
   - Use `j`/`k` or arrow keys to select a file
   - Press `Enter` to open it
   - For each conflict, choose `c` (current), `i` (incoming), or `b` (both)
   - Navigate between conflicts with `j`/`k`
   - Press `s` to save when done

4. If it's a rebase, choose what to do next:
   - Press `c` to continue the rebase
   - Or handle it manually later

## How It Works

Enkai parses git conflict markers in your files:

```
<<<<<<< HEAD
current changes
=======
incoming changes
>>>>>>> branch-name
```

And presents them in a clean interface where you can:
- See both versions side-by-side
- Choose which version to keep
- Track your progress across multiple conflicts and files

## Development

### Building

```bash
cargo build --release
```

The binary will be at `target/release/enkai`.

### Running Tests

```bash
cargo test
```

### Project Structure

```
enkai/
├── src/
│   ├── app/           # Application state management
│   ├── domain/        # Core domain models
│   ├── git/           # Git integration
│   └── tui/           # Terminal UI
│       └── views/     # UI components
├── Cargo.toml
└── README.md
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with:
- [ratatui](https://ratatui.rs/) - Terminal UI framework
- [git2-rs](https://github.com/rust-lang/git2-rs) - Git bindings for Rust
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation

---

Made with ❤️ by Igor Vieira
