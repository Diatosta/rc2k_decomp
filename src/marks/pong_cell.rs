//! `marks::pong_cell` (0x4B8557): one cell of a score digit: unless the pattern byte at
//! eax is 0, `marks::draw_pong_box` at (ebx times `flt_7691FA` plus `dword_7687D4`, ecx
//! times `flt_7691FE` plus `dword_7687D8`) sized `flt_769202` by `flt_769206`; eax on one.
//!
//! In:  eax the pattern byte, ebx the column, ecx the row
//! Out: eax plus 1; the rest kept (`pusha` .. `popa`)
//!
//! Caller: `marks::pong_digit` (twenty times). The box through `part` (1 with its four
//! values), in the game its Rust version. Hooked through a private copy (a `jz` in the
//! first bytes). Shadow mode runs the original alone (it draws). Self-test: a copy whose
//! box goes to a stand-in taking the values off, against the Rust version.
//!
//! Converted 2026-10-21: the box's place through `rc2k_core::marks::pong::cell_place` (a
//! whole number times a float plus a whole number: exact in `f64`, as the x87's at double
//! precision; at extended too while the product fits 53 bits, as the score's cells do).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::marks::pong;
use crate::scene::matrix3::extended;
use rc2k_core::math::extended::from_f64;

const STEP_RVA: [usize; 2] = [rva(0x7691FA), rva(0x7691FE)]; // flt_7691FA, flt_7691FE
const SIZE_RVA: [usize; 2] = [rva(0x769202), rva(0x769206)]; // flt_769202, flt_769206

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::pong::cell_place`
pub unsafe fn pong_cell(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if (g.eax as *const u8).read() != 0 {
            let step = STEP_RVA.map(|r| f32::from_bits(game_read(r)));
            let at = pong::DIGIT_AT.map(|r| game_read::<i32>(r));
            let [x, y] = rc2k_core::marks::pong::cell_place(g.ebx as i32, g.ecx as i32, step, at);
            let values = [
                extended(game_read(SIZE_RVA[1])),
                extended(game_read(SIZE_RVA[0])),
                from_f64(y),
                from_f64(x),
            ];
            let mut r = g;
            crate::marks::pong_cell::game_part(1, &mut r, &values)?;
        }
    }
    g.eax = g.eax.wrapping_add(1);
    Ok(g)
}

/// In the game: `draw_pong_box`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::marks::draw_pong_box as pb;
    unsafe { pb::draw_pong_box(r, [v[0], v[1], v[2], v[3]]) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { pong_cell(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
