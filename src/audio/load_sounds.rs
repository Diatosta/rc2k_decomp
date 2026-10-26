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

use crate::audio::store::{AUDIO, SAMPLE_RECORDS};
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::session::FLAGS;

/// The core's loader: each step the Rust version of the original's callee, with what it
/// reads (the record, the voice, the shared voice or the mode; `make_buffer` the record as
/// its ebx, which the EAX set-up reads through).
struct GameLoader;

impl GameLoader {
    /// The record's place, which the callees take for their ebx.
    fn record_at(&self, sound: u32) -> u32 {
        SAMPLE_RECORDS.records[sound as usize].addr()
    }
}

impl rc2k_core::audio::load::Loader for GameLoader {
    type Error = Refused;
    fn record(&mut self, sound: u32) -> &mut SampleRecord {
        unsafe { SAMPLE_RECORDS.records[sound as usize].borrow() }
    }
    fn full(&mut self) -> bool {
        AUDIO.voice_count.get() == VOICES
    }
    fn shared(&mut self, at: rc2k_core::audio::load::At) -> Result<u32, Refused> {
        let count = AUDIO.voice_count.get();
        Ok({
            crate::audio::samples::find_shared_voice(
                &SAMPLE_RECORDS.records[at.sound as usize].get(),
                count.min(VOICES),
            )
        })
    }
    fn read(&mut self, at: rc2k_core::audio::load::At) -> Result<bool, Refused> {
        use crate::audio::{read_sample_of, samples};
        unsafe {
            read_sample_of::read_sample_of(
                &SAMPLE_RECORDS.records[at.sound as usize].get(),
                &mut |p| samples::read_sample_in_game(crate::pointers::host(p) as *const u8),
            )
        }
    }
    fn new_voice(&mut self, _: rc2k_core::audio::load::At, _: u32) -> Result<u32, Refused> {
        crate::audio::new_voice::new_voice_in_game()
    }
    fn make_ready(
        &mut self,
        at: rc2k_core::audio::load::At,
        voice_at: u32,
        shared: u32,
    ) -> Result<(), Refused> {
        use crate::audio::*;
        let record = self.record_at(at.sound);
        fill_voice::fill_voice_in_game(voice_at, record, shared)?;
        let mode = sample_mode::sample_mode_of(AUDIO.sound_mode.get(), voice(voice_at).sound)?;
        samples::prepare_sample(voice(voice_at), mode)?;
        keep_sample::keep_sample(voice(voice_at), &mut |n| keep_sample::allocate_in_game(n))?;
        make_buffer::make_buffer(voice_at, record)?;
        fill_buffer::fill_buffer(voice(voice_at))?;

        Ok(())
    }
    fn mark_loaded(&mut self, voice: u32) {
        crate::audio::voices::voice(voice).flags |= LOADED;
    }
    fn quitting(&mut self) -> bool {
        FLAGS.quitting.get() == 1
    }
}

/// Converted: `rc2k_core::audio::load` (`entries`, `load`).
pub fn load_sounds(list: u32) -> Result<(), Refused> {
    if list == 0 {
        return Ok(());
    }
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D").into());
    }
    let entries = rc2k_core::audio::load::entries(|n| read(list + n as u32 * 4))?;
    let mut l = GameLoader;
    rc2k_core::audio::load::load(&mut l, &entries)
}
