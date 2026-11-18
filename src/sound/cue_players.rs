//! How many cars' co-drivers speak in the race being set up: the first two players', or the three
//! or four of a split screen. The race's sound set-up says it, the physics' pace notes and the
//! sound turn ask.

use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// The cars whose spoken cues are queued and played (a one-player race's second object takes
/// none, as it is not in use).
static COUNT: AtomicU32 = AtomicU32::new(2);

/// The race's co-driver players: 2 for every race but a split screen of three or four players.
pub fn set(count: u32) {
    COUNT.store(count, Relaxed);
}

/// The cars whose co-driver speaks.
pub fn count() -> u32 {
    COUNT.load(Relaxed)
}
