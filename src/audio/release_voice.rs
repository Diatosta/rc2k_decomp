//! Releases a voice's buffers and makes the voice empty: a buffer is stopped, then its 3D buffer
//! and the buffer released, and the voice's fields reset (own frequency 44100, scale and volume
//! factor 0x40, minimum distance 1.0, the rest 0).

use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::platform::sound as ds;

pub fn release_voice(state: &AudioOwn, at: u32) {
    update_voice(state, at, |v| {
        if v.buffer != 0 {
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
