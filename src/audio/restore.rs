//! Lost buffers restored, and the mixer's primary buffer closed (IDA, 2026-10-15):
//! `audio::restore_buffer`, `replay_looping`, `restore_lost`, `close_primary_mixer`. Their
//! calls go through a `Part` (the place, eax, esi -> the carry), so that the tests can
//! stand them in.

use crate::audio::owner::Audio;
use crate::audio::store::AudioState;
use crate::errors::{BadInput, Refused};
use crate::system::owner::system;

/// `restore_buffer` of the buffer `buffer`: true when it failed (the carry).
fn restore_failed(state: &AudioState, buffer: u32) -> Result<bool, Refused> {
    Ok(restore_buffer(state, buffer)?.carry)
}

/// The voice `n`'s buffer refilled: true when it failed.
fn refill_failed(state: &AudioState, n: usize) -> Result<bool, Refused> {
    crate::audio::fill_buffer::fill_buffer(state, &state.voices[n].get())
}

/// `audio::make_primary` on the game's variables: true when it failed (the carry).
pub fn make_primary_failed(state: &AudioState) -> Result<bool, Refused> {
    Ok(!crate::audio::primary::make_primary(state)?)
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
pub fn restore_buffer(state: &AudioState, b: u32) -> Result<Restored, Refused> {
    if crate::audio::buffer_settings::a3d(state) {
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
            state.sound_buffer_lost.set(1);
            left.carry = true;
        }
    }
    Ok(left)
}

/// `audio::replay_looping`: each playing voice (the list at `g_playingVoices`, linked at
/// +0x48) whose +0x3C is 2 and that has a buffer (+8) played again (`play_buffer`).
pub fn replay_looping(audio: &Audio) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::voices::sounding_to_replay` (the list is walked before
    // any is played, so a list without its end plays none).
    let lists = rc2k_core::audio::voices::Lists {
        playing: audio.state.playing_voices.get(),
        ..Default::default()
    };
    let mut pool = crate::audio::voices::GamePool::new(audio);
    for v in rc2k_core::audio::voices::sounding_to_replay(&mut pool, &lists)? {
        crate::audio::buffer_play::play_buffer(
            &audio.state,
            &crate::audio::voices::voice_of(&audio.state, v),
        )?;
    }
    Ok(())
}

/// `audio::restore_lost`: a second's wait; with the window active: the flag cleared, the
/// primary buffer and each voice's buffer restored (and the voice's refilled), the primary
/// played, the looping voices replayed. A failure ends it. Refuses A3D (0x40CB0A).
pub fn restore_lost(audio: &Audio) -> Result<(), Refused> {
    let state = &audio.state;
    if crate::audio::buffer_settings::a3d(state) {
        return Err(BadInput("A3D (0x40CB0A)").into());
    }
    crate::system::sleep::sleep(1000);
    if system().session.flags.window_active.get() == 0 {
        return Ok(());
    }
    state.sound_buffer_lost.set(0);
    if restore_failed(state, state.primary_buffer.get())? {
        return Ok(());
    }
    for n in 0..state.voice_count.get() as usize {
        if restore_failed(state, state.voices[n].get().buffer)? || refill_failed(state, n)? {
            return Ok(());
        }
    }
    crate::audio::thread_parts::play_primary(state);
    replay_looping(audio)
}

/// `audio::close_primary_mixer` on the game's variables: the channels cleared; in the
/// mixer's mode with a DirectSound: `release_primary`, cooperative level 3, `make_primary`.
/// The carry it leaves: `make_primary`'s, when that ran.
pub fn close_primary_mixer(audio: &Audio) -> Result<bool, Refused> {
    let state = &audio.state;
    crate::audio::clear_channels::clear_channels(audio);
    // Converted: `rc2k_core::audio::buffers::remakes_primary`.
    if !rc2k_core::audio::buffers::remakes_primary(state.sound_mode.get(), || {
        state.direct_sound.get()
    }) {
        return Ok(false);
    }
    crate::audio::sound_device::release_primary(state);
    let ds = state.direct_sound.get();
    crate::platform_sound::set_cooperative_level(ds, system().session.machine.window.get(), 3);
    make_primary_failed(state)
}
