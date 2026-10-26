//! The audio thread's small set-up and tear-down parts (IDA, 2026-10-15): the work buffer
//! for reading samples, the primary buffer's play and stop, the 3D listener's release.

use crate::audio::store::AUDIO;
use crate::errors::Refused;

/// `audio::take_sample_buffer`: the setting's KB (0: one) taken on its list.
pub fn take_sample_buffer() -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::levels::sample_buffer_size`.
    let size = rc2k_core::audio::levels::sample_buffer_size(AUDIO.sample_buffer_kb.get());
    AUDIO.sample_buffer_size.set(size);
    let b = crate::memory::allocate::allocate(AUDIO.sample_buffer_memory.as_ptr(), size)?;
    AUDIO.sample_buffer.set(crate::pointers::game_address(b));
    Ok(())
}

/// `audio::free_sample_buffer`: its list freed, size and buffer 0.
pub fn free_sample_buffer() -> Result<(), Refused> {
    crate::memory::free_list::free_list(AUDIO.sample_buffer_memory.as_ptr())?;
    AUDIO.sample_buffer_size.set(0);
    AUDIO.sample_buffer.set(0);
    Ok(())
}

/// `audio::play_primary`: unless A3D, the primary buffer played looping; a failure sets
/// the lost flag.
pub fn play_primary() {
    let p = AUDIO.primary_buffer.get();
    if crate::audio::buffer_settings::a3d() || p == 0 {
        return;
    }
    if crate::platform_sound::play(p, 1) != 0 {
        AUDIO.sound_buffer_lost.set(1);
    }
}

/// `audio::stop_primary`: unless A3D, the primary buffer stopped.
pub fn stop_primary() {
    let p = AUDIO.primary_buffer.get();
    if !crate::audio::buffer_settings::a3d() && p != 0 {
        crate::platform_sound::stop(p);
    }
}

/// `audio::release_sound_listener`: the listener released and forgotten.
pub fn release_sound_listener() {
    let l = AUDIO.sound_listener.get();
    if l != 0 {
        crate::platform_sound::release(l);
        AUDIO.sound_listener.set(0);
    }
}
