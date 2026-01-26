# Testing Guide

This document describes the testing strategy, how to run tests, and how to contribute new tests to Murasaki.

## Table of Contents

- [Test Coverage](#test-coverage)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [Continuous Integration](#continuous-integration)
- [Test Organization](#test-organization)

## Test Coverage

Current test coverage statistics:

```bash
cargo test
```

**Test Count:** 21 tests
- Domain layer: 16 tests
- Git layer: 5 tests
- Application layer: 0 tests (planned)
- TUI layer: 0 tests (planned)

### Coverage by Module

| Module | Tests | Coverage |
|--------|-------|----------|
| `domain::resolution` | 3 tests | Good |
| `domain::conflict` | 13 tests | Good |
| `git::parser` | 3 tests | Partial |
| `git::applier` | 1 test | Partial |
| `git::detector` | 1 test | Partial |
| `app::state` | 0 tests | None |
| `tui::*` | 0 tests | None |

### Target Coverage

- **Phase 1 (Current):** Domain layer - 100%
- **Phase 2:** Git layer - 80%
- **Phase 3:** Application layer - 70%
- **Phase 4:** Integration tests - Key workflows

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Specific Test

```bash
cargo test test_name
```

### Run Tests for Specific Module

```bash
cargo test domain::
cargo test git::
```

### Run Tests in Release Mode

```bash
cargo test --release
```

### Generate Coverage Report

Install tarpaulin:
```bash
cargo install cargo-tarpaulin
```

Generate coverage:
```bash
cargo tarpaulin --verbose --all-features --workspace --timeout 120
```

Generate HTML report:
```bash
cargo tarpaulin --out Html
```

## Writing Tests

### Unit Tests

Unit tests are placed in the same file as the code they test, in a `tests` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let result = my_function();
        assert_eq!(result, expected);
    }
}
```

### Test Naming Convention

- **Function tests:** `test_function_name_scenario`
- **Method tests:** `test_struct_method_scenario`
- **Edge cases:** `test_function_edge_case_description`
- **Error cases:** `test_function_error_condition`

Examples:
```rust
#[test]
fn test_conflict_hunk_creation() { }

#[test]
fn test_conflict_hunk_resolve_current() { }

#[test]
fn test_conflicted_file_set_resolution_out_of_bounds() { }
```

### Domain Layer Tests

Domain layer tests should be **pure** and **isolated**:

```rust
#[test]
fn test_conflict_hunk_resolve_both() {
    let hunk = ConflictHunk::new(
        "current".to_string(),
        "incoming".to_string(),
        0,
        5,
    );
    assert_eq!(hunk.resolve(Resolution::Both), "current\nincoming");
}
```

### Git Layer Tests

Git layer tests use temporary files:

```rust
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_apply_resolutions() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "content").unwrap();

    // Test logic...
}
```

### Integration Tests

Integration tests go in the `tests/` directory:

```bash
tests/
├── integration_test.rs
└── helpers/
    └── mod.rs
```

Example:
```rust
use murasaki_rs::*;

