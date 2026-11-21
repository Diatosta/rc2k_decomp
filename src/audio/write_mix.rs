//! Clips the work buffer into the primary buffer's two locked parts, by the sound mode's format
//! (see `audio/mixer.rs`). Refuses sixteen bits with a right sample that clips (the clamp there
//! would loop for ever).

use crate::audio::mixer::*;
use crate::audio::own::AudioOwn;
use crate::errors::Refused;
use crate::memory::session::PlainDword;

pub(super) fn write_mix_in_game(state: &AudioOwn) -> Result<(), Refused> {
    let parts = state.locked_parts.each_ref().map(PlainDword::get);
    write_mix(
        state,
        state.sound_mode.get(),
        state.primary_buffer.get(),
        parts,
    )
}
