//! Lost buffers restored, and the mixer's primary buffer closed: `audio::restore_buffer`,
//! `replay_looping`, `restore_lost`, `close_primary_mixer`. Their calls go through a `Part`, so
//! that the tests can stand them in.

use crate::audio::own::{AudioOwn, AudioThread};
use crate::errors::{BadInput, Refused};
use crate::system::owner::System;

/// `restore_buffer` of the buffer `buffer`: true when it failed.
fn restore_failed(state: &AudioOwn, buffer: u32) -> Result<bool, Refused> {
    Ok(restore_buffer(state, buffer)?.carry)
}

/// The voice `n`'s buffer refilled: true when it failed.
fn refill_failed(state: &AudioOwn, n: usize) -> Result<bool, Refused> {
    crate::audio::fill_buffer::fill_buffer(state, &state.voices[n].get())
}

/// `audio::make_primary`: true when it failed.
pub fn make_primary_failed(audio: &AudioThread, system: &System) -> Result<bool, Refused> {
    Ok(!crate::audio::primary::make_primary(audio, system)?)
}

/// What `restore_buffer` leaves: failure, and the status and the `Restore` result where it changes
/// them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Restored {
    pub carry: bool,
    pub result: Option<u32>,
    pub status: Option<u32>,
}

/// The buffer `b`, if any: its status is read; lost: `Restore`, whose failure sets the lost flag
/// and fails. Refuses A3D.
pub fn restore_buffer(state: &AudioOwn, b: u32) -> Result<Restored, Refused> {
    if crate::audio::buffer_settings::a3d(state) {
        return Err(BadInput("A3D (0x40CB0A)").into());
    }
    let mut left = Restored {
        carry: false,
        result: None,
        status: None,
    };
    {
        if b == 0 {
            return Ok(left);
        }
        // A failed call leaves the status 0.
        let status = crate::platform::sound::status(b).unwrap_or(0);
        left.status = Some(status);
        if !rc2k_core::audio::buffers::needs_restore(status) {
            return Ok(left);
        }
        let r = crate::platform::sound::restore(b);
        left.result = Some(r as u32);
        // After `Restore` the status is no longer known (as after any COM call).
        left.status = None;
        if r != 0 {
            state.sound_buffer_lost.set(1);
            left.carry = true;
        }
    }
    Ok(left)
}

/// Each playing voice that has a buffer and is looping is played again.
pub fn replay_looping(audio: &AudioThread) -> Result<(), Refused> {
    // The list is walked before any is played, so a list without its end plays none.
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

/// A second's wait; with the window active: the flag cleared, the primary buffer and each voice's
/// buffer restored (and the voice's refilled), the primary played, the looping voices replayed. A
/// failure ends it. Refuses A3D.
pub fn restore_lost(audio: &AudioThread, system: &System) -> Result<(), Refused> {
    let state = &audio.state;
    if crate::audio::buffer_settings::a3d(state) {
        return Err(BadInput("A3D (0x40CB0A)").into());
    }
    crate::system::sleep::sleep(1000);
    if system.session.flags.window_active.get() == 0 {
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

/// The channels cleared; in the mixer's mode with a DirectSound: `release_primary`, cooperative
/// level 3, `make_primary`. Returns failure of `make_primary`, when that ran.
pub fn close_primary_mixer(audio: &AudioThread, system: &System) -> Result<bool, Refused> {
    let state = &audio.state;
    crate::audio::clear_channels::clear_channels(audio);
    if !rc2k_core::audio::buffers::remakes_primary(state.sound_mode.get(), || {
        state.direct_sound.get()
    }) {
        return Ok(false);
    }
    crate::audio::sound_device::release_primary(state);
    let ds = state.direct_sound.get();
    crate::platform::sound::set_cooperative_level(ds, system.session.host.window.get(), 3);
    make_primary_failed(audio, system)
}
