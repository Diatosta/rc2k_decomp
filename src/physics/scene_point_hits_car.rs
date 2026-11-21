//! `physics::point_hits_car` for the scene point whose number is the record's second dword (the
//! scene points, 12 bytes each), with no offset: `object_offset` set to 0.

use crate::errors::Refused;
use crate::physics::point_hits_car::point_hits_car;
use crate::physics::step_families::ContactInputs;

/// For course face `face` (its number: the record and the object are both the face): (inside, the
/// point's number).
pub(super) fn scene_point_hits_car(
    c: &ContactInputs<'_>,
    face: u32,
) -> Result<(bool, u32), Refused> {
    let number = crate::race::course_places::face_field::<u32>(c.course.reads(), face, 4)?;
    c.contact.borrow_mut().object_offset = [0; 3];
    let place = crate::race::scene_points::point_number(
        c.course.reads(),
        &crate::race::scene_points::no_own(),
        number,
    )?;
    Ok(({ point_hits_car(c, place, face) }?, number))
}
