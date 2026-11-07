//! `audio::open_voice_manager` (0x40C6B5): DirectSound's voice manager ("DSV") opened
//! through a 3D buffer.
//!
//! Takes: where the 3D buffer is (a voice's `buffer_3d`)
//!
//! See `audio/property_sets.rs`: nothing with `g_ignoreDsvSetting` or the set already
//! there (`g_voiceManager`); its mode set to
//! 2, "AUD: Using DSV" logged, `g_voiceManagerOn` 1. One caller, `audio::make_buffer` with bit
//! 0x800000.

use crate::audio::property_sets::*;
use crate::audio::store::AudioState;
use crate::errors::Refused;

/// True for the carry clear.
pub fn open_voice_manager(state: &AudioState, holder: u32) -> Result<bool, Refused> {
    open_property_set(state, &Opening::voice_manager(state), holder)
}
