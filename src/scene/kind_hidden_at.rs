//! `scene::kind_hidden_at` (0x4522F7): `scene::kind_is_hidden` for the object whose
//! number is the byte at esi+1, eax kept (`push eax` / `pop eax`).
//!
//! In:  esi the record (its byte +1 the object)
//! Out: CF as `kind_is_hidden` leaves it; every register kept
//!
//! Caller: `sub_45E3BF` (0x45E3D0, `jb`).

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::scene::kind_is_hidden::kind_is_hidden;

/// The Rust version: whether the record's object is hidden by its kind.
pub fn kind_hidden_at(record: u32) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::scene::kinds::kind_is_hidden` (through `kind_is_hidden`).
    let object = GamePtr::new(record.wrapping_add(1)).read::<u8>("a course face")? as u32;
    // A byte: never past the table.
    Ok({ kind_is_hidden(object) }.is_ok_and(|(hidden, _)| hidden))
}
