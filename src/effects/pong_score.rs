//! `effects::pong_score`: the pong score: the best (`PongState.best`) raised to the hits
//! (`PongState.hits`) when they reach it; the place 0x100, 0x384; the hits (low byte, kept) as
//! three `effects::pong_digit`s (hundreds, tens, ones), 0x20 on, the best (low byte, kept) the
//! same.

use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::memory::session::PlainDword;

/// A number of the score as three digits.
fn number(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    at: &PlainDword,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use crate::effects::pong_digit::pong_digit;
    let (n, [hundreds, tens, ones]) = rc2k_core::effects::pong::score_digits(at.get());
    at.set(n);
    pong_digit(buffer, pong, fl, hundreds, scene, effects)?;
    pong_digit(buffer, pong, fl, tens, scene, effects)?;
    pong_digit(buffer, pong, fl, ones, scene, effects)?;
    Ok(())
}

// Converted: `rc2k_core::effects::pong::{best, score_digits}`
pub fn pong_score(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    scene: &crate::scene::owner::Scene,
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
    number(buffer, pong, fl, &p.hits, scene, effects)?;
    p.digit_at[0].set(p.digit_at[0].get().wrapping_add(SCORE_GAP));
    number(buffer, pong, fl, &p.best, scene, effects)?;
    Ok(())
}
