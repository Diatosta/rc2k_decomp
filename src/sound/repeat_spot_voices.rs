//! `sound::repeat_spot_voices` (0x40EB44): each voice heard and not playing told when to
//! start again: a random delay of its kind and, while its repeats last, a restart.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! For each voice heard (+4 1) and not playing: its kind's record (`unk_517FAC`, 8 bytes
//! from kind 1: words +2 the most repeats, +4 and +6 the least and most delay): +0x30 =
//! the least + `math::random_below` (most - least) (as words; no call when they are
//! equal); then with no limit (+2 0), or the repeats (+0x2C) below it (counted up), the
//! restart flag +0x24 1. One caller, `sound::spot_sounds`.
//! Refuses (before anything) a voice of kind 0 or past 0x15.
//! Shadow mode: both (nothing sent; the random state put back between), compared.
//! Self-test: a copy whose call goes to a stand-in (the test's random numbers), against
//! the Rust version, on the game's voices (put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::sound::spot_voices::{MOST_KIND, REPEATS, VOICE_COUNT, voice, vw};
use crate::sound::turn_parts::{BELOW, Part, turn_entry};

// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::repeat`; each random number asked with the
/// registers the original has there.
pub fn repeat_spot_voices_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, store_voices};
    use rc2k_core::sound::spot_voices::{Repeats, repeat};
    let table = game_addr(rva(REPEATS)) as u32;
    let records: [[u8; 8]; MOST_KIND as usize] =
        unsafe { crate::framework::game_read(rva(REPEATS)) };
    let table_of = records.map(Repeats::from_bytes);
    let mut voices = load_voices();
    let mut c = *r;
    let done = repeat::<Refused>(&mut voices, &table_of, |n, range| {
        c.ebx = voice(n as u32);
        c.ecx = VOICE_COUNT - n as u32;
        c.edx = table + (voices_kind(n) - 1) * 8;
        c.eax = range;
        part(BELOW, &mut c)?;
        Ok(c.eax)
    });
    done?;
    store_voices(&voices);
    Ok(())
}

/// A voice's kind as the game holds it (read before the core changes anything of it).
fn voices_kind(n: usize) -> u32 {
    vw(voice(n as u32), 0x18)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, repeat_spot_voices_in)
}
