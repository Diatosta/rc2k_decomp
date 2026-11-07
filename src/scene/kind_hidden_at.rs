//! `scene::kind_hidden_at` (0x4522F7): `scene::kind_is_hidden` for the object whose
//! number is the byte at esi+1, eax kept (`push eax` / `pop eax`).
//!
//! Takes: the record (its byte +1 the object)
//! Out: CF as `kind_is_hidden` leaves it; every register kept
//!
//! Caller: `sub_45E3BF` (0x45E3D0, `jb`).

use crate::errors::BadInput;
use crate::race::owner::Race;
use crate::scene::kind_is_hidden::kind_is_hidden;

/// The Rust version: whether the object of course face `face` (its number; the face is
/// the original's record) is hidden by its kind.
pub fn kind_hidden_at(
    race: &Race,
    face: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::scene::kinds::kind_is_hidden` (through `kind_is_hidden`).
    let object = crate::race::course_places::face_field::<u8>(face, 1, "a course face")? as u32;
    // A byte: never past the table.
    Ok({ kind_is_hidden(&race.shared, object, scene) }.is_ok_and(|(hidden, _)| hidden))
}
