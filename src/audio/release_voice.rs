//! Releases a voice's buffers and makes the voice empty: a buffer is stopped, then its 3D buffer
//! and the buffer released, and the voice's fields reset (own frequency 44100, scale and volume
//! factor 0x40, minimum distance 1.0, the rest 0).

use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::platform_sound as ds;

pub fn release_voice(state: &AudioOwn, at: u32, a3d: bool) {
    update_voice(state, at, |v| {
        if a3d {
            // A3D's source objects: A3D is never there (`audio::a3d`), so no voice has one.
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
        rc2k_core::audio::voices::empty(v);
    });
}

/// With the game's A3D switch.
pub fn release_voice_in_game(state: &AudioOwn, at: u32) {
    release_voice(state, at, state.use_a3d.get() == 1);
}
