//! What the tests share to read the game's first values and to put the game's state back,
//! by store name and field path rather than by address: `first_values` (the frozen first
//! values in `data/first_values.golden`) and `state` (a snapshot of every store).

pub mod first_values;
mod freeze;
pub mod front;
pub mod race_places;
pub mod race_vectors;
pub mod state;
pub mod vectors;

/// The lock every test holds that writes the game's statics (and restores them), or reads
/// their first values: tests run in parallel threads.
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
