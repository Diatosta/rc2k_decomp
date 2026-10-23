//! `scene::kind_hidden_at` (0x4522F7): `scene::kind_is_hidden` for the object whose
//! number is the byte at esi+1, eax kept (`push eax` / `pop eax`).
//!
//! In:  esi the record (its byte +1 the object)
//! Out: CF as `kind_is_hidden` leaves it; every register kept
//!
//! Caller: `sub_45E3BF` (0x45E3D0, `jb`).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): both settings, a test record with objects 0 .. 0xFF and test kinds.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::scene::kind_is_hidden::kind_is_hidden;

/// The Rust version: whether the record's object is hidden by its kind.
pub unsafe fn kind_hidden_at(record: u32) -> bool {
    // Converted: `rc2k_core::scene::kinds::kind_is_hidden` (through `kind_is_hidden`).
    let object = unsafe { ((record + 1) as *const u8).read() } as u32;
    // A byte: never past the table.
    unsafe { kind_is_hidden(object) }.is_ok_and(|(hidden, _)| hidden)
}
