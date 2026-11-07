//! `audio::open_eax` (0x40D27F): EAX 1's reverb opened through a 3D buffer.
//!
//! Takes: where the 3D buffer is (a voice's `buffer_3d`)
//!
//! See `audio/property_sets.rs`: nothing with `g_ignoreEaxSetting` or the set already there
//! (`g_eaxProperties`); "AUD: Using EAX" logged, `g_hasEax` 1. One caller,
//! `audio::make_buffer` with bit 0x1000000, where ebx is not always the holder (the game's
//! bug, `notes/bugs.md`: refused when no pointer).

use crate::audio::property_sets::*;
use crate::audio::store::AudioState;
use crate::errors::Refused;

/// True for the carry clear.
pub fn open_eax(state: &AudioState, holder: u32) -> Result<bool, Refused> {
    open_property_set(state, &Opening::eax(state), holder)
}
