//! The advanced page's switch for the engine sounds' smooth loops (kept in `rc2k.toml`): the
//! blend of a looping sample's end into its start, made when the sample is loaded, and the
//! overlapping grains an engine's loop is mixed as, decided where each voice is mixed. Off, a
//! loop is the plain loop.

use std::sync::atomic::{AtomicBool, Ordering};

static SMOOTH_ON: AtomicBool = AtomicBool::new(true);

pub fn smooth_on() -> bool {
    SMOOTH_ON.load(Ordering::Relaxed)
}

pub fn set_smooth_on(on: bool) {
    SMOOTH_ON.store(on, Ordering::Relaxed);
}
