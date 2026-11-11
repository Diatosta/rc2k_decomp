//! `physics::object_turn_matrix` (0x45E2D2): the turn of the object hit (`face_object`)
//! into `object_turn_matrix`. `scene::set_turn_y` by minus `angles`, copied (16 dwords);
//! with `dword_635040` set also: the turn about y by minus `dword_635044` times it into
//! `g_litMatrix` (`scene::multiply_matrices`), the turn about x by minus `dword_635040`
//! times that into `g_turnedMatrix`, the turn about y by `dword_635044` times that into
//! `object_turn_matrix`. Then each of its 16 floats times `size` / 16 (`fild`, `fld1`
//! doubled four times, `fdivp`; the share through `scratch`).
//!
//! Caller: `physics::object_hits_car`. Refuses (before anything) an object past 0xFF.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::{ext_div2, int_80};
use crate::physics::contact_store::ContactWork;
use crate::race::matrices_store::SceneMatrices;
use crate::scene::matrix::{multiply_matrices, set_turn_x, set_turn_y};

pub fn object_turn_matrix(
    contact: &ContactWork,
    matrices: &SceneMatrices,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let object = contact.face_object.get();
    if object > 0xFF {
        return Err(BadInput("an object past 0xFF").into());
    }
    set_turn_y(
        &matrices.turn_y_matrix,
        &scene.drawing.scene.turn_y_work,
        drawing.object_work.angles[0].get().wrapping_neg(),
    );
    let (out, turn_y, turn_x) = (
        &matrices.object_turn_matrix,
        &matrices.turn_y_matrix,
        &matrices.turn_x_matrix,
    );
    let (lit, turned) = (&matrices.lit_matrix, &matrices.turned_matrix);
    for k in 0..16 {
        out[k].set_bits(turn_y[k].bits());
    }
    if drawing.object_work.angles[1].get() != 0 {
        set_turn_y(
            &matrices.turn_y_matrix,
            &scene.drawing.scene.turn_y_work,
            drawing.object_work.angles[2].get().wrapping_neg(),
        );
        multiply_matrices(&out, &turn_y, &lit);
        set_turn_x(
            &matrices.turn_x_matrix,
            &scene.drawing.scene.turn_x_work,
            drawing.object_work.angles[1].get().wrapping_neg(),
        );
        multiply_matrices(&lit, &turn_x, &turned);
        set_turn_y(
            &matrices.turn_y_matrix,
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
        cell.set_bits(x87::mul(cell, share));
    }
    Ok(())
}
