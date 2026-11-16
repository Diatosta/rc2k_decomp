//! `scene::kind_hidden_at`: `scene::kind_is_hidden` for the object whose number is the
//! record's byte +1. Caller: `physics::face_contact`.

use crate::errors::BadInput;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use crate::scene::kind_is_hidden::kind_is_hidden;

/// Whether the object of course face `face` (its number; the face is the record) is hidden by
/// its kind.
pub fn kind_hidden_at(
    course: &Course,
    shared: &RaceShared,
    face: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, BadInput> {
    // Logic: `rc2k_core::scene::kinds::kind_is_hidden` (through `kind_is_hidden`).
    let object = crate::race::course_places::face_field::<u8>(course.reads(), face, 1)? as u32;
    // A byte: never past the table.
    Ok({ kind_is_hidden(shared, object, scene) }.is_ok_and(|(hidden, _)| hidden))
}
