//! `marks::pong_score` (0x4B8382): the pong score: the best (`dword_7687CC`) raised to the
//! hits (`dword_7687D0`) when they reach it; the place 0x100, 0x384; the hits (low byte,
//! kept) as three `marks::pong_digit`s (hundreds, tens, ones), 0x20 on, the best (low
//! byte, kept) the same.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_pong`. The digits through `part` (1), in the game their Rust
//! version. Shadow mode runs the original alone (it draws). Self-test: a copy whose digit
//! calls go to a stand-in, against the Rust version, hits and bests by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::marks::pong;
use crate::marks::store::{Slot, game_read, game_write};

/// A number of the score as three digits.
unsafe fn number(at: Slot) -> Result<(), Refused> {
    use crate::marks::pong_digit::pong_digit;
    unsafe {
        let (n, [hundreds, tens, ones]) = rc2k_core::marks::pong::score_digits(game_read(at));
        game_write(at, n);
        pong_digit(hundreds)?;
        pong_digit(tens)?;
        pong_digit(ones)?;
    }
    Ok(())
}

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::pong::{best, score_digits}`
pub unsafe fn pong_score() -> Result<(), Refused> {
    use rc2k_core::marks::pong::{SCORE_AT, SCORE_GAP, best};
    unsafe {
        let hits: u32 = game_read(pong::HITS_RVA);
        let was: u32 = game_read(pong::BEST_RVA);
        if best(hits, was) != was {
            game_write(pong::BEST_RVA, hits);
        }
        game_write(pong::DIGIT_AT[0], SCORE_AT[0]);
        game_write(pong::DIGIT_AT[1], SCORE_AT[1]);
        number(pong::HITS_RVA)?;
        game_write(
            pong::DIGIT_AT[0],
            game_read::<u32>(pong::DIGIT_AT[0]).wrapping_add(SCORE_GAP),
        );
        number(pong::BEST_RVA)?;
    }
    Ok(())
}
