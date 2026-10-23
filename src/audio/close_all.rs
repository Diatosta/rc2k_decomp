//! `audio::close_all` (0x409DD5): everything of the sound closed (the audio thread's end).
//!
//! In:  ebx, ecx, edx = handed to `audio::stop_streams` (edx its action)
//! Out: every register preserved
//!
//! `audio::stop_streams`, `audio::free_voices`, `audio::close_way`,
//! `audio::release_primary`, `audio::close_sound`, then `g_soundMode` 0. One caller, the
//! audio thread. Shadow mode runs the original alone. Self-test: a redirected copy whose
//! five calls go to stand-ins (the first noting ebx, ecx, edx), 2 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//!
//! Plain since 2026-10-23 (phase 3). Fixed (section 16, 2026-10-23): the streams are
//! stopped (action 0); the original hands on the thread's edx, 1 (a feed), so the music
//! and the stream were left playing on the DirectSound it then closes.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read};

pub fn close_all() -> Result<(), Refused> {
    use crate::audio::sound_device::*;
    use crate::audio::voices::*;
    crate::audio::stop_streams::stop_streams()?;
    unsafe {
        crate::audio::free_voices::free_voices(
            &World::game(),
            &crate::audio::free_voices::game_vars(),
        )?;
        crate::audio::mixer_set_up::close_way(game_read(SOUND_MODE_RVA))?;
        release_primary(
            game_addr(PRIMARY_RVA) as u32,
            game_addr(VOLUME_KEPT_RVA) as u32,
        )?;
        close_sound(
            game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
            game_addr(WINDOW_RVA) as u32,
        )?;
        write(game_addr(SOUND_MODE_RVA) as u32, 0);
    }
    Ok(())
}
