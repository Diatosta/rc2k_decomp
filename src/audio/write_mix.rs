//! `audio::write_mix` (0x40C03A): the work buffer clipped into the primary buffer's two
//! locked parts.
//!
//! In:  nothing (`g_lockedParts`, `g_soundMode`)
//!
//! See `audio/mixer.rs`; 0x40C087, 0x40C0E0, 0x40C153 (the three formats)
//! inlined. Refuses sixteen bits with a right sample that clips (the original loops for
//! ever there: section 16).

use crate::audio::mixer::*;
use crate::audio::store::AudioState;
use crate::errors::Refused;

pub fn write_mix_in_game(state: &AudioState) -> Result<(), Refused> {
    let parts = state
        .locked_parts
        .each_ref()
        .map(super::super::session::Dword::get);
    write_mix(state.sound_mode.get(), state.primary_buffer.get(), parts)
}
