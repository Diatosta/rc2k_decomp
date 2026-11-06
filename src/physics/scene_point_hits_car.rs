//! `physics::scene_point_hits_car` (0x45E7BF): `physics::point_hits_car` for the scene
//! point whose number is the dword at esi+4 (`g_scenePoints` + 12 n), with no offset:
//! `dword_614108` + 1, `dword_6D3B40` .. = 0.
//!
//! In:  esi the record (+4 the point's number), ebp the object
//! Out: CF as `point_hits_car` leaves it; ebx the number, esi the point; the rest kept
//!
//! Caller: `sub_45E3BF` (0x45E48C, `jb`).

use crate::errors::Refused;
use crate::physics::flow::fill_cells;
use crate::physics::point_hits_car::point_hits_car;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner::race;

/// The Rust version for course face `face` (its number: the original's record and object
/// are both the face): (inside, the point's number).
pub fn scene_point_hits_car(face: u32) -> Result<(bool, u32), Refused> {
    let number = crate::race::course_places::face_field::<u32>(face, 4, "a scene point record")?;
    PHYSICS
        .scene_point_hits
        .set(PHYSICS.scene_point_hits.get().wrapping_add(1));
    fill_cells(&race().physics.contact.object_offset, 0);
    let place = crate::race::scene_points::point_number(number)?;
    Ok(({ point_hits_car(place, face) }?, number))
}
