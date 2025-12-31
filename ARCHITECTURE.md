# Enkai Architecture

## Overview

Enkai is a TUI (Terminal User Interface) application for resolving git conflicts during merge or rebase operations. The architecture follows a layered design with clear separation of concerns.

## Architecture Layers

```
┌─────────────────────────────────────────────────┐
│  TUI Layer (src/tui/)                           │
│  - User Interface & Rendering                   │
│  - Event Handling                               │
│  - Views (file_list, conflict_view, rebase)    │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  Application Layer (src/app/)                   │
│  - State Management                             │
│  - Navigation Logic                             │
│  - View Mode Control                            │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  Domain Layer (src/domain/)                     │
│  - Core Business Models                         │
│  - Conflict Representation                      │
│  - Resolution Strategies                        │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  Infrastructure Layer (src/git/)                │
│  - Git Integration                              │
│  - File I/O                                     │
│  - Conflict Parsing                             │
│  - Command Execution                            │
└─────────────────────────────────────────────────┘
```

## Module Structure

### Domain Layer (`src/domain/`)

**Purpose**: Core business logic and data models

- `conflict.rs`: Defines `ConflictHunk` and `ConflictedFile` structures
- `resolution.rs`: `Resolution` enum (Current, Incoming, Both)
- `git_operation.rs`: `GitOperation` enum (Merge, Rebase, RebaseInteractive)

**Key Types**:
```rust
ConflictHunk {
    current: String,
    incoming: String,
    start_line: usize,
    end_line: usize,
}

ConflictedFile {
    path: PathBuf,
    conflicts: Vec<ConflictHunk>,
    resolutions: Vec<Option<Resolution>>,
    original_content: String,
}
```

### Application Layer (`src/app/`)

**Purpose**: Application state and business logic

- `state.rs`: Manages application state and navigation

**Key Types**:
```rust
AppState {
    files: Vec<ConflictedFile>,
    view_mode: ViewMode,
    selected_file: usize,
    git_operation: GitOperation,
    should_quit: bool,
}

ViewMode {
    FileList,
    ConflictResolve { file_index, conflict_index },
    RebaseActions,
}
```

### Git Layer (`src/git/`)

**Purpose**: Git integration and file operations

- `detector.rs`: Detects git operations and finds conflicted files
- `parser.rs`: Parses conflict markers from files
- `applier.rs`: Applies resolutions to files
- `commands.rs`: Executes git commands (continue, abort, skip)

**Key Functions**:
- `detect_git_operation()`: Determines if merge or rebase is in progress
- `find_conflicted_files()`: Lists all files with conflicts
- `parse_conflicts()`: Parses conflict markers into `ConflictHunk` structures
- `apply_resolutions()`: Writes resolved content back to files

### TUI Layer (`src/tui/`)

**Purpose**: Terminal user interface

- `app.rs`: Main TUI loop and terminal setup
- `event.rs`: Keyboard event handling
- `views/`: UI rendering for different views
  - `file_list.rs`: List of conflicted files
  - `conflict_view.rs`: Conflict resolution interface
  - `rebase_actions.rs`: Rebase action selector

## Data Flow

### 1. Application Startup

```
main.rs
  → Open git repository
  → Detect git operation type
  → Find conflicted files
  → Parse conflicts
  → Create AppState
  → Run TUI
```

### 2. Conflict Resolution Flow

```
User Input (keyboard)
  → Event Handler (src/tui/event.rs)
  → AppState mutation (src/app/state.rs)
  → UI Re-render (src/tui/views/*.rs)
```

### 3. Save Flow

```
User presses 's' to save
  → Check all conflicts resolved
  → apply_resolutions() (src/git/applier.rs)
  → Write to file system
  → Back to file list
  → If all files resolved + rebase → show rebase actions
```

## Design Patterns

### 1. **State Pattern**
- `ViewMode` enum represents different UI states
- State transitions controlled through `AppState` methods

### 2. **Repository Pattern** (Git Layer)
- Abstracts git operations behind clean interfaces
- Separates git logic from business logic

### 3. **Model-View Pattern**
- Domain models are separate from views
- Views render based on current state
- State mutations happen through dedicated methods

## Key Design Decisions

### 1. Single-Pane UI
- **Why**: Simpler navigation, less screen clutter
- **How**: View transitions instead of split screens

### 2. Vim-Style + Arrow Keys
- **Why**: Accessibility for all users
- **How**: Dual keybinding support in event handler

### 3. Conflict Marker Parsing
- **Why**: No git library dependency for parsing
- **How**: Custom parser for `<<<<<<<`, `=======`, `>>>>>>>` markers

### 4. Auto-Detection
- **Why**: Zero-config workflow
- **How**: Scan git repository state on startup

### 5. Immediate Save
- **Why**: Simple workflow, predictable behavior
- **How**: Direct file writes on 's' key press

## Testing Strategy

### Unit Tests
- Domain models (`ConflictHunk`, `Resolution`)
- Conflict parser with various marker patterns
- Resolution applier logic

### Integration Tests
- Git operations with temporary repositories
- End-to-end conflict resolution workflows

## Future Enhancements

1. **Custom Merge Strategies**: Allow user-defined resolution patterns
2. **Diff Visualization**: Show line-by-line diffs with syntax highlighting
3. **Undo/Redo**: Navigate resolution history
4. **Configuration File**: User preferences for keybindings and colors
5. **Three-Way Merge View**: Show base, current, and incoming
6. **Conflict Search**: Jump to next/previous file with conflicts
7. **Git Integration**: Auto-stage resolved files

## Dependencies

- `ratatui`: TUI framework
- `crossterm`: Terminal manipulation
- `git2`: Git repository operations
- `anyhow`: Error handling
- `clap`: CLI argument parsing

## Performance Considerations

- Lazy loading for large files
- Efficient string operations for conflict resolution
- Minimal allocations in render loop
- Terminal buffer optimization through ratatui

---

Last updated: 2025-12-31
