//! The windscreen's drops and flakes spawn once a frame; with frames faster than the
//! simulation's ticks they spawn once a tick instead (`rc2k_core::race::blend::spawn_due`), so a
//! faster screen gets no more rain. At a hundred frames a second or fewer every frame spawns, as
//! it always did.

use rc2k_core::race::blend::spawn_due;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// One effect's last spawn.
pub struct SpawnGate(AtomicU32);

impl SpawnGate {
    pub const fn new() -> Self {
        SpawnGate(AtomicU32::new(0))
    }

    /// Whether the frame at clock `now` (the scene's, in milliseconds) spawns.
    pub fn due(&self, now: u32) -> bool {
        match spawn_due(self.0.load(Relaxed), now) {
            Some(last) => {
                self.0.store(last, Relaxed);
                true
            }
            None => false,
        }
    }
}

/// The drops' gate.
pub static DROPS: SpawnGate = SpawnGate::new();

/// The flakes' gate.
pub static FLAKES: SpawnGate = SpawnGate::new();
