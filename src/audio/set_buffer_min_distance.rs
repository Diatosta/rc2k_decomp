//! A voice's 3D buffer's minimum distance: `SetMinDistance` with its `min_distance` (an f32),
//! deferred. Refuses a voice without its 3D buffer.

use crate::audio::voices::Voice;
use crate::errors::{BadInput, Refused};

pub fn set_buffer_min_distance(v: &Voice) -> Result<i32, Refused> {
    if v.buffer_3d == 0 {
        return Err(BadInput("a voice without its 3D buffer").into());
    }
    // The platform takes it at once.
    Ok(crate::platform_sound::set_min_distance(
        v.buffer_3d,
        f32::from_bits(v.min_distance),
    ))
}