#[test]
fn test_end_to_end_conflict_resolution() {
    // Create test repository
    // Create conflicts
    // Run resolution
    // Verify results
}
```

## Continuous Integration

### GitHub Actions Workflows

We use GitHub Actions for CI/CD with multiple workflows:

#### CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and pull request:

- **Test Matrix:** Tests on Ubuntu, macOS, and Windows with stable and beta Rust
- **Clippy:** Linting with all warnings as errors
- **Format Check:** Ensures code is properly formatted
- **Coverage:** Generates code coverage reports
- **Security Audit:** Checks for vulnerable dependencies
- **Build:** Builds release binaries for all platforms

#### Release Workflow (`.github/workflows/release.yml`)

Runs on version tags:

- Builds release binaries for all platforms
- Creates GitHub release
- Uploads binary artifacts
- Publishes to crates.io (optional)

### Running CI Locally

#### Run Clippy
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

#### Check Formatting
```bash
cargo fmt --all -- --check
```

#### Run Security Audit
```bash
cargo install cargo-audit
cargo audit
```

## Test Organization

### Unit Tests

Located in the same file as the code:
```
src/
├── domain/
│   ├── conflict.rs      (contains #[cfg(test)] mod tests)
│   └── resolution.rs    (contains #[cfg(test)] mod tests)
```

### Integration Tests

Located in the `tests/` directory:
```
tests/
├── conflict_resolution.rs
├── git_operations.rs
└── helpers/
    └── mod.rs
```

### Test Helpers

Common test utilities:

```rust
// tests/helpers/mod.rs
use std::path::PathBuf;

pub fn create_temp_repo() -> PathBuf {
    // Helper to create temporary git repository
}

pub fn create_conflict_file(path: &PathBuf, content: &str) {
    // Helper to create file with conflicts
}
```

## Test Best Practices

### 1. Test Isolation

Each test should be independent:
```rust
#[test]
fn test_isolated() {
    // Create fresh state
    let mut file = ConflictedFile::new(/* ... */);

    // Test
    file.set_resolution(0, Resolution::Current);

    // Assert
    assert!(file.is_fully_resolved());
}
```

### 2. Use Descriptive Names

```rust
// Good
#[test]
fn test_conflicted_file_set_resolution_out_of_bounds() { }

// Bad
#[test]
fn test_bounds() { }
```

### 3. Test Edge Cases

```rust
#[test]
fn test_empty_conflicts() { }

#[test]
fn test_very_large_conflict() { }

#[test]
fn test_unicode_content() { }
```

### 4. Test Error Conditions

```rust
#[test]
#[should_panic(expected = "Invalid conflict range")]
fn test_invalid_range_panics() { }

#[test]
fn test_invalid_input_returns_error() {
    let result = parse_conflicts(invalid_path);
    assert!(result.is_err());
}
```

### 5. Use Fixtures for Complex Data

```rust
fn sample_conflict() -> ConflictHunk {
    ConflictHunk::new(
        "current".to_string(),
        "incoming".to_string(),
        0,
        5,
    )
}

#[test]
fn test_with_fixture() {
    let conflict = sample_conflict();
    // Use conflict in test
}
```

## Test Coverage Goals

### Short Term (Current Sprint)
- [x] Domain layer: 100% coverage
- [ ] Git parser: Additional edge case tests
- [ ] Git applier: Error handling tests

### Medium Term (Next Sprint)
- [ ] Application state: Full coverage
- [ ] Event handler: Key navigation tests
- [ ] Integration tests: End-to-end workflows

### Long Term (Future)
- [ ] TUI rendering: Snapshot tests
- [ ] Performance benchmarks
- [ ] Property-based testing with quickcheck

## Contributing Tests

When contributing new features:

1. **Write tests first** (TDD approach recommended)
2. **Cover happy path** - Normal usage scenarios
3. **Cover edge cases** - Boundary conditions
4. **Cover error cases** - Invalid inputs, failures
5. **Update this document** - Document new test patterns

### Pull Request Requirements

All PRs must:
- [ ] Include tests for new functionality
- [ ] All existing tests pass
- [ ] Code coverage doesn't decrease
- [ ] Pass clippy checks
- [ ] Be properly formatted

## Troubleshooting Tests

### Tests Fail on CI but Pass Locally

- Check platform-specific code (Windows vs Unix)
- Check file path separators
- Check line ending differences (CRLF vs LF)

### Flaky Tests

- Ensure tests are isolated
- Don't rely on timing
- Use deterministic test data
- Clean up temporary files properly

### Slow Tests

- Use `#[ignore]` for slow tests
- Run slow tests separately: `cargo test -- --ignored`
- Consider mocking expensive operations

## Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo test documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Rust Testing Guide](https://rust-lang.github.io/api-guidelines/necessities.html#all-items-have-a-rustdoc-example-c-example)

---

Last updated: January 25, 2026
