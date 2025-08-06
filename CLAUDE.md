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
- ✅ Matrix sequence builder with grid interface
- ✅ Sequential task execution with real-time output
- ✅ VSCode integration for task editing
- ✅ Task content display and information viewing
- ✅ Comprehensive keyboard controls
- ✅ Mouse interaction support (clicking buttons, scrolling)
- ✅ Button hover states and visual feedback
- ✅ Auto-hiding task list optimization
- ✅ Scrolling support for large task lists
- ✅ Dynamic run/stop buttons with per-task state tracking
- ✅ Visual execution feedback (red stop buttons, current step highlighting)
- ✅ Concurrency control preventing simultaneous task execution
- ✅ Smart button state management (greyed out when unavailable)

The application now features a complete matrix-style interface:

```
┌─ Available Tasks ─────────────────────────────────────────────────────────┐
│ Task Name     │ Step 1 │ Step 2 │ Step 3 │ [Stop sequence] [Clear]        │
│ > build       │   ●    │   ●̂    │   ●    │ [stop] [cat] [edit]            │
│   test        │        │   ●    │        │ [run] [cat] [edit]             │
│   deploy      │        │        │   ●    │ [run] [cat] [edit]             │
└───────────────────────────────────────────────────────────────────────────┘
┌─ Task Output ─────────────────────────────────────────────────────────────┐
│ Step 2/3: Running 'build'...                                              │
│ [build output here]                                                       │
└───────────────────────────────────────────────────────────────────────────┘
┌─ Controls ────────────────────────────────────────────────────────────────────────────────────────────────┐
│ ↑/↓: Navigate | 1/2/3: Toggle step | Enter: Run sequence | Shift-Enter: Run current step | q: Quit        │
└───────────────────────────────────────────────────────────────────────────────────────────────────────────┘

Visual States:
• Green steps (●): Task assigned to step
• Red steps (●̂): Currently executing step  
• Red buttons: Stop actions ([stop], [Stop sequence])
• Greyed buttons: Unavailable actions (other tasks running)
```

## Controls

### Keyboard
- `↑/↓`: Navigate through tasks
- `1/2/3`: Toggle task in respective steps (Step 1, Step 2, Step 3)
- `Enter`: Run entire sequence
- `x`: Run current task individually
- `e`: Edit current task in VSCode
- `Tab`: Show task info/content
- `c`: Clear all sequence assignments
- `q`: Quit application

### Mouse
- **Click buttons**: Direct interaction with task action buttons
  - `[run]`: Start task execution (cyan/blue) 
  - `[stop]`: Stop running task (red)
  - `[cat]`, `[edit]`: Task information and editing (cyan)
- **Click sequence controls**: [Run sequence], [Stop sequence], [Add as task], [Clear] buttons
  - Dynamic text and colors based on execution state
  - Greyed out when unavailable (prevents conflicts)
- **Click step toggles**: Enable/disable tasks for specific sequence steps
  - Green background: Task assigned to step
  - Red background: Currently executing step
- **Scroll**: Navigate through task lists with mouse wheel
- **Hover**: Visual feedback on interactive elements

### Visual Feedback
- **Red buttons**: Indicate stop actions for running processes
- **Greyed buttons**: Show unavailable actions (prevents simultaneous execution)
- **Red step indicators**: Highlight currently executing sequence step
- **Per-task state**: Each task shows its individual running state independently

## Architecture Overview

```
src/
├── app/              # Core application logic
│   ├── event_handlers.rs    # Keyboard/mouse event processing
│   ├── task_management.rs   # Individual task operations
│   └── sequence_management.rs # Multi-step sequence execution
├── mise/             # Mise CLI integration
│   └── client.rs     # Task discovery, info, and execution
├── models/           # Data structures
│   ├── app_event.rs  # Event system
│   ├── app_state.rs  # Application state management
│   ├── mise_task.rs  # Task and task info structures
│   └── sequence.rs   # Sequence state and configuration
├── terminal/         # Terminal I/O
│   ├── input.rs      # Keyboard input handling
│   └── setup.rs      # Terminal initialization
├── ui/               # User interface components
│   ├── task_list.rs  # Simple task list (fallback)
│   ├── task_detail.rs # Task detail view (fallback)
│   ├── task_running.rs # Task execution view (fallback)
│   ├── sequence_builder.rs # Matrix sequence builder (primary)
│   ├── button_layout.rs # Mouse interaction and button positioning
│   └── constants.rs  # UI constants and styling definitions
└── main.rs           # Application entry point
```

## Dependencies

- **ratatui** - Terminal UI framework
- **tokio** - Async runtime for concurrent task execution
- **anyhow** - Error handling and propagation
- **serde** - JSON serialization for mise CLI integration

## Development Commands

- `mise run check_green` - Command to run until green (includes coverage)
- `cargo run` - Start the sequence builder TUI
- `cargo check` - Type check all modules
- `cargo build --release` - Build optimized binary
- `mise run fmt` - Format code with trunk
- `mise run check` - Run linters and checks
- `mise run coverage` - Generate LCOV coverage report for VSCode
- `mise run coverage-watch` - Watch files and regenerate coverage on changes

## Future Enhancements

- **Sequence Templates:** Save/load common sequences
- **Conditional Execution:** Skip steps based on previous results
- **Parallel Execution:** Multiple tasks per step
- **Task Parameters:** Custom arguments per sequence step
- **Export:** Generate shell scripts from sequences
