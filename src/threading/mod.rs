//! The game's own threads: their slots, how they start and how the shutdown waits for them.

pub(crate) mod request_quit;
mod shutdown_thread;
pub(crate) mod thread_slot;
