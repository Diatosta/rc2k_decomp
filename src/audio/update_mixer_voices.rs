//! `audio::update_mixer_voices` (0x40BC05, a chunk of 0x40AC77 entered by `jnz`): the
//! turn's update of the voices on the game's own mixer.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The playing list: a voice with dirty bits gets, on its channel, as marked: the step
//! (frequency `>>` the halvings, over `g_mixRate`, as 32.32) for 4; the left and right
//! volumes for 1 or 2 (volume x factor, times 64 - pan and 64 for a pan of 0 and up, 64 and
//! 64 + pan below, each `>> 6`; with mode bit 0x2000000 and a negative pan sign the right
//! negated); for a restart (0x20) or a voice not yet sounding, the channel's position and
//! fraction 0 and its format, data, length (the voice's frames) and kind, the voice's
//! state 2; dirty bits 0. A voice that does not loop whose channel is no longer used (the
//! mixer ended it) has its channel freed and 0, goes to the stopping list's front, and
//! `g_channelsUsed` is one less. The stopping list: each voice's dirty bits + 0xF, state
//! 0, channel freed and 0, and the list emptied. `audio::mix`. The playing list again for
//! the voices the mix ended.
//! Refuses, before anything: lists without their end; a playing voice without a channel; no
//! mixing rate with a frequency to set (the original divides by 0); and a voice whose
//! volume or pan changes while its sound has ended: the original's ebx (the list's link)
//! then holds the pan, and it writes the link through it (section 16).
//! Shadow mode runs the original alone. Self-test: a redirected copy of the chunk whose
//! variables are the test's and whose mix notes itself, 200 states by lot.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::mixer::MIX_RATE_RVA;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::store::addr32 as a;
use rc2k_core::audio::voices::Lists;

#[derive(Clone, Copy)]
pub struct Vars {
    pub rate: u32,
    pub mode: u32,
    pub playing: u32,
    pub stopping: u32,
    pub used: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            rate: a(MIX_RATE_RVA),
            mode: a(SOUND_MODE_RVA),
            playing: a(PLAYING_RVA),
            stopping: a(STOPPING_RVA),
            used: a(CHANNELS_USED_RVA),
        }
    }
}

unsafe fn lists(v: &Vars) -> Lists {
    unsafe {
        Lists {
            playing: read(v.playing),
            stopping: read(v.stopping),
            channels_used: read(v.used),
            channel_count: 0,
        }
    }
}

unsafe fn store(v: &Vars, l: &Lists) {
    unsafe {
        write(v.playing, l.playing);
        write(v.stopping, l.stopping);
        write(v.used, l.channels_used);
    }
}

/// `mix`: `audio::mix`.
pub unsafe fn update_mixer_voices(
    v: &Vars,
    mix: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::mixer_voices`; the lists stored for the mix, which
    // reads them, and read again after it.
    use rc2k_core::audio::mixer_voices::{after_mix, before_mix};
    unsafe {
        let rate = read(v.rate);
        let mode = read(v.mode);
        let mut pool = GamePool::new(&World::game());
        let mut l = lists(v);
        before_mix(&mut pool, &mut l, rate, mode)?;
        store(v, &l);
        mix()?;
        let mut l = lists(v);
        after_mix(&mut pool, &mut l, rate, mode)?;
        store(v, &l);
    }
    Ok(())
}

/// The voices' turn of the game's own mixer, the mix (`audio::mix`) in it.
pub fn update_mixer_voices_in_game() -> Result<(), Refused> {
    let mut mix = || -> Result<(), Refused> {
        unsafe { crate::audio::mix::mix(&crate::audio::mix::Vars::game()) }?;
        Ok(())
    };
    unsafe { update_mixer_voices(&Vars::game(), &mut mix) }
}
