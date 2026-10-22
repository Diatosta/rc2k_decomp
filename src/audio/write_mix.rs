//! `audio::write_mix` (0x40C03A): the work buffer clipped into the primary buffer's two
//! locked parts.
//!
//! In:  nothing (`g_lockedParts`, `g_soundMode`)
//! Out: eax .. ebp as its loops leave them (the one caller, `audio::mix`, keeps its own)
//!
//! See `audio/mixer.rs`; `sub_40C087`, `sub_40C0E0`, `sub_40C153` (the three formats)
//! inlined. Refuses sixteen bits with a right sample that clips (the original loops for
//! ever there: section 16).
//! Compared in shadow mode (the two parts). Self-test: a redirected copy (0x40C03A ..
//! 0x40C18C with the three) whose mode, buffer and parts are the test's, 4 formats x 2
//! splits x 4 loudnesses (the loudest not for sixteen bits).
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::fill_buffer::LOCKED_PARTS_RVA;
use crate::audio::mixer::*;
use crate::audio::voices::SOUND_MODE_RVA;
use crate::errors::Refused;
use crate::framework::{game_addr, game_read};

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
