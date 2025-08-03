# Mise Command Sequencer

TUI for running individual mise tasks and sequencing commands in series.

## Project Structure
- `models/` - Core data types (MiseTask, AppEvent, AppState)
- `mise/` - Mise CLI integration
- `app/` - Application logic and event handling
- `ui/` - Terminal UI components
- `terminal/` - Terminal setup and input handling

## Commands
- `cargo run` - Start the TUI
- `cargo check` - Type check
- `cargo build` - Build release

## Dependencies
- ratatui - Terminal UI
- tokio - Async runtime
- anyhow - Error handling
- serde - JSON serialization