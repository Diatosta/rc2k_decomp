//! `audio::release_voice` (0x40B767): a voice's buffers released and the voice made empty.
//!
//! In:  esi = the voice
//! Out: every register preserved
//!
//! With A3D (`g_useA3d` 1, the chunk at 0x40CD47): its buffer, if any, is stopped (A3D's
//! +0x38, `sub_40CEBB`) and released. Otherwise a buffer is stopped (`Stop`, +0x48,
//! `sub_40B963`), then its 3D buffer and the buffer released (`Release`, +8) and cleared.
//! Then the voice: `flags`, `data_key` .. `length`, `format`, `frames`, `priority`,
//! `kind`, `state`, `dirty`, `channel`, `next` 0; `own_frequency` 44100, `scale` and
//! `volume_factor` 0x40, `min_distance` 1.0. Callers: `audio::start` (0x40A0DB) and
//! `audio::free_voices`.
//! Shadow mode runs the original alone (a buffer is released once). Self-test: a
//! redirected copy whose A3D switch is the test's, voices with test objects, 12 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::com::{self, RELEASE};
use crate::audio::voices::*;
use crate::framework::game_read;

const STOP: u32 = 0x48;
const A3D_STOP: u32 = 0x38;

// ---- Rust version ----

pub unsafe fn release_voice(at: u32, a3d: bool) {
    unsafe {
        let v = voice(at);
        if a3d {
            if v.buffer != 0 {
                com::call(v.buffer, A3D_STOP);
                if v.buffer != 0 {
                    com::call(v.buffer, RELEASE);
                    v.buffer = 0;
                }
            }
        } else if v.buffer != 0 {
            com::call(v.buffer, STOP);
            if v.buffer_3d != 0 {
                com::call(v.buffer_3d, RELEASE);
                v.buffer_3d = 0;
            }
            if v.buffer != 0 {
                com::call(v.buffer, RELEASE);
                v.buffer = 0;
            }
        }
        // Converted: `rc2k_core::audio::voices::empty`.
        rc2k_core::audio::voices::empty(v);
    }
}

/// With the game's A3D switch.
pub unsafe fn release_voice_in_game(at: u32) {
    unsafe { release_voice(at, game_read::<u32>(A3D_RVA) == 1) }
}
