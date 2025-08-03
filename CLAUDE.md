# Mise Command Sequencer

A Terminal User Interface (TUI) for building and executing sequences of mise tasks in a matrix-style interface, similar to a beat sequencer for development workflows.

## Project Goal

This application provides an intuitive way to:

- **Browse** available mise tasks from your project
- **Compose** multi-step sequences by assigning tasks to sequential execution steps
- **Execute** sequences with real-time output and progress tracking
- **Manage** individual tasks with quick access to view content and edit in VSCode

The core concept is a matrix interface where tasks are rows and execution steps are columns. Users can toggle tasks on/off for each step, creating custom workflow sequences that run automatically.

## Current Implementation Status

**Completed:**

- ✅ Basic TUI with mise task listing
- ✅ Individual task execution with output streaming
- ✅ Task detail view and information display
- ✅ Async event handling and terminal management
- ✅ Mise CLI integration for task discovery and execution

**Current Task: Matrix Sequence Builder**
Transforming from a simple task runner into a comprehensive sequence builder with the following target interface:

```
┌─ Available Tasks ─────────────────────────────────────────────────────────┐
│ Task Name     │ Step 1 │ Step 2 │ Step 3 │ [Run sequence] [Clear]         │
│ > build       │   ●    │        │   ●    │ [run] [cat] [edit]             │
│   test        │        │   ●    │        │ [run] [cat] [edit]             │
│   deploy      │        │        │   ●    │ [run] [cat] [edit]             │
└───────────────────────────────────────────────────────────────────────────┘
┌─ Task Output ─────────────────────────────────────────────────────────────┐
│ Step 1/3: Running 'build'...                                              │
│ [build output here]                                                       │
└───────────────────────────────────────────────────────────────────────────┘
┌─ Controls ────────────────────────────────────────────────────────────────────────────────────────────────┐
│ ↑/↓: Navigate | 1/2/3: Toggle step | Enter: Run sequence | Shift-Enter: Run current step | q: Quit        │
└───────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Architecture Overview

```
src/
├── app/              # Core application logic
│   ├── event_handlers.rs    # Keyboard/mouse event processing
│   ├── task_management.rs   # Individual task operations
│   └── sequence_management.rs # NEW: Multi-step sequence execution
├── mise/             # Mise CLI integration
│   └── client.rs     # Task discovery, info, and execution
├── models/           # Data structures
│   ├── app_event.rs  # Event system
│   ├── app_state.rs  # Application state management
│   ├── mise_task.rs  # Task and task info structures
│   └── sequence.rs   # NEW: Sequence state and configuration
├── terminal/         # Terminal I/O
│   ├── input.rs      # Keyboard input handling
│   └── setup.rs      # Terminal initialization
├── ui/               # User interface components
│   ├── task_list.rs  # EXISTING: Simple task list (fallback)
│   ├── task_detail.rs # EXISTING: Task detail view (fallback)
│   ├── task_running.rs # EXISTING: Task execution view (fallback)
│   └── sequence_builder.rs # NEW: Matrix sequence builder (primary)
└── main.rs           # Application entry point
```

## Planned Changes

### Phase 1: Core Matrix Interface

- [ ] **New Models:**

  - `SequenceState` struct to track task-to-step assignments
  - `AppEvent` variants for sequence operations (ToggleStep, RunSequence, etc.)
  - `AppState::SequenceBuilder` as new primary interface

- [ ] **Matrix UI Component:**

  - Grid layout with tasks as rows, steps as columns
  - Toggle buttons/indicators for each task-step combination
  - Action buttons (run/cat/edit) per task row
  - Global sequence controls (run/clear)

- [ ] **Event System Updates:**
  - Number keys (1/2/3) toggle selected task in respective steps
  - Navigation with arrow keys through task list
  - Mouse click support for action buttons

### Phase 2: Sequence Execution

- [ ] **Sequential Runner:**

  - Execute all enabled tasks in Step 1, then Step 2, then Step 3
  - Real-time output streaming to dedicated panel
  - Visual progress indication of current step
  - Stop-on-failure behavior

- [ ] **Output Management:**
  - Task output panel below matrix interface
  - Current step and task indication
  - Error highlighting and reporting

### Phase 3: Enhanced Task Management

- [ ] **VSCode Integration:**

  - "edit" button opens task file in VSCode (`code <filename>`)
  - Automatic file detection from mise task info

- [ ] **Quick Actions:**
  - "cat" button shows task content in popup/panel
  - "run" button executes individual task (existing functionality)

### Phase 4: Polish & Usability

- [ ] **Visual Enhancements:**

  - Color coding for step states (enabled/disabled/running/completed)
  - Better visual hierarchy and spacing
  - Loading states and animations

- [ ] **Keyboard Shortcuts:**
  - Comprehensive keyboard navigation
  - Quick task assignment shortcuts
  - Sequence management hotkeys

## Dependencies

- **ratatui** - Terminal UI framework
- **tokio** - Async runtime for concurrent task execution
- **anyhow** - Error handling and propagation
- **serde** - JSON serialization for mise CLI integration

## Development Commands

- `cargo run` - Start the sequence builder TUI
- `cargo check` - Type check all modules
- `cargo build --release` - Build optimized binary
- `mise run fmt` - Format code with trunk
- `mise run check` - Run linters and checks

## Future Enhancements

- **Sequence Templates:** Save/load common sequences
- **Conditional Execution:** Skip steps based on previous results
- **Parallel Execution:** Multiple tasks per step
- **Task Parameters:** Custom arguments per sequence step
- **Export:** Generate shell scripts from sequences
