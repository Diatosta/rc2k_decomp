//! `physics::object_turn_matrix` (0x45E2D2): the turn of the object hit (`dword_6D321C`)
//! into `flt_5EA444`. `scene::set_turn_y` by minus `dword_63503C`, copied (16 dwords);
//! with `dword_635040` set also: the turn about y by minus `dword_635044` times it into
//! `g_litMatrix` (`scene::multiply_matrices`), the turn about x by minus `dword_635040`
//! times that into `g_turnedMatrix`, the turn about y by `dword_635044` times that into
//! `flt_5EA444`. Then each of its 16 floats times `dword_635038` / 16 (`fild`, `fld1`
//! doubled four times, `fdivp`; the share through `dword_62400C`).
//!
//! Caller: `physics::object_hits_car`. Refuses (before anything) an object past 0xFF.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::{ext_div2, int_80};
use crate::race::owner::Race;
use crate::scene::matrix::{multiply_matrices, set_turn_x, set_turn_y};

pub fn object_turn_matrix(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let object = race.physics.contact.face_object.get();
    if object > 0xFF {
        return Err(BadInput("an object past 0xFF").into());
    }
    set_turn_y(
        &race.frame.matrices.turn_y_matrix,
        &scene.drawing.scene.turn_y_work,
        drawing.object_work.angles[0].get().wrapping_neg(),
    );
    let (out, turn_y, turn_x) = (
        &race.frame.matrices.object_turn_matrix,
        &race.frame.matrices.turn_y_matrix,
        &race.frame.matrices.turn_x_matrix,
    );
    let (lit, turned) = (
        &race.frame.matrices.lit_matrix,
        &race.frame.matrices.turned_matrix,
    );
    for k in 0..16 {
        out[k].set_bits(turn_y[k].bits());
    }
    if drawing.object_work.angles[1].get() != 0 {
        set_turn_y(
            &race.frame.matrices.turn_y_matrix,
            &scene.drawing.scene.turn_y_work,
            drawing.object_work.angles[2].get().wrapping_neg(),
        );
        multiply_matrices(&out, &turn_y, &lit);
        set_turn_x(
            &race.frame.matrices.turn_x_matrix,
            &scene.drawing.scene.turn_x_work,
            drawing.object_work.angles[1].get().wrapping_neg(),
        );
        multiply_matrices(&lit, &turn_x, &turned);
        set_turn_y(
            &race.frame.matrices.turn_y_matrix,
            &scene.drawing.scene.turn_y_work,
            drawing.object_work.angles[2].get(),
        );
        multiply_matrices(&turned, &turn_y, &out);
    }
    let figure = drawing.object_work.size.get();
    drawing.scene.scratch.set(figure);
    let sixteen = 16.0f32.to_bits();
    let share = ext_div2(int_80(figure as i32, false), sixteen, 0x3F80_0000);
    drawing.scene.scratch.set(share);
    for cell in out {
        cell.set_bits(x87::mul(cell.bits(), share));
    }
    Ok(())
}
