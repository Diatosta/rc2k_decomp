//! `effects::pong_score` (0x4B8382): the pong score: the best (`dword_7687CC`) raised to the
//! hits (`dword_7687D0`) when they reach it; the place 0x100, 0x384; the hits (low byte,
//! kept) as three `effects::pong_digit`s (hundreds, tens, ones), 0x20 on, the best (low
//! byte, kept) the same.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `effects::run_pong`.

use crate::effects::pong::PONG;
use crate::errors::Refused;
use crate::session::Dword;

/// A number of the score as three digits.
fn number(at: &Dword) -> Result<(), Refused> {
    use crate::effects::pong_digit::pong_digit;
    let (n, [hundreds, tens, ones]) = rc2k_core::effects::pong::score_digits(at.get());
    at.set(n);
    pong_digit(hundreds)?;
    pong_digit(tens)?;
    pong_digit(ones)?;
    Ok(())
}

/// The Rust version (see the header).
// Converted: `rc2k_core::effects::pong::{best, score_digits}`
pub fn pong_score() -> Result<(), Refused> {
    use rc2k_core::effects::pong::{SCORE_AT, SCORE_GAP, best};
    let p = &PONG;
    let hits = p.hits.get();
    let was = p.best.get();
    if best(hits, was) != was {
        p.best.set(hits);
    }
    p.digit_at[0].set(SCORE_AT[0]);
    p.digit_at[1].set(SCORE_AT[1]);
    number(&p.hits)?;
    p.digit_at[0].set(p.digit_at[0].get().wrapping_add(SCORE_GAP));
    number(&p.best)?;
    Ok(())
}
