//! `audio::close_all` (0x409DD5): everything of the sound closed (the audio thread's end).
//!
//! In:  ebx, ecx, edx = handed to `audio::stop_streams` (edx its action)
//! Out: every register preserved
//!
//! `audio::stop_streams`, `audio::free_voices`, `audio::close_way`,
//! `audio::release_primary`, `audio::close_sound`, then `g_soundMode` 0. One caller, the
//! audio thread.
//!
//! Plain since 2026-10-23 (phase 3). Fixed (section 16, 2026-10-23): the streams are
//! stopped (action 0); the original hands on the thread's edx, 1 (a feed), so the music
//! and the stream were left playing on the DirectSound it then closes.

use crate::audio::owner::audio;
use crate::errors::Refused;

pub fn close_all() -> Result<(), Refused> {
    let state = &audio().state;
    use crate::audio::sound_device::*;
    crate::audio::stop_streams::stop_streams()?;
    crate::audio::free_voices::free_voices()?;
    crate::audio::mixer_set_up::close_way(state.sound_mode.get())?;
    release_primary()?;
    close_sound()?;
    state.sound_mode.set(0);
    Ok(())
}
