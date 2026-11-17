//! The layout of a split screen of three or four players, among the ones `Split::many_layouts`
//! lists: the one chosen in the race (the split key cycles them) and kept between sessions in
//! `rc2k.toml` (`display.split_layout`). The two-player split keeps its orientation in the race's
//! own settings.

use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

static INDEX: AtomicU32 = AtomicU32::new(0);

/// The layout's index (wrapped round the layouts of the race's player count where it is used).
pub fn index() -> u32 {
    INDEX.load(Relaxed)
}

/// Sets the layout's index.
pub fn set_index(index: u32) {
    INDEX.store(index, Relaxed);
}

/// The layout after the current one for `players` players, set and given back.
pub fn cycle(players: u32) -> u32 {
    let next = rc2k_core::race::view::Split::next_many(players, index());
    set_index(next);
    next
}
