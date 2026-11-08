//! `audio::set_buffer_min_distance` (0x40C69A): a voice's 3D buffer's minimum distance.
//!
//! Takes: the voice
//!
//! `SetMinDistance` (+0x44) of its `buffer_3d` with its `min_distance` (an f32),
//! deferred (1). One caller, `audio::make_buffer`. Refuses a voice without its 3D buffer
//! (the original calls through 0).

use crate::audio::voices::Voice;
use crate::errors::{BadInput, Refused};

pub fn set_buffer_min_distance(v: &Voice) -> Result<i32, Refused> {
    if v.buffer_3d == 0 {
        return Err(BadInput("a voice without its 3D buffer").into());
    }
    // (Deferred: the platform takes it at once.)
    Ok(crate::platform_sound::set_min_distance(
        v.buffer_3d,
        f32::from_bits(v.min_distance),
    ))
}
