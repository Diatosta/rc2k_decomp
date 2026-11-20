//! What a replay of this project's own format (`rc2k_core::race::replay_v2`) keeps beside the
//! original's frame: the cars of the replay (its extension) and the block's tail, the part that
//! cars 4 .. 7 and the cars recorded by their states add. The tail is gathered while a second
//! is recorded and written with the block; a block played has its tail read into the same place.

use rc2k_core::race::replay_v2::{Extension, Tail};
use std::sync::{LazyLock, Mutex, MutexGuard};

/// The replay's cars (none for the old format) and the tail of the block.
pub struct V2 {
    pub ext: Option<Extension>,
    pub tail: Tail,
}

impl V2 {
    /// The cars recorded by their states, bit n for car n; 0 without an extension.
    pub fn state_mask(&self) -> u8 {
        self.ext.as_ref().map_or(0, Extension::state_mask)
    }
}

/// The store, locked for short uses.
pub struct ReplayV2(LazyLock<Mutex<V2>>);

impl ReplayV2 {
    pub const fn new() -> Self {
        ReplayV2(LazyLock::new(|| {
            Mutex::new(V2 {
                ext: None,
                tail: Tail::new(),
            })
        }))
    }

    /// The state, for a short use (never held over a call that takes it again).
    pub fn lock(&self) -> MutexGuard<'_, V2> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Default for ReplayV2 {
    fn default() -> Self {
        Self::new()
    }
}
