//! Whether the single rally's three- and four-player items race all their players together on one
//! split screen (the advanced page's row, kept in `rc2k.toml`) or, as the original's, one after
//! another.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

static TOGETHER: AtomicBool = AtomicBool::new(false);

/// Whether three and four players race together.
pub fn together() -> bool {
    TOGETHER.load(Relaxed)
}

pub fn set_together(on: bool) {
    TOGETHER.store(on, Relaxed);
}
