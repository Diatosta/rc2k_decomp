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

use crate::audio::store::AudioState;
use crate::audio::voices::*;
use crate::platform_sound as ds;

// ---- Rust version ----

pub fn release_voice(state: &AudioState, at: u32, a3d: bool) {
    update_voice(state, at, |v| {
        if a3d {
            // A3D's source objects (`Stop` at +0x38, `Release`): A3D is never there
            // (`audio::a3d`), so no voice has one.
            if v.buffer != 0 {
                crate::fatal::raise::end(
                    "audio::release_voice",
                    crate::errors::BadInput("an A3D voice (A3D is never there)"),
                );
            }
        } else if v.buffer != 0 {
            ds::stop(v.buffer);
            if v.buffer_3d != 0 {
                ds::release(v.buffer_3d);
                v.buffer_3d = 0;
            }
            if v.buffer != 0 {
                ds::release(v.buffer);
                v.buffer = 0;
            }
        }
        // Converted: `rc2k_core::audio::voices::empty`.
        rc2k_core::audio::voices::empty(v);
    });
}

/// With the game's A3D switch.
pub fn release_voice_in_game(state: &AudioState, at: u32) {
    release_voice(state, at, state.use_a3d.get() == 1);
}
