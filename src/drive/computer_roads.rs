//! The advanced page's "computer cars on wide roads" switch: whether the computer driver tells a
//! wide road from a narrow one (`rc2k_core::drive::computer_driver::narrow`). Off, as in the
//! original, every road is driven as a narrow one (its slower speed table and its lower turn
//! limit for noting a bend); on, a wide road uses the first speed table and the higher limit.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

static ON: AtomicBool = AtomicBool::new(false);

/// Whether wide roads are driven as wide.
pub fn on() -> bool {
    ON.load(Relaxed)
}

pub fn set_on(on: bool) {
    ON.store(on, Relaxed);
}
