//! `physics::cue_car_sound` (0x40ECE7): for eax 0 or 1 (unsigned), the sound object of
//! car `dword_514F70[eax]` (`g_raceSoundObjects`); when its +8 is 1,
//! `physics::queue_sound_cue` with eax 0 (first), edi its +0x298, the cue in ebx. All in
//! `pusha` / `popa`.
//!
//! In:  eax which, ebx the cue
//! Out: every register kept; no flags read
//!
//! Caller: `physics::cue_listed_car_sound`. A short jump in its first five bytes: hooked
//! by a private copy (its call relocated) and a hand-made patch.
//! Refuses (before anything) a car past 7 or a null sound object.
//! Shadow mode runs the original alone (`physics/sound_cues.rs`).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version: which 0, 1,
//! 2, objects ready or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::physics::sound_cues::{Part, QUEUE, cue_entry};
use rc2k_core::physics::listed_cars::{CuePlace, QUEUE_OFFSET, cue_object};

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The steps on `r` (its registers kept).
// Converted: `rc2k_core::physics::listed_cars::cue_object`.
pub fn cue_car_sound_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let Some(o) = cue_object(
        r.eax,
        |which| g(0x514F70 + 4 * which as usize),
        |car| g(0x512A10 + 4 * car as usize),
        |o| unsafe { ((o + 8) as *const u32).read_unaligned() },
    )?
    else {
        return Ok(());
    };
    let mut c = *r;
    c.eax = CuePlace::First as u32;
    c.esi = o;
    c.edi = o + QUEUE_OFFSET;
    part(QUEUE, &mut c)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    cue_entry(r, cue_car_sound_in)
}
