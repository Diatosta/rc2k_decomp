//! `audio::clear_channels` (0x40ACA1): the first 0x10 mixer channels made free.
//!
//! Each channel's `used`, `voice` and the five dwords from +0x44 on are 0. Five callers
//! (the mode's set-up, `audio::free_voices`, the mixer's start).

use crate::audio::owner::Audio;
use crate::audio::voices::GamePool;

pub fn clear_channels(audio: &Audio) {
    // Converted: `rc2k_core::audio::voices::clear_channels`.
    rc2k_core::audio::voices::clear_channels(&mut GamePool::new(audio));
}
