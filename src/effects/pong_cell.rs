//! `effects::pong_cell`: one cell of a score digit: unless the pattern byte is 0,
//! `effects::draw_pong_box` at (the column times a cell width plus the score's x, the row times a
//! cell height plus the score's y) sized by the cell's width and height; the pattern pointer then
//! moves on one.
//!
//! The box's place is `rc2k_core::effects::pong::cell_place` (a whole number times a float plus a
//! whole number: exact in `f64`, as the x87's at double precision; at extended too while the
//! product fits 53 bits, as the score's cells do).

use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::math::matrix3::extended;
use rc2k_core::math::extended::from_f64;

/// `pattern` is the pattern byte, `column` and `row` the cell's place.
// The logic is in `rc2k_core::effects::pong::cell_place`
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the cell's place, the owners and the vertex buffer"
)]
pub fn pong_cell(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    pattern: u8,
    column: u32,
    row: u32,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        if pattern != 0 {
            let step = pong
                .rules
                .cell_step
                .each_ref()
                .map(crate::memory::session::PlainFloat::get);
            let at = pong.state.digit_at.each_ref().map(|d| d.get() as i32);
            let [x, y] = rc2k_core::effects::pong::cell_place(column as i32, row as i32, step, at);
            let values = [
                extended(pong.rules.cell_size[1].get()),
                extended(pong.rules.cell_size[0].get()),
                from_f64(y),
                from_f64(x),
            ];
            crate::effects::draw_pong_box::draw_pong_box(buffer, pong, fl, values, scene, effects);
        }
    }
    Ok(())
}
