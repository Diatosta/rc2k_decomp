//! `physics::object_hits_car` (0x45E755): whether any point of the object hit
//! (`dword_6D321C`) lies inside the work car's box. `dword_61410C` + 1;
//! `physics::object_turn_matrix` (in `pusha` / `popa`); then for each of the object's
//! points (`g_objectFirstPoint[n]` on, `g_objectPointCount2[n]` of them, in
//! `g_objectPoints`, three whole numbers each): `fild`ed, turned by `flt_5EA444`
//! (`scene::rotate_point`) and `fistp`ed into the offset `dword_6D3B40` .., then
//! `physics::point_hits_car` (esi, ebp the caller's); the first inside ends it.
//!
//! In:  esi, ebp passed to `point_hits_car` (ebp the object's record)
//! Out: CF set when a point is inside; eax the point's number, ecx the points left (0 when
//!      none was inside), edx its address (kept with no points), ebx `dword_6D321C`; the
//!      rest kept
//!
//! Caller: `sub_45E3BF` (0x45E51B, `jb`). Refuses (before anything) an object past 0xFF
//! (the tables' room; `object_turn_matrix` refuses it too).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87_run::{ext_to_int, int_80};
use crate::physics::object_turn_matrix::object_turn_matrix;
use crate::physics::point_hits_car::point_hits_car;
use crate::physics::tables_store::PHYSICS;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::contacts;

/// The Rust version: whether one of the object's points is inside the car (CF); `point`
/// and `object` what esi and ebp hold at entry (`point_hits_car`'s). The registers it
/// leaves (eax the point's number, ebx the object, ecx the points left, edx the last
/// address) are read by none of its callers.
// Converted: `rc2k_core::physics::contacts::{check_object, walk_points}` (the turns stay
// x87).
pub fn object_hits_car(point: u32, face: u32) -> Result<bool, Refused> {
    let number = crate::race::owner::race().physics.contact.face_object.get();
    contacts::check_object(number)?;
    { object_turn_matrix() }?;
    PHYSICS.object_hits.set(
        crate::physics::tables_store::PHYSICS
            .object_hits
            .get()
            .wrapping_add(1),
    );
    let count = crate::scene::object_slots_store::OBJECT_SLOTS.point_counts[number as usize].get();
    // With no points the original skips to `clc` before it sets edx (0x45E779); the
    // points' table is read only when there are some.
    let points = if count == 0 {
        0
    } else {
        crate::race::owner::race().course.lists.object_points.get()
    };
    let refused = std::cell::Cell::new(None::<Refused>);
    let walk = contacts::walk_points(
        crate::scene::object_slots_store::OBJECT_SLOTS.first_point[number as usize].get(),
        count,
        points,
        0,
        |edx| {
            if refused.get().is_some() {
                return false;
            }
            let at = |k: u32| {
                crate::scene::object_points::word(edx + 4 * k)
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
            let turned = rotate_point(
                &crate::race::owner::race().frame.matrices.object_turn_matrix,
                [x, y, z],
            );
            for (k, e) in turned.iter().enumerate() {
                crate::race::owner::race().physics.contact.object_offset[k]
                    .set(ext_to_int(*e) as u32);
            }
            // `point` is the scene point's number (from `face_contact`).
            let place = match crate::race::scene_points::point_number(point) {
                Ok(p) => p,
                Err(e) => {
                    refused.set(Some(e.into()));
                    return false;
                }
            };
            match point_hits_car(place, face) {
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
