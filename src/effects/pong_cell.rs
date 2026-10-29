//! `effects::pong_cell` (0x4B8557): one cell of a score digit: unless the pattern byte at
//! eax is 0, `effects::draw_pong_box` at (ebx times `flt_7691FA` plus `dword_7687D4`, ecx
//! times `flt_7691FE` plus `dword_7687D8`) sized `flt_769202` by `flt_769206`; eax on one.
//!
//! In:  eax the pattern byte, ebx the column, ecx the row
//! Out: nothing (its caller walks the pattern itself)
//!
//! Caller: `effects::pong_digit` (twenty times).
//!
//! Converted 2026-10-21: the box's place through `rc2k_core::effects::pong::cell_place` (a
//! whole number times a float plus a whole number: exact in `f64`, as the x87's at double
//! precision; at extended too while the product fits 53 bits, as the score's cells do).

use crate::effects::pong;
use crate::effects::pong::PONG_RULES;
use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::scene::matrix3::extended;
use rc2k_core::math::extended::from_f64;

/// The Rust version (see the header): `pattern` the pattern byte's address (eax),
/// `column` (ebx), `row` (ecx).
// Converted: `rc2k_core::effects::pong::cell_place`
pub fn pong_cell(pattern: u32, column: u32, row: u32) -> Result<(), Refused> {
    {
        if GamePtr::new(pattern).read::<u8>("a pong pattern byte")? != 0 {
            let step = PONG_RULES.cell_step.each_ref().map(|f| f.get());
            let at = pong::PONG.digit_at.each_ref().map(|d| d.get() as i32);
            let [x, y] = rc2k_core::effects::pong::cell_place(column as i32, row as i32, step, at);
            let values = [
                extended(PONG_RULES.cell_size[1].bits()),
                extended(PONG_RULES.cell_size[0].bits()),
                from_f64(y),
                from_f64(x),
            ];
            crate::effects::draw_pong_box::draw_pong_box(values);
        }
    }
    Ok(())
}
