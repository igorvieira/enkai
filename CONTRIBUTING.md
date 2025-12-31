# Contributing to Enkai

First off, thank you for considering contributing to Enkai! 🎉

## Code of Conduct

Be respectful, inclusive, and collaborative. We're all here to build something useful together.

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

**When filing a bug report, include**:
- Your OS and version
- Steps to reproduce
- Expected behavior
- Actual behavior
- Error messages or screenshots if applicable

### Suggesting Features

We welcome feature suggestions! Please:
- Check if the feature has already been suggested
- Provide a clear description of the feature
- Explain why it would be useful
- Include examples of how it would work

### Pull Requests

1. **Fork** the repository
2. **Create a branch** from `main`:
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make your changes**
4. **Test** your changes:
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
5. **Commit** with a clear message:
   ```bash
   git commit -m "Add amazing feature"
   ```
6. **Push** to your fork:
   ```bash
   git push origin feature/amazing-feature
   ```
7. **Open a Pull Request**

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Git

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/enkai.git
cd enkai

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

### Testing Your Changes

To test enkai with real git conflicts:

```bash
# Create a test repository
mkdir test-repo && cd test-repo
git init

# Create initial commit
echo "line 1" > test.txt
git add test.txt
git commit -m "Initial commit"

# Create a branch and make changes
git checkout -b feature
echo "line 2 from feature" >> test.txt
git commit -am "Feature change"

# Go back to main and make conflicting changes
git checkout main
echo "line 2 from main" >> test.txt
git commit -am "Main change"

# Create a conflict
git merge feature

# Now run enkai
cargo run --manifest-path ../Cargo.toml
```

## Code Style

We follow standard Rust conventions:

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write tests for new features
- Document public APIs with doc comments

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- First line should be 50 characters or less
- Reference issues and PRs when relevant

Example:
```
Add support for three-way merge view

Implements a new view mode that shows base, current, and incoming
changes side-by-side for better conflict understanding.

Closes #42
```

## Project Structure

```
src/
├── app/           # Application state management
├── domain/        # Core domain models
├── git/           # Git integration layer
└── tui/           # Terminal UI
    └── views/     # UI components
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## Testing Guidelines

### Unit Tests

Place tests in the same file as the code they test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test code
    }
}
```

### Integration Tests

Create files in `tests/` directory:

```rust
// tests/conflict_resolution.rs
use enkai::*;

#[test]
fn test_end_to_end_resolution() {
    // Integration test code
}
```

## Documentation

- Update README.md if you change functionality
- Add doc comments to public APIs
- Update ARCHITECTURE.md for significant architectural changes

## Questions?

Feel free to:
- Open an issue with the "question" label
- Start a discussion on GitHub Discussions
- Reach out to maintainers

## Recognition

Contributors will be recognized in the README and release notes.

Thank you for contributing! 🚀
