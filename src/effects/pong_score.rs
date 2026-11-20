//! `effects::pong_score`: the pong score: the best (`PongState.best`) raised to the hits
//! (`PongState.hits`) when they reach it; the place 0x100, 0x384; the hits (low byte, kept) as
//! three `effects::pong_digit`s (hundreds, tens, ones), 0x20 on, the best (low byte, kept) the
//! same.

use crate::effects::pong_store::{Pong, PongState};
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;

/// A number of the score as three digits.
fn number(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    field: fn(&mut PongState) -> &mut u32,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use crate::effects::pong_digit::pong_digit;
    let (n, [hundreds, tens, ones]) =
        rc2k_core::effects::pong::score_digits(*field(&mut pong.state.borrow_mut()));
    *field(&mut pong.state.borrow_mut()) = n;
    pong_digit(buffer, pong, fl, hundreds, scene, effects)?;
    pong_digit(buffer, pong, fl, tens, scene, effects)?;
    pong_digit(buffer, pong, fl, ones, scene, effects)?;
    Ok(())
}

// The logic is in `rc2k_core::effects::pong::{best, score_digits}`
pub(super) fn pong_score(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use rc2k_core::effects::pong::{SCORE_AT, SCORE_GAP, best};
    {
        let mut p = pong.state.borrow_mut();
        let (hits, was) = (p.hits, p.best);
        if best(hits, was) != was {
            p.best = hits;
        }
        p.digit_at = SCORE_AT;
    }
    number(buffer, pong, fl, |p| &mut p.hits, scene, effects)?;
    {
        let mut p = pong.state.borrow_mut();
        p.digit_at[0] = p.digit_at[0].wrapping_add(SCORE_GAP);
    }
    number(buffer, pong, fl, |p| &mut p.best, scene, effects)?;
    Ok(())
}
