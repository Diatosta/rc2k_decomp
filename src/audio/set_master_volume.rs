//! The primary buffer's volume: nothing with A3D or without a primary buffer; else `SetVolume`
//! with the volume table at the volume, held to 0x40 (unsigned).

use crate::audio::own::AudioOwn;

pub fn set_master_volume(state: &AudioOwn, volume: u32) {
    if crate::audio::buffer_settings::a3d(state) {
        return;
    }
    let buffer = state.primary_buffer.get();
    if buffer == 0 {
        return;
    }
    let level = rc2k_core::audio::levels::master_volume(volume, |n| {
        crate::audio::buffer_settings::volumes(state, n)
    });
    crate::platform_sound::set_volume(buffer, level as i32);
}
