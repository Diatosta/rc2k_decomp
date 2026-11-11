//! `scene::kind_hidden_at` (0x4522F7): `scene::kind_is_hidden` for the object whose
//! number is the record's byte +1.
//!
//! Takes: the record (its byte +1 the object)
//!
//! Caller: `physics::face_contact` (0x45E3D0, `jb`).

use crate::errors::BadInput;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use crate::scene::kind_is_hidden::kind_is_hidden;

/// The Rust version: whether the object of course face `face` (its number; the face is
/// the original's record) is hidden by its kind.
pub fn kind_hidden_at(
    course: &Course,
    shared: &RaceShared,
    face: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::scene::kinds::kind_is_hidden` (through `kind_is_hidden`).
    let object = crate::race::course_places::face_field::<u8>(course, face, 1)? as u32;
    // A byte: never past the table.
    Ok({ kind_is_hidden(shared, object, scene) }.is_ok_and(|(hidden, _)| hidden))
}
