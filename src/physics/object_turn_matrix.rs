//! `physics::object_turn_matrix` (0x45E2D2): the turn of the object hit (`face_object`)
//! into `object_turn_matrix`. `scene::set_turn_y` by minus `angles`, copied (16 dwords);
//! with `ObjectWork.angles[1]` set also: the turn about y by minus `ObjectWork.angles[2]` times it into
//! `g_litMatrix` (`scene::multiply_matrices`), the turn about x by minus `ObjectWork.angles[1]`
//! times that into `g_turnedMatrix`, the turn about y by `ObjectWork.angles[2]` times that into
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
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    contact: &ContactWork,
    matrices: &SceneMatrices,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let (drawing, work) = (&scene.drawing, scene.object_work());
    let object = contact.face_object.get();
    if object > 0xFF {
        return Err(BadInput("an object past 0xFF").into());
    }
    set_turn_y(
        engine_tables,
        &matrices.turn_y_matrix,
        &scene.drawing.scene.get().turn_y_work,
        work.angles[0].get().wrapping_neg(),
    );
    let (out, turn_y, turn_x) = (
        &matrices.object_turn_matrix,
        &matrices.turn_y_matrix,
        &matrices.turn_x_matrix,
    );
    let (lit, turned) = (&matrices.lit_matrix, &matrices.turned_matrix);
    for k in 0..16 {
        out[k].set(turn_y[k].get());
    }
    if work.angles[1].get() != 0 {
        set_turn_y(
            engine_tables,
            &matrices.turn_y_matrix,
            &scene.drawing.scene.get().turn_y_work,
            work.angles[2].get().wrapping_neg(),
        );
        multiply_matrices(&out, &turn_y, &lit);
        set_turn_x(
            engine_tables,
            &matrices.turn_x_matrix,
            &scene.drawing.scene.get().turn_x_work,
            work.angles[1].get().wrapping_neg(),
        );
        multiply_matrices(&lit, &turn_x, &turned);
        set_turn_y(
            engine_tables,
            &matrices.turn_y_matrix,
            &scene.drawing.scene.get().turn_y_work,
            work.angles[2].get(),
        );
        multiply_matrices(&turned, &turn_y, &out);
    }
    let figure = work.size.get();
    drawing.scene.get().scratch.set(figure);
    let share = ext_div2(int_80(figure as i32, false), 16.0f32, 1.0f32);
    drawing.scene.get().scratch.set(share.to_bits());
    for cell in out {
        cell.set(x87::mul(cell, share));
    }
    Ok(())
}
