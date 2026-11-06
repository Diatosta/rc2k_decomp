//! `physics::object_turn_matrix` (0x45E2D2): the turn of the object hit (`dword_6D321C`)
//! into `flt_5EA444`. `scene::set_turn_y` by minus `dword_63503C`, copied (16 dwords);
//! with `dword_635040` set also: the turn about y by minus `dword_635044` times it into
//! `g_litMatrix` (`scene::multiply_matrices`), the turn about x by minus `dword_635040`
//! times that into `g_turnedMatrix`, the turn about y by `dword_635044` times that into
//! `flt_5EA444`. Then each of its 16 floats times `dword_635038` / 16 (`fild`, `fld1`
//! doubled four times, `fdivp`; the share through `dword_62400C`).
//!
//! In:  nothing
//! Out: the registers as the steps leave them (eax, ebx `dword_635038`, ecx 0, edx, esi
//!      past the matrix, edi, ebp); no flags read (the caller returns)
//!
//! Caller: `sub_45E755`. Refuses (before anything) an object past 0xFF.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::{ext_div2, int_80};
use crate::scene::draw_store::DRAW;
use crate::scene::matrix::{multiply_matrices, set_turn_x, set_turn_y};
use crate::scene::object_store::OBJECT_WORK;

/// The Rust version. The registers it leaves are read by none of its callers.
pub fn object_turn_matrix() -> Result<(), Refused> {
    let object = crate::race::owner::race().physics.contact.face_object.get();
    if object > 0xFF {
        return Err(BadInput("an object past 0xFF").into());
    }
    set_turn_y(OBJECT_WORK.angles[0].get().wrapping_neg());
    let (out, turn_y, turn_x) = (
        &crate::race::owner::race().frame.matrices.object_turn_matrix,
        &crate::race::owner::race().frame.matrices.turn_y_matrix,
        &crate::race::owner::race().frame.matrices.turn_x_matrix,
    );
    let (lit, turned) = (
        &crate::race::owner::race().frame.matrices.lit_matrix,
        &crate::race::owner::race().frame.matrices.turned_matrix,
    );
    for k in 0..16 {
        out[k].set_bits(turn_y[k].bits());
    }
    if OBJECT_WORK.angles[1].get() != 0 {
        set_turn_y(OBJECT_WORK.angles[2].get().wrapping_neg());
        multiply_matrices(&out, &turn_y, &lit);
        set_turn_x(OBJECT_WORK.angles[1].get().wrapping_neg());
        multiply_matrices(&lit, &turn_x, &turned);
        set_turn_y(OBJECT_WORK.angles[2].get());
        multiply_matrices(&turned, &turn_y, &out);
    }
    let figure = OBJECT_WORK.size.get();
    DRAW.scratch.set(figure);
    let sixteen = 16.0f32.to_bits();
    let share = ext_div2(int_80(figure as i32, false), sixteen, 0x3F80_0000);
    DRAW.scratch.set(share);
    for cell in out {
        cell.set_bits(x87::mul(cell.bits(), share));
    }
    Ok(())
}
