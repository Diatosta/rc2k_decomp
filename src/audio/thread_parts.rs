//! The audio thread's small set-up and tear-down parts: the work buffer for reading samples, the
//! primary buffer's play and stop, the 3D listener's release.

use crate::audio::own::AudioOwn;
use crate::errors::Refused;

/// The setting's KB (0: one) taken on its list.
pub(super) fn take_sample_buffer(state: &AudioOwn) {
    let size = rc2k_core::audio::levels::sample_buffer_size(state.sample_buffer_kb.get());
    state.sample_buffer_size.set(size);
    // Rust's memory (`audio::sample_data`), counted as the game's block was.
    state
        .sample_data
        .take(crate::audio::sample_data::Buffer::Work, size);
    crate::memory::count_owned(&state.sample_buffer_memory, size);
}

/// Its list freed, size and buffer 0.
pub(super) fn free_sample_buffer(state: &AudioOwn) -> Result<(), Refused> {
    crate::memory::free_list::free_list(&state.sample_buffer_memory)?;
    state
        .sample_data
        .free(crate::audio::sample_data::Buffer::Work);
    state.sample_buffer_size.set(0);
    state.sample_buffer.set(0);
    Ok(())
}

/// The primary buffer played looping; a failure is logged.
pub(super) fn play_primary(state: &AudioOwn) {
    let p = state.primary_buffer.get();
    if p == 0 {
        return;
    }
    let r = crate::platform::sound::play(p, 1);
    if r != 0 {
        crate::audio::buffer_play::log_failed_play("the primary buffer", r);
    }
}

/// The primary buffer stopped.
pub(super) fn stop_primary(state: &AudioOwn) {
    let p = state.primary_buffer.get();
    if p != 0 {
        crate::platform::sound::stop(p);
    }
}

/// The listener released and forgotten.
pub(super) fn release_sound_listener(state: &AudioOwn) {
    let l = state.sound_listener.get();
    if l != 0 {
        crate::platform::sound::release(l);
        state.sound_listener.set(0);
    }
}
