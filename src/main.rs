use anyhow::{Context, Result};
use tokio::sync::mpsc;

mod app;
mod mise;
mod models;
mod terminal;
mod ui;

use app::App;
use terminal::{cleanup_terminal, setup_terminal, spawn_input_handler, spawn_tick_handler};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger - controlled by RUST_LOG environment variable
    env_logger::init();
    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create event channel
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    // Create app
    let mut app = App::new(event_tx.clone());
    app.initialize().await.context("Failed to initialize app")?;

    // Spawn input and tick handlers
    spawn_input_handler(event_tx.clone());
    spawn_tick_handler(event_tx.clone());

    // Main event loop
    while let Some(event) = event_rx.recv().await {
        app.handle_event(event).await?;
        app.poll_task_output();

        terminal.draw(|f| app.draw(f))?;

        if app.should_quit() {
            break;
        }
    }

    // Cleanup
    cleanup_terminal(terminal)?;

    Ok(())
}
