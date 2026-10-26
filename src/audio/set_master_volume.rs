//! `audio::set_master_volume` (0x40A740): the primary buffer's volume.
//!
//! In:  eax = the volume, 0 .. 0x40
//! Out: every register preserved
//!
//! Nothing with A3D (`g_useA3d` 1: `nullsub_46`) or without a primary buffer
//! (`g_primaryBuffer`); else its `SetVolume` (+0x3C) with `g_volumeTable` at the volume,
//! held to 0x40 (unsigned). Callers: `sub_40A54B` (the mode's set-up) and
//! `audio::volume_command`.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose switch and
//! buffer are the test's, 72 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::com::{self};
use crate::audio::voices::*;
use crate::framework::{game_addr, game_read};

const SET_VOLUME: u32 = 0x3C;

// ---- Rust version ----

/// `primary`: where the primary buffer is kept.
pub unsafe fn set_master_volume(volume: u32, a3d: bool, primary: u32) {
    unsafe {
        if a3d {
            return;
        }
        let buffer = read(primary);
        if buffer == 0 {
            return;
        }
        // Converted: `rc2k_core::audio::levels::master_volume`.
        let level =
            rc2k_core::audio::levels::master_volume(volume, crate::audio::buffer_settings::volumes);
        com::call1(buffer, SET_VOLUME, level);
    }
}

pub unsafe fn set_master_volume_in_game(volume: u32) {
    unsafe {
        set_master_volume(
            volume,
            game_read::<u32>(A3D_RVA) == 1,
            game_addr(PRIMARY_RVA) as u32,
        )
    }
}
