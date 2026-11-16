//! Command 5: the sound mode changed (menus <-> a race). "SAM:" and the mode are logged; streams
//! stopped, voices freed, the way closed, the primary released; the channels: 8 (0x10 in the
//! mixer mode), held to the free hardware buffers of the mode's kind when there are any, and at
//! least 6; DirectSound reopened, the primary made (either failing: opened mode 0, "SAM: FAIL"),
//! the way opened, a second's sleep, the primary played; "SAM: OK". Either way the window is
//! restored last.

use crate::audio::own::AudioThread;
use crate::errors::Refused;

const SAM_TEXT: &std::ffi::CStr = c"SAM:";
const OK_TEXT: &std::ffi::CStr = c"SAM: OK";
const FAIL_TEXT: &std::ffi::CStr = c"SAM: FAIL";

/// True when it worked.
pub fn set_mode(
    audio: &AudioThread,
    system: &'static crate::system::owner::System,
    mode: u32,
) -> Result<bool, Refused> {
    use crate::audio::mixer_set_up::{close_way, open_way};
    use crate::audio::sound_device as sd;
    let caps = &audio.state.sound_caps;
    sd::log_text(SAM_TEXT, false);
    sd::log_hex(mode, true);
    // Fixed: the streams are always stopped here ("Changing the sound mode leaves the music
    // playing", `wiki/Fixed-bugs.md`).
    crate::audio::stop_streams::stop_streams(audio)?;
    crate::audio::free_voices::free_voices(audio)?;
    close_way(audio, system, audio.state.sound_mode.get())?;
    sd::release_primary(&audio.state);
    audio.state.sound_mode.set(mode);
    // The sound caps: free hardware buffers +0x24 all, +0x28 static, +0x3C 3D all, +0x40 3D static.
    let (channels, _) = rc2k_core::audio::modes::channels(
        mode,
        [caps.get_u32(0x24), caps.get_u32(0x28)],
        [caps.get_u32(0x3C), caps.get_u32(0x40)],
    );
    audio.state.channel_count.set(channels);
    let ok = !sd::reopen_sound(&audio.state, system)?
        && crate::audio::primary::make_primary(audio, system)?;
    if ok {
        open_way(audio, system, audio.state.sound_mode.get())?;
        // No pause before playing (the original waited a second for the sound card of its
        // day; every start and every change of the sound mode waited on it).
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
