//! `scene::load_front_end_objects` (0x425CA9): loads the shapes of all 0x100 object
//! slots.
//!
//! In:  nothing
//! Out: eax = the faces' memory used, ecx = 0x100, edi = the end of the kinds' table,
//!      ebx = the last slot that had a kind (unchanged if none had)
//!
//! `scene::reset_objects`; `dword_631B68` and `dword_631F6C` = 0; for each slot its kind from
//! `dword_5CCE7C` to `dword_634FE8`, the slot to `dword_634FEC`, and `scene::load_object`; then
//! `dword_634FA4` = the points used, in bytes, and `dword_634FA8` = the faces' memory used. Two
//! callers (the front end's start and a stage's leaving).

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::scene::load_object::load_object;
use crate::scene::objects::{FACE_CURSOR_RVA, KIND_RVA, POINT_COUNT_RVA, SLOT_RVA, SLOTS};
use crate::scene::reset_objects::reset_objects;

// ---- game data ----

const TEXTURE_COUNT_RVA: usize = rva(0x631B68); // dword_631B68
const NO_NORMALS_RVA: usize = rva(0x631F6C); // dword_631F6C
const SLOT_KINDS_RVA: usize = rva(0x5CCE7C); // dword_5CCE7C
/// `dword_634FA4` the points used in bytes, `dword_634FA8` the faces' memory used.
const USED_RVA: usize = rva(0x634FA4);

// ---- Rust version ----

/// The Rust version. The registers the original leaves (eax the faces' memory used, ecx
/// 0x100, edi, ebx) are read by no caller (taint at its sites: `race::set_object_kinds`' and
/// `load_front_end_objects_logged`'s).
pub unsafe fn load_front_end_objects() -> Result<(), Refused> {
    unsafe {
        reset_objects()?;
        game_write(TEXTURE_COUNT_RVA, 0u32);
        game_write(NO_NORMALS_RVA, 0u32);
        for slot in 0..SLOTS {
            let kind: u32 = game_read(SLOT_KINDS_RVA + slot as usize * 4);
            game_write(KIND_RVA, kind);
            game_write(SLOT_RVA, slot);
            load_object()?;
        }
        let cursor: u32 = game_read(FACE_CURSOR_RVA);
        game_write(
            USED_RVA,
            [game_read::<u32>(POINT_COUNT_RVA).wrapping_mul(12), cursor],
        );
        Ok(())
    }
}
