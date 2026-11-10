//! `effects::pong_score` (0x4B8382): the pong score: the best (`dword_7687CC`) raised to the
//! hits (`dword_7687D0`) when they reach it; the place 0x100, 0x384; the hits (low byte,
//! kept) as three `effects::pong_digit`s (hundreds, tens, ones), 0x20 on, the best (low
//! byte, kept) the same.
//!
//! Caller: `effects::run_pong`.

use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::session::Dword;

/// A number of the score as three digits.
fn number(
    pong: &Pong,
    fl: &WindscreenFlakes,
    at: &Dword,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use crate::effects::pong_digit::pong_digit;
    let (n, [hundreds, tens, ones]) = rc2k_core::effects::pong::score_digits(at.get());
    at.set(n);
    pong_digit(pong, fl, hundreds, effects)?;
    pong_digit(pong, fl, tens, effects)?;
    pong_digit(pong, fl, ones, effects)?;
    Ok(())
}

// Converted: `rc2k_core::effects::pong::{best, score_digits}`
pub fn pong_score(
    pong: &Pong,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use rc2k_core::effects::pong::{SCORE_AT, SCORE_GAP, best};
    let p = &pong.state;
    let hits = p.hits.get();
    let was = p.best.get();
    if best(hits, was) != was {
        p.best.set(hits);
    }
    p.digit_at[0].set(SCORE_AT[0]);
    p.digit_at[1].set(SCORE_AT[1]);
    number(pong, fl, &p.hits, effects)?;
    p.digit_at[0].set(p.digit_at[0].get().wrapping_add(SCORE_GAP));
    number(pong, fl, &p.best, effects)?;
    Ok(())
}
