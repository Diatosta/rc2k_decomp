//! `effects::pong_digit`: one score digit: `effects::pong_cell` for its 4 x 5 pattern (the digit
//! patterns, 0x14 bytes a digit; walking the cells by column and row), then the score's x on 0x40.
//!
//! Refuses a digit past 9 (the patterns' end), before anything.

use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;

/// `digit` is the digit.
// Converted: `rc2k_core::effects::pong::{pattern, cells}`
pub fn pong_digit(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    digit: u32,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use rc2k_core::effects::pong as core;
    let first = core::pattern(digit)? as usize; // the digit's offset, 20 bytes a digit
    for (n, (col, row)) in core::cells().enumerate() {
        crate::effects::pong_cell::pong_cell(
            buffer,
            pong,
            fl,
            pong.state.digit_patterns[first / 20][n].get(),
            col,
            row,
            scene,
            effects,
        )?;
    }
    pong.state.digit_at[0].set(pong.state.digit_at[0].get().wrapping_add(core::DIGIT_STEP));
    Ok(())
}
