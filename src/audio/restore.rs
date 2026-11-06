//! Lost buffers restored, and the mixer's primary buffer closed (IDA, 2026-10-15):
//! `audio::restore_buffer`, `replay_looping`, `restore_lost`, `close_primary_mixer`. Their
//! calls go through a `Part` (the place, eax, esi -> the carry), so that the tests can
//! stand them in; in the game they are the Rust versions.

use crate::errors::{BadInput, Refused};

/// `restore_buffer` of the buffer `buffer`: true when it failed (the carry).
fn restore_failed(buffer: u32) -> Result<bool, Refused> {
    Ok(restore_buffer(buffer)?.carry)
}

/// The voice `n`'s buffer refilled: true when it failed.
fn refill_failed(n: usize) -> Result<bool, Refused> {
    crate::audio::fill_buffer::fill_buffer(&crate::audio::owner::audio().state.voices[n].get())
}

/// `audio::make_primary` on the game's variables: true when it failed (the carry).
pub fn make_primary_failed() -> Result<bool, Refused> {
    Ok(!crate::audio::primary::make_primary()?)
}

/// What `restore_buffer` leaves: the carry, and eax and edx where it changes them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Restored {
    pub carry: bool,
    pub eax: Option<u32>,
    pub edx: Option<u32>,
}

/// `audio::restore_buffer`: the buffer `b`, if any: `GetStatus` (+0x24, into edx, which
/// `Restore` then leaves as it likes); lost: `Restore` (+0x50, into eax), whose failure sets
/// the lost flag and the carry. Refuses A3D (0x40CB0A).
pub fn restore_buffer(b: u32) -> Result<Restored, Refused> {
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D (0x40CB0A)").into());
    }
    let mut left = Restored {
        carry: false,
        eax: None,
        edx: None,
    };
    {
        if b == 0 {
            return Ok(left);
        }
        // A failed call leaves the status 0.
        let status = crate::platform_sound::status(b).unwrap_or(0);
        left.edx = Some(status);
        // Converted: `rc2k_core::audio::buffers::needs_restore`.
        if !rc2k_core::audio::buffers::needs_restore(status) {
            return Ok(left);
        }
        let r = crate::platform_sound::restore(b);
        left.eax = Some(r as u32);
        // `Restore` may change edx as it likes (as any COM call): not the status any more.
        left.edx = None;
        if r != 0 {
            crate::audio::owner::audio().state.sound_buffer_lost.set(1);
            left.carry = true;
        }
    }
    Ok(left)
}

/// `audio::replay_looping`: each playing voice (the list at `g_playingVoices`, linked at
/// +0x48) whose +0x3C is 2 and that has a buffer (+8) played again (`play_buffer`).
pub fn replay_looping() -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::voices::sounding_to_replay` (the list is walked before
    // any is played, so a list without its end plays none).
    let lists = rc2k_core::audio::voices::Lists {
        playing: crate::audio::owner::audio().state.playing_voices.get(),
        ..Default::default()
    };
    let mut pool = crate::audio::voices::GamePool;
    for v in rc2k_core::audio::voices::sounding_to_replay(&mut pool, &lists)? {
        crate::audio::buffer_play::play_buffer_in_game(&crate::audio::voices::voice(v))?;
    }
    Ok(())
}

/// `audio::restore_lost`: a second's wait; with the window active: the flag cleared, the
/// primary buffer and each voice's buffer restored (and the voice's refilled), the primary
/// played, the looping voices replayed. A failure ends it. Refuses A3D (0x40CB0A).
pub fn restore_lost() -> Result<(), Refused> {
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D (0x40CB0A)").into());
    }
    crate::system::sleep::sleep(1000);
    if crate::system::owner::system()
        .session
        .flags
        .window_active
        .get()
        == 0
    {
        return Ok(());
    }
    crate::audio::owner::audio().state.sound_buffer_lost.set(0);
    if restore_failed(crate::audio::owner::audio().state.primary_buffer.get())? {
        return Ok(());
    }
    for n in 0..crate::audio::owner::audio().state.voice_count.get() as usize {
        if restore_failed(crate::audio::owner::audio().state.voices[n].get().buffer)?
            || refill_failed(n)?
        {
            return Ok(());
        }
    }
    crate::audio::thread_parts::play_primary();
    replay_looping()
}

/// `audio::close_primary_mixer` on the game's variables: the channels cleared; in the
/// mixer's mode with a DirectSound: `release_primary`, cooperative level 3, `make_primary`.
/// The carry it leaves: `make_primary`'s, when that ran.
pub fn close_primary_mixer_in_game() -> Result<bool, Refused> {
    crate::audio::clear_channels::clear_channels();
    // Converted: `rc2k_core::audio::buffers::remakes_primary`.
    if !rc2k_core::audio::buffers::remakes_primary(
        crate::audio::owner::audio().state.sound_mode.get(),
        || crate::audio::owner::audio().state.direct_sound.get(),
    ) {
        return Ok(false);
    }
    crate::audio::sound_device::release_primary()?;
    let ds = crate::audio::owner::audio().state.direct_sound.get();
    crate::platform_sound::set_cooperative_level(
        ds,
        crate::system::owner::system().session.machine.window.get(),
        3,
    );
    make_primary_failed()
}
