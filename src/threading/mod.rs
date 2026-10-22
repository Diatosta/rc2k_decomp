//! The game's threads, events and waits: no hooks left (conversion phase 4, 2026-10-22);
//! Rust callers call these directly.

pub mod create_event;
pub mod create_mutex;
pub mod find_thread_slot;
pub mod request_quit;
pub mod set_priority;
pub mod shutdown_thread;
pub mod start_slot_thread;
pub mod start_threads;
pub mod thread_slot;
pub mod wait_for_object;
pub mod wait_for_threads;
