pub mod input;
pub mod setup;

pub use input::{spawn_input_handler, spawn_tick_handler};
pub use setup::{cleanup_terminal, setup_terminal};
