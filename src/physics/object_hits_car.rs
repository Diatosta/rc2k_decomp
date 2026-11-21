//! Whether any point of the object hit (`face_object`) lies inside the work car's box.
//! `physics::object_turn_matrix` runs; then each of the object's
//! points (in `object_points`, three whole numbers each) is turned by `object_turn_matrix`
//! (`scene::rotate_point`) and rounded into the offset `object_offset`, then
//! `physics::point_hits_car` runs with the caller's object; the first inside ends it.
//!
//! Refuses (before anything) an object past 0xFF (the tables' room; `object_turn_matrix` refuses it
//! too).

use crate::errors::Refused;
use crate::math::x87::{ext_to_int, int_80};
use crate::physics::object_turn_matrix::object_turn_matrix;
use crate::physics::point_hits_car::point_hits_car;
use crate::physics::step_families::ContactInputs;
use rc2k_core::physics::contacts;

/// Whether one of the object's points is inside the car ; `point` and `object` are
/// `point_hits_car`'s arguments.
// Core: `rc2k_core::physics::contacts::{check_object, walk_points}` (the turns stay x87).
pub fn object_hits_car(
    c: &ContactInputs<'_>,
    point: u32,
    face: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    let objects = &scene.objects;
    let number = c.contact.borrow().face_object;
    contacts::check_object(number)?;
    {
        object_turn_matrix(
            &c.shared.buffers.reads.get().engine_tables,
            c.contact,
            c.sound.matrices,
            scene,
        )
    }?;
    let count = objects.slots.borrow().point_counts[number as usize];
    // The points' table is read only when there are some.
    let points = if count == 0 {
        0
    } else {
        c.course.lists.get().object_points.get()
    };
    let refused = std::cell::Cell::new(None::<Refused>);
    let walk = contacts::walk_points(
        objects.slots.borrow().first_point[number as usize],
        count,
        points,
        0,
        |edx| {
            if refused.get().is_some() {
                return false;
            }
            let at = |k: u32| {
                c.course
                    .stage
                    .object_points
                    .word(edx + 4 * k)
                    .map(|v| int_80(v as i32, false))
                    .map_err(Refused::from)
            };
            let (x, y, z) = match (at(0), at(1), at(2)) {
                (Ok(x), Ok(y), Ok(z)) => (x, y, z),
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                    refused.set(Some(e));
                    return false;
                }
            };
            let turned = crate::scene::rotate_point::rotate_point_by(
                &c.sound.matrices.borrow().object_turn_matrix,
                [x, y, z],
            );
            for (k, e) in turned.iter().enumerate() {
                c.contact.borrow_mut().object_offset[k] = ext_to_int(*e) as u32;
            }
            // `point` is the scene point's number (from `face_contact`).
            let place = match crate::race::scene_points::point_number(
                c.course.reads(),
                &crate::race::scene_points::no_own(),
                point,
            ) {
                Ok(p) => p,
                Err(e) => {
                    refused.set(Some(e.into()));
                    return false;
                }
            };
            match point_hits_car(c, place, face) {
                Ok(inside) => inside,
                Err(e) => {
                    refused.set(Some(e));
                    false
                }
            }
        },
    );
    if let Some(e) = refused.take() {
        return Err(e);
    }
    Ok(walk.inside)
}
