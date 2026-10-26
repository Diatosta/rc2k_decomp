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
//! caller, `audio::take_commands`.
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
use crate::framework::{Regs, game_addr, rva};

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

/// Its nine callees, in the order of the doc above (0 `find_shared_voice` .. 8
/// `fill_buffer`), on the registers as the original has them; true: the carry they leave.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

/// The core's loader: each step through `part` with the registers the original has
/// there (ebx the record, ecx the voices left, edx past the entry; esi the voice; eax the
/// shared voice or the mode).
struct GameLoader<'a, 'b> {
    list: u32,
    v: &'a Vars,
    part: &'a mut Part<'b>,
}

impl GameLoader<'_, '_> {
    fn base(&self, at: rc2k_core::audio::load::At) -> Regs {
        Regs {
            ebx: self.v.samples + at.sound * 0x40,
            ecx: at.left,
            edx: self.list + (at.entry as u32 + 1) * 4,
            ..Regs::default()
        }
    }
}

impl rc2k_core::audio::load::Loader for GameLoader<'_, '_> {
    type Error = Refused;
    fn record(&mut self, sound: u32) -> &mut SampleRecord {
        unsafe { &mut *((self.v.samples + sound * 0x40) as *mut SampleRecord) }
    }
    fn full(&mut self) -> bool {
        unsafe { read(self.v.voice_count) == VOICES }
    }
    fn shared(&mut self, at: rc2k_core::audio::load::At) -> Result<u32, Refused> {
        let mut r = self.base(at);
        (self.part)(0, &mut r)?;
        Ok(r.eax)
    }
    fn read(&mut self, at: rc2k_core::audio::load::At) -> Result<bool, Refused> {
        let mut r = self.base(at);
        Ok(!(self.part)(1, &mut r)?)
    }
    fn new_voice(&mut self, at: rc2k_core::audio::load::At, shared: u32) -> Result<u32, Refused> {
        let mut r = Regs {
            eax: shared,
            ..self.base(at)
        };
        (self.part)(2, &mut r)?;
        Ok(r.esi)
    }
    fn make_ready(
        &mut self,
        at: rc2k_core::audio::load::At,
        voice: u32,
        shared: u32,
    ) -> Result<(), Refused> {
        let base = self.base(at);
        let with = |eax| Regs {
            eax,
            esi: voice,
            ..base
        };
        (self.part)(3, &mut with(shared))?;
        let mut r = with(unsafe { read(self.v.mode) });
        (self.part)(4, &mut r)?;
        let mode = r.eax;
        for p in 5..=8 {
            (self.part)(p, &mut with(mode))?;
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
pub unsafe fn load_sounds(list: u32, v: &Vars, part: Part) -> Result<(), Refused> {
    if list == 0 {
        return Ok(());
    }
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D").into());
    }
    let entries = rc2k_core::audio::load::entries(|n| unsafe { read(list + n as u32 * 4) })?;
    let mut part = part;
    let mut l = GameLoader {
        list,
        v,
        part: &mut part,
    };
    rc2k_core::audio::load::load(&mut l, &entries)
}

pub fn game_part(which: u32, r: &mut Regs) -> Result<bool, Refused> {
    use crate::audio::*;
    unsafe {
        let record = r.ebx;
        match which {
            0 => {
                let count: u32 = crate::framework::game_read(VOICE_COUNT_RVA);
                r.eax = samples::find_shared_voice(
                    &*(record as *const SampleRecord),
                    game_addr(VOICES_RVA) as u32,
                    count.min(VOICES),
                );
            }
            1 => {
                let found =
                    read_sample_of::read_sample_of(&*(record as *const SampleRecord), &mut |p| {
                        samples::read_sample_in_game(p as *const u8)
                    })?;
                return Ok(!found);
            }
            2 => r.esi = new_voice::new_voice_in_game()?,
            3 => fill_voice::fill_voice_in_game(r.esi, record, r.eax)?,
            4 => r.eax = sample_mode::sample_mode_of(r.eax, voice(r.esi).sound)?,
            5 => samples::prepare_sample(voice(r.esi), r.eax)?,
            6 => keep_sample::keep_sample(voice(r.esi), &mut |n| keep_sample::allocate_in_game(n))?,
            7 => {
                let vars = make_buffer::Vars::game();
                let made = make_buffer::make_buffer(r.esi, r.ebx, &vars, &mut |p, r| {
                    make_buffer::game_part(p, r)
                })?;
                return Ok(!made);
            }
            _ => {
                let parts = game_addr(fill_buffer::LOCKED_PARTS_RVA) as u32;
                let lost = game_addr(BUFFER_LOST_RVA) as u32;
                return Ok(!fill_buffer::fill_buffer(voice(r.esi), parts, lost)?);
            }
        }
    }
    Ok(false)
}
