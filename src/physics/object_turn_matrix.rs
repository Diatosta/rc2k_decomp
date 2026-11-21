//! The turn of the object hit (`face_object`), into `object_turn_matrix`: `scene::set_turn_y` by
//! minus `ObjectWork.angles[0]`, copied (16 dwords); with `ObjectWork.angles[1]` set also: the turn
//! about y by minus `ObjectWork.angles[2]` times it into `lit_matrix` (`scene::multiply_matrices`),
//! the turn about x by minus `ObjectWork.angles[1]` times that into `turned_matrix`, the turn about
//! y by `ObjectWork.angles[2]` times that into `object_turn_matrix`. Then each of its 16 floats is
//! multiplied by `size` / 16 (the share through `scratch`).
//!
//! Refuses (before anything) an object past 0xFF.

use crate::errors::{BadInput, Refused};
use crate::math::x87::{ext_div2, int_80};
use crate::memory::session::DrawPlain;
use crate::physics::contact_store::ContactWork;
use crate::race::matrices_store::SceneMatrices;
use crate::scene::matrix::{TURN_X_BASE, TURN_Y_BASE, product, scaled, turned_x, turned_y};
use std::cell::RefCell;

pub fn object_turn_matrix(
    engine_tables: crate::math::engine_tables::EngineTablesRef,
    contact: &RefCell<ContactWork>,
    matrices: &DrawPlain<SceneMatrices>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let work = scene.object_work();
    let object = contact.borrow().face_object;
    if object > 0xFF {
        return Err(BadInput("an object past 0xFF").into());
    }
    // Each turn starts from its base (a turn leaves the floats it does not set as they were).
    let (turn_y, _) = turned_y(engine_tables, TURN_Y_BASE, work.angles[0].wrapping_neg());
    let mut out = turn_y;
    if work.angles[1] != 0 {
        let (turn_y, _) = turned_y(engine_tables, TURN_Y_BASE, work.angles[2].wrapping_neg());
        let lit = {
            let mut kept = matrices.borrow_mut();
            kept.lit_matrix = product(&out, &turn_y, kept.lit_matrix);
            kept.lit_matrix
        };
        let (turn_x, _) = turned_x(engine_tables, TURN_X_BASE, work.angles[1].wrapping_neg());
        let turned = {
            let mut kept = matrices.borrow_mut();
            kept.turned_matrix = product(&lit, &turn_x, kept.turned_matrix);
            kept.turned_matrix
        };
        let (turn_y, _) = turned_y(engine_tables, TURN_Y_BASE, work.angles[2]);
        out = product(&turned, &turn_y, out);
    }
    let figure = work.size;
    let share = ext_div2(int_80(figure as i32, false), 16.0f32, 1.0f32);
    matrices.borrow_mut().object_turn_matrix = scaled(out, share);
    Ok(())
}
