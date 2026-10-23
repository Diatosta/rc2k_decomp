//! `audio::load_sounds` (0x40AEB9): command 3: a list of sounds loaded.
//!
//! In:  eax = the list (the sender's eax): sound numbers, -2 passed over, ended by -1
//! Out: every register preserved
//!
//! For each sound whose record has a file (`data_key`) and `copies` above 0 (signed), that
//! many voices while fewer than 0x200 are in use: `audio::find_shared_voice`; none:
//! `audio::read_sample_of` (a carry ends the sound's voices); `audio::new_voice` (the
//! record's first voice if it has none); `audio::fill_voice` with the shared voice;
//! `audio::sample_mode` on `g_soundMode`; `audio::prepare_sample`; `audio::keep_sample`;
//! `audio::make_buffer` and `audio::fill_buffer` (their carries not looked at); the
//! voice's flags get `LOADED`. After each entry of the list, `g_quitting` 1 ends it. The
//! callees get the registers the original has at each call (ebx the record, esi the voice,
//! eax the shared voice or the mode): `make_buffer` hands ebx on to the EAX set-up. One
//! caller, `audio::take_commands`. Plain since 2026-10-23 (phase 3): each callee gets
//! what it reads.
//! Refuses, before anything is done: a list without its -1 within 0x1000 entries, or with
//! a sound of 0x1CD or more; A3D.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose variables are the
//! test's and whose nine calls follow a script and note their registers, 40 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};

pub const QUITTING_RVA: usize = rva(0x51837C); // g_quitting
/// Its variables: the records, the voices in use, the mode, the quit switch.
#[derive(Clone, Copy)]
pub struct Vars {
    pub samples: u32,
    pub voice_count: u32,
    pub mode: u32,
    pub quitting: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            samples: a(SAMPLES_RVA),
            voice_count: a(VOICE_COUNT_RVA),
            mode: a(SOUND_MODE_RVA),
            quitting: a(QUITTING_RVA),
        }
    }
}

/// The core's loader: each step the Rust version of the original's callee, with what it
/// reads (the record, the voice, the shared voice or the mode; `make_buffer` the record as
/// its ebx, which the EAX set-up reads through).
struct GameLoader<'a> {
    v: &'a Vars,
}

impl GameLoader<'_> {
    fn record_at(&self, sound: u32) -> u32 {
        self.v.samples + sound * 0x40
    }
}

impl rc2k_core::audio::load::Loader for GameLoader<'_> {
    type Error = Refused;
    fn record(&mut self, sound: u32) -> &mut SampleRecord {
        unsafe { &mut *(self.record_at(sound) as *mut SampleRecord) }
    }
    fn full(&mut self) -> bool {
        unsafe { read(self.v.voice_count) == VOICES }
    }
    fn shared(&mut self, at: rc2k_core::audio::load::At) -> Result<u32, Refused> {
        let count: u32 = unsafe { crate::framework::game_read(VOICE_COUNT_RVA) };
        Ok(unsafe {
            crate::audio::samples::find_shared_voice(
                &*(self.record_at(at.sound) as *const SampleRecord),
                game_addr(VOICES_RVA) as u32,
                count.min(VOICES),
            )
        })
    }
    fn read(&mut self, at: rc2k_core::audio::load::At) -> Result<bool, Refused> {
        use crate::audio::{read_sample_of, samples};
        unsafe {
            read_sample_of::read_sample_of(
                &*(self.record_at(at.sound) as *const SampleRecord),
                &mut |p| samples::read_sample_in_game(p as *const u8),
            )
        }
    }
    fn new_voice(&mut self, _: rc2k_core::audio::load::At, _: u32) -> Result<u32, Refused> {
        unsafe { crate::audio::new_voice::new_voice_in_game() }
    }
    fn make_ready(
        &mut self,
        at: rc2k_core::audio::load::At,
        voice_at: u32,
        shared: u32,
    ) -> Result<(), Refused> {
        use crate::audio::*;
        let record = self.record_at(at.sound);
        unsafe {
            fill_voice::fill_voice_in_game(voice_at, record, shared)?;
            let mode = sample_mode::sample_mode_of(read(self.v.mode), voice(voice_at).sound)?;
            samples::prepare_sample(voice(voice_at), mode)?;
            keep_sample::keep_sample(voice(voice_at), &mut |n| keep_sample::allocate_in_game(n))?;
            make_buffer::make_buffer(voice_at, record, &make_buffer::Vars::game())?;
            let parts = game_addr(fill_buffer::LOCKED_PARTS_RVA) as u32;
            let lost = game_addr(BUFFER_LOST_RVA) as u32;
            fill_buffer::fill_buffer(voice(voice_at), parts, lost)?;
        }
        Ok(())
    }
    fn mark_loaded(&mut self, voice: u32) {
        unsafe { crate::audio::voices::voice(voice).flags |= LOADED };
    }
    fn quitting(&mut self) -> bool {
        unsafe { read(self.v.quitting) == 1 }
    }
}

/// Converted: `rc2k_core::audio::load` (`entries`, `load`).
pub unsafe fn load_sounds(list: u32, v: &Vars) -> Result<(), Refused> {
    if list == 0 {
        return Ok(());
    }
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D").into());
    }
    let entries = rc2k_core::audio::load::entries(|n| unsafe { read(list + n as u32 * 4) })?;
    let mut l = GameLoader { v };
    rc2k_core::audio::load::load(&mut l, &entries)
}
