//! `audio::set_mode` (0x40A426): command 5, the sound mode changed (menus <-> a race).
//!
//! In:  eax = the new mode; ebx, ecx = handed to `audio::stop_streams`, with action 0 (the
//!      original hands on edx: section 16, fixed 2026-10-20)
//! Out: carry = failed ("SAM: FAIL"); every register preserved
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
//! Hook removed 2026-10-22; route 2026-10-23. Plain since 2026-10-23 (phase 3): its argument
//! is what it reads; the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::store::addr32 as a;
use crate::store::game_addr;

const SAM_TEXT: usize = rva(0x4DA6CD); // "SAM:"
const OK_TEXT: usize = rva(0x4DA6D2); // "SAM: OK"
const FAIL_TEXT: usize = rva(0x4DA6DA); // "SAM: FAIL"

#[derive(Clone, Copy)]
pub struct ModeVars {
    pub mode: u32,
    /// `g_soundCaps` (free hardware buffers: +0x24 all, +0x28 static, +0x3C 3D all,
    /// +0x40 3D static).
    pub caps: u32,
    pub channel_count: u32,
    pub opened: u32,
}

impl ModeVars {
    pub fn game() -> Self {
        Self {
            mode: a(crate::audio::voices::SOUND_MODE_RVA),
            caps: a(crate::audio::sound_device::CAPS_RVA),
            channel_count: a(crate::audio::voices::CHANNEL_COUNT_RVA),
            opened: a(crate::audio::sound_device::OPENED_MODE_RVA),
        }
    }
}

/// True for no carry.
pub fn set_mode(mode: u32, v: &ModeVars) -> Result<bool, Refused> {
    use crate::audio::mixer_set_up::{close_way, open_way};
    use crate::audio::sound_device as sd;
    use crate::audio::voices::*;
    unsafe {
        sd::log_text(game_addr(SAM_TEXT) as u32, false);
        sd::log_hex(mode, true);
        // Fixed (section 16, 2026-10-20): the streams stopped with action 0; the original
        // hands on whatever edx holds (normally the thread's 1, a feed), so the music
        // outlived the DirectSound it played on.
        crate::audio::stop_streams::stop_streams()?;
        crate::audio::free_voices::free_voices(
            &World::game(),
            &crate::audio::free_voices::game_vars(),
        )?;
        close_way(read(v.mode))?;
        sd::release_primary(
            game_addr(PRIMARY_RVA) as u32,
            game_addr(sd::VOLUME_KEPT_RVA) as u32,
        )?;
        write(v.mode, mode);
        // Converted: `rc2k_core::audio::modes::channels`.
        let (channels, _) = rc2k_core::audio::modes::channels(
            mode,
            [read(v.caps + 0x24), read(v.caps + 0x28)],
            [read(v.caps + 0x3C), read(v.caps + 0x40)],
        );
        write(v.channel_count, channels);
        let ok = !sd::reopen_sound(v.mode, v.opened)?
            && crate::audio::primary::make_primary(&crate::audio::primary::PrimaryVars::game())?;
        if ok {
            open_way(read(v.mode))?;
            crate::system::sleep::sleep(1000);
            crate::audio::thread_parts::play_primary(
                game_addr(PRIMARY_RVA) as u32,
                game_addr(BUFFER_LOST_RVA) as u32,
            );
            write(v.opened, read(v.mode));
            sd::log_text(game_addr(OK_TEXT) as u32, true);
        } else {
            write(v.opened, 0);
            sd::log_text(game_addr(FAIL_TEXT) as u32, true);
        }
        crate::system::restore_window::restore_window();
        Ok(ok)
    }
}
