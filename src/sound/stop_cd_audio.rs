//! `sound::stop_cd_audio` (0x40FCAA): `dword_518110` (the track playing) 0 and the CD
//! audio stopped five times: three calls of `sound::stop_cd_track`, then `call $+5` into
//! it, whose return lands on it once more, and from there back to the caller.
//!
//! In:  nothing
//! Out: every register preserved (`stop_cd_track` keeps them all)
//! Six callers: the CD audio's functions (0x40F8BB .. 0x40FDB9) and `system::main_loop`
//! (0x415439).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};

// ---- game data ----

const TRACK_RVA: usize = rva(0x518110); // dword_518110
const STOPS: u32 = 5;

/// A call: 0 `sound::stop_cd_track`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

// ---- Rust version ----

pub unsafe fn stop_cd_audio(entry: &Regs, track: u32, part: Part) -> Result<(), Refused> {
    unsafe { (track as *mut u32).write_unaligned(0) };
    let mut r = *entry;
    for _ in 0..STOPS {
        part(0, &mut r)?;
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let track = game_addr(TRACK_RVA) as u32;
    unsafe { stop_cd_audio(&entry, track, &mut crate::sound::stop_cd_track::game_part) }?;
    Ok(())
}
