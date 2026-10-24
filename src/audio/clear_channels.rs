//! `audio::clear_channels` (0x40ACA1): the first 0x10 mixer channels made free.
//!
//! In:  nothing
//! Out: every register preserved (the flags of `test eax, eax`, which no caller reads)
//!
//! Each channel's `used`, `voice` and the five dwords from +0x44 on are 0. Five callers
//! (the mode's set-up, `audio::free_voices`, the mixer's start).

use crate::audio::voices::*;

// ---- Rust version ----

pub unsafe fn clear_channels(channels: u32) {
    // Converted: `rc2k_core::audio::voices::clear_channels`.
    let w = World {
        channels,
        ..World::game()
    };
    rc2k_core::audio::voices::clear_channels(&mut unsafe { GamePool::new(&w) });
}
