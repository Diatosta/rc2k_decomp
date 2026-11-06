//! The audio thread's small set-up and tear-down parts (IDA, 2026-10-15): the work buffer
//! for reading samples, the primary buffer's play and stop, the 3D listener's release.

use crate::audio::owner::audio;
use crate::errors::Refused;

/// `audio::take_sample_buffer`: the setting's KB (0: one) taken on its list.
pub fn take_sample_buffer() -> Result<(), Refused> {
    let audio = audio();
    // Converted: `rc2k_core::audio::levels::sample_buffer_size`.
    let size = rc2k_core::audio::levels::sample_buffer_size(audio.state.sample_buffer_kb.get());
    audio.state.sample_buffer_size.set(size);
    // Rust's memory (`audio::sample_data`), counted as the game's block was;
    // `g_sampleBuffer` is no longer an address (it stays 0).
    crate::audio::sample_data::take(crate::audio::sample_data::Buffer::Work, size);
    crate::memory::count_owned(&audio.state.sample_buffer_memory, size);
    Ok(())
}

/// `audio::free_sample_buffer`: its list freed, size and buffer 0.
pub fn free_sample_buffer() -> Result<(), Refused> {
    let state = &audio().state;
    crate::memory::free_list::free_list(crate::pointers::GamePtr::to(&state.sample_buffer_memory))?;
    crate::audio::sample_data::free(crate::audio::sample_data::Buffer::Work);
    state.sample_buffer_size.set(0);
    state.sample_buffer.set(0);
    Ok(())
}

/// `audio::play_primary`: unless A3D, the primary buffer played looping; a failure sets
/// the lost flag.
pub fn play_primary() {
    let state = &audio().state;
    let p = state.primary_buffer.get();
    if crate::audio::buffer_settings::a3d() || p == 0 {
        return;
    }
    if crate::platform_sound::play(p, 1) != 0 {
        state.sound_buffer_lost.set(1);
    }
}

/// `audio::stop_primary`: unless A3D, the primary buffer stopped.
pub fn stop_primary() {
    let p = audio().state.primary_buffer.get();
    if !crate::audio::buffer_settings::a3d() && p != 0 {
        crate::platform_sound::stop(p);
    }
}

/// `audio::release_sound_listener`: the listener released and forgotten.
pub fn release_sound_listener() {
    let state = &audio().state;
    let l = state.sound_listener.get();
    if l != 0 {
        crate::platform_sound::release(l);
        state.sound_listener.set(0);
    }
}
