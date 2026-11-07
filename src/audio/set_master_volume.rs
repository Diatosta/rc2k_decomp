//! `audio::set_master_volume` (0x40A740): the primary buffer's volume.
//!
//! In:  eax = the volume, 0 .. 0x40
//!
//! Nothing with A3D (`g_useA3d` 1: `nullsub_46`) or without a primary buffer
//! (`g_primaryBuffer`); else its `SetVolume` (+0x3C) with `g_volumeTable` at the volume,
//! held to 0x40 (unsigned). Callers: `sub_40A54B` (the mode's set-up) and
//! `audio::volume_command`.

use crate::audio::store::AudioState;

// ---- Rust version ----

pub fn set_master_volume_in_game(state: &AudioState, volume: u32) {
    if crate::audio::buffer_settings::a3d(state) {
        return;
    }
    let buffer = state.primary_buffer.get();
    if buffer == 0 {
        return;
    }
    // Converted: `rc2k_core::audio::levels::master_volume`.
    let level = rc2k_core::audio::levels::master_volume(volume, |n| {
        crate::audio::buffer_settings::volumes(state, n)
    });
    crate::platform_sound::set_volume(buffer, level as i32);
}
