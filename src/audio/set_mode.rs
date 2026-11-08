//! `audio::set_mode` (0x40A426): command 5, the sound mode changed (menus <-> a race).
//!
//! "SAM:" and the mode logged; `audio::stop_streams`, `audio::free_voices`,
//! `audio::close_way`, `audio::release_primary`; `g_soundMode` the new mode; the channels:
//! 8 (0x10 with bit 0x400000), held to the free hardware buffers of the mode's kind (2D or
//! 3D: bit 0x10000 all, 0x20000 the other count of `g_soundCaps`) when there are any, and
//! at least 6, into `g_channelCount`; `audio::reopen_sound`, `audio::make_primary` (either
//! failing: `g_openedSoundMode` 0, "SAM: FAIL", the carry), `audio::open_way`, a second's
//! `system::sleep`, `audio::play_primary`; `g_openedSoundMode` the mode, "SAM: OK". Either
//! way `system::restore_window` last (its flags kept around it). Callers: the audio thread's
//! start (the menus' mode), `audio::take_commands` (command 5).

use crate::audio::owner::Audio;
use crate::errors::Refused;

const SAM_TEXT: &std::ffi::CStr = c"SAM:"; // "SAM:"
const OK_TEXT: &std::ffi::CStr = c"SAM: OK"; // "SAM: OK"
const FAIL_TEXT: &std::ffi::CStr = c"SAM: FAIL"; // "SAM: FAIL"

/// True for no carry.
pub fn set_mode(audio: &Audio, mode: u32) -> Result<bool, Refused> {
    use crate::audio::mixer_set_up::{close_way, open_way};
    use crate::audio::sound_device as sd;
    let caps = &audio.state.sound_caps;
    sd::log_text(SAM_TEXT, false);
    sd::log_hex(mode, true);
    // Fixed (section 16, 2026-10-20): the streams stopped with action 0; the original
    // hands on whatever edx holds (normally the thread's 1, a feed), so the music
    // outlived the DirectSound it played on.
    crate::audio::stop_streams::stop_streams(audio)?;
    crate::audio::free_voices::free_voices(audio)?;
    close_way(audio, audio.state.sound_mode.get())?;
    sd::release_primary(&audio.state)?;
    audio.state.sound_mode.set(mode);
    // Converted: `rc2k_core::audio::modes::channels`. `g_soundCaps`: free hardware buffers
    // +0x24 all, +0x28 static, +0x3C 3D all, +0x40 3D static.
    let (channels, _) = rc2k_core::audio::modes::channels(
        mode,
        [caps.get_u32(0x24), caps.get_u32(0x28)],
        [caps.get_u32(0x3C), caps.get_u32(0x40)],
    );
    audio.state.channel_count.set(channels);
    let ok = !sd::reopen_sound(&audio.state)? && crate::audio::primary::make_primary(&audio.state)?;
    if ok {
        open_way(audio, audio.state.sound_mode.get())?;
        crate::system::sleep::sleep(1000);
        crate::audio::thread_parts::play_primary(&audio.state);
        audio
            .state
            .opened_sound_mode
            .set(audio.state.sound_mode.get());
        sd::log_text(OK_TEXT, true);
    } else {
        audio.state.opened_sound_mode.set(0);
        sd::log_text(FAIL_TEXT, true);
    }
    crate::system::restore_window::restore_window();
    Ok(ok)
}
