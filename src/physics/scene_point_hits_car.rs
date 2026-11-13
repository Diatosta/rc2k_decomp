//! `physics::scene_point_hits_car` (0x45E7BF): `physics::point_hits_car` for the scene point
//! whose number is the record's second dword (`g_scenePoints` + 12 n), with no offset:
//! `scene_point_hits` is incremented and `object_offset` set to 0.
//!
//! Takes: the record (+4 the point's number), the object
//!
//! Caller: `physics::face_contact`.

use crate::errors::Refused;
use crate::physics::flow::fill_cells;
use crate::physics::point_hits_car::point_hits_car;
use crate::physics::step_families::ContactInputs;

/// For course face `face` (its number: the original's record and object are both the face):
/// (inside, the point's number).
pub fn scene_point_hits_car(c: &ContactInputs<'_>, face: u32) -> Result<(bool, u32), Refused> {
    let number = crate::race::course_places::face_field::<u32>(c.course.reads(), face, 4)?;
    c.tables
        .scene_point_hits
        .set(c.tables.scene_point_hits.get().wrapping_add(1));
    fill_cells(&c.contact.object_offset, 0.0);
    let place = crate::race::scene_points::point_number(
        c.course.reads(),
        &crate::race::scene_points::no_own(),
        number,
    )?;
    Ok(({ point_hits_car(c, place, face) }?, number))
}
