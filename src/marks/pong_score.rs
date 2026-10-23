//! `marks::pong_score` (0x4B8382): the pong score: the best (`dword_7687CC`) raised to the
//! hits (`dword_7687D0`) when they reach it; the place 0x100, 0x384; the hits (low byte,
//! kept) as three `marks::pong_digit`s (hundreds, tens, ones), 0x20 on, the best (low
//! byte, kept) the same.
//!
//! In:  nothing
//! Out: eax / edx the best's ones, ebx 10; the rest as the digits leave them
//!
//! Caller: `marks::run_pong`. The digits through `part` (1), in the game their Rust
//! version. Shadow mode runs the original alone (it draws). Self-test: a copy whose digit
//! calls go to a stand-in, against the Rust version, hits and bests by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write};
use crate::marks::pong;

/// A call by place: 1 `pong_digit`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A number of the score as three digits (the digits keep every register: the registers
/// set for each as the original's divisions leave them).
unsafe fn number(g: &mut Regs, at: usize, part: &mut Part) -> Result<(), Refused> {
    unsafe {
        let (n, [hundreds, tens, ones]) = rc2k_core::marks::pong::score_digits(game_read(at));
        game_write(at, n);
        [g.eax, g.edx, g.ebx] = [hundreds, n % 100, 100];
        part(1, g)?;
        [g.eax, g.edx, g.ebx] = [tens, ones, 10];
        part(1, g)?;
        g.eax = ones;
        part(1, g)?;
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::pong::{best, score_digits}`
pub unsafe fn pong_score(entry: &Regs, mut part: Part) -> Result<Regs, Refused> {
    use rc2k_core::marks::pong::{SCORE_AT, SCORE_GAP, best};
    let mut g = *entry;
    unsafe {
        let hits: u32 = game_read(pong::HITS_RVA);
        let was: u32 = game_read(pong::BEST_RVA);
        if best(hits, was) != was {
            game_write(pong::BEST_RVA, hits);
        }
        game_write(pong::DIGIT_AT[0], SCORE_AT[0]);
        game_write(pong::DIGIT_AT[1], SCORE_AT[1]);
        number(&mut g, pong::HITS_RVA, &mut part)?;
        game_write(
            pong::DIGIT_AT[0],
            game_read::<u32>(pong::DIGIT_AT[0]).wrapping_add(SCORE_GAP),
        );
        number(&mut g, pong::BEST_RVA, &mut part)?;
    }
    Ok(g)
}

/// In the game: `pong_digit`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::pong_digit as pd;
    *r = unsafe { pd::pong_digit(r) }?;
    Ok(())
}
