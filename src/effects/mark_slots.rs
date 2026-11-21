//! The mark slots' matrix handle and its matrix (the slots themselves: `mark_slots_store`).

use crate::race::owner_frame::Frame;

/// A slot's matrix handle for the view place matrix (the frame's `view_place` matrix), the only one
/// `effects::add_mark_slot` is given.
pub(super) const VIEW_PLACE_MATRIX: u32 = 0xCD00_0001;

/// The matrix a slot's handle names; refused for any other value.
pub(super) fn slot_matrix(
    frame: &Frame,
    handle: u32,
) -> Result<crate::scene::matrix::Matrix, crate::errors::BadInput> {
    match handle {
        VIEW_PLACE_MATRIX => Ok(frame.view_kept.borrow().view_place),
        _ => Err(crate::errors::BadInput(
            "a mark slot's matrix handle that names none",
        )),
    }
}
