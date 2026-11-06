//! `audio::write_mix` (0x40C03A): the work buffer clipped into the primary buffer's two
//! locked parts.
//!
//! In:  nothing (`g_lockedParts`, `g_soundMode`)
//! Out: eax .. ebp as its loops leave them (the one caller, `audio::mix`, keeps its own)
//!
//! See `audio/mixer.rs`; `sub_40C087`, `sub_40C0E0`, `sub_40C153` (the three formats)
//! inlined. Refuses sixteen bits with a right sample that clips (the original loops for
//! ever there: section 16).

use crate::audio::mixer::*;
use crate::errors::Refused;

pub fn write_mix_in_game() -> Result<(), Refused> {
    let parts = crate::audio::owner::audio()
        .state
        .locked_parts
        .each_ref()
        .map(super::super::session::Dword::get);
    write_mix(
        crate::audio::owner::audio().state.sound_mode.get(),
        crate::audio::owner::audio().state.primary_buffer.get(),
        parts,
    )
}
