//! `audio::write_mix` (0x40C03A): the work buffer clipped into the primary buffer's two
//! locked parts.
//!
//! Works on `g_lockedParts` and `g_soundMode`.
//!
//! See `audio/mixer.rs`; 0x40C087, 0x40C0E0, 0x40C153 (the three formats)
//! inlined. Refuses sixteen bits with a right sample that clips (the original loops for
//! ever there: section 16).

use crate::audio::mixer::*;
use crate::audio::own::AudioOwn;
use crate::errors::Refused;
use crate::session::PlainDword;

pub fn write_mix_in_game(state: &AudioOwn) -> Result<(), Refused> {
    let parts = state.locked_parts.each_ref().map(PlainDword::get);
    write_mix(state.sound_mode.get(), state.primary_buffer.get(), parts)
}
