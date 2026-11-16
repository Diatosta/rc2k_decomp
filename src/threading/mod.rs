//! The game's own threads: their slots, how they start and how the shutdown waits for them.

pub mod request_quit;
pub mod shutdown_thread;
pub mod thread_slot;
