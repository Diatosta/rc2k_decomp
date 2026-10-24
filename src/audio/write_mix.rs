//! `audio::write_mix` (0x40C03A): the work buffer clipped into the primary buffer's two
//! locked parts.
//!
//! In:  nothing (`g_lockedParts`, `g_soundMode`)
//! Out: eax .. ebp as its loops leave them (the one caller, `audio::mix`, keeps its own)
//!
//! See `audio/mixer.rs`; `sub_40C087`, `sub_40C0E0`, `sub_40C153` (the three formats)
//! inlined. Refuses sixteen bits with a right sample that clips (the original loops for
//! ever there: section 16).

use crate::audio::fill_buffer::LOCKED_PARTS_RVA;
use crate::audio::mixer::*;
use crate::audio::voices::SOUND_MODE_RVA;
use crate::errors::Refused;
use crate::store::{game_addr, game_read};

pub unsafe fn write_mix_in_game() -> Result<(), Refused> {
    unsafe {
        let parts = (game_addr(LOCKED_PARTS_RVA) as *const [u32; 4]).read_unaligned();
        write_mix(
            game_read(SOUND_MODE_RVA),
            game_addr(MIX_BUFFER_RVA) as u32,
            parts,
        )
    }
}
