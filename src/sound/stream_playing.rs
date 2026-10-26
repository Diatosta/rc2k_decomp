//! `sound::stream_playing` (0x40C4D5): the music stream's playing flag (`dword_506960`,
//! `g_soundStream.playing`) compared with 1: zero set when it plays, carry when it is 0.
//!
//! In:  nothing
//! Out: the flags of `cmp dword_506960, 1`; every register kept
//!
//! Callers: `sub_46F7AD` (`jz`), `sub_47F314` (`jnz`); `sound::step_car_stream` has it
//! inlined. Self-test: the original and the Rust version on the flag 0, 1, 2 and -1, both
//! flags compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, rva};

const PLAYING_RVA: usize = rva(0x506960); // g_soundStream.playing

/// The Rust version: (zero, carry) as the original's `cmp` leaves them.
pub fn stream_playing() -> (bool, bool) {
    let playing: u32 = unsafe { game_read(PLAYING_RVA) };
    (playing == 1, playing < 1)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let (zero, carry) = stream_playing();
    r.set_zero(zero);
    r.set_carry(carry);
    Ok(())
}
