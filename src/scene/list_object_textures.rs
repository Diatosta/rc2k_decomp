//! `scene::list_object_textures` (0x426ACA): the different textures the faces of the
//! file just read use.
//!
//! In:  nothing (the file in `byte_A2BB54`, the slot in `dword_634FEC`)
//! Out: every register preserved
//!
//! Goes through the file's faces (after its points; a face is 0x10 bytes and as many again for
//! each vertex) and lists each face's texture (its word +4) once, in the order met, in
//! `dword_631B6C`; their number to `dword_631B68`. For a slot under 0x40 the number and the
//! list also go to the slot's record of 0x40 bytes at `unk_5DCEBC`; more than fifteen textures
//! run on into the next slots' records, as in the original, while that stays inside the 0x40
//! records. One caller, `scene::load_object`, for slots up to 0x40. Refuses a file of no faces
//! (the original goes through 2^32), faces that run past the file's room, more than 0x100
//! textures, and a list that would leave the records.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::objects::FILE_ROOM;
use crate::scene::texture_places_store::TEXTURE_PLACES;

// ---- game data ----

// ---- Rust version ----

/// The file's bytes at an offset, if they lie in its room.
pub fn file_dword(offset: usize) -> Result<u32, BadInput> {
    if offset + 4 > FILE_ROOM {
        return Err(BadInput("the object's file runs past its room"));
    }
    GamePtr::new(crate::scene::objects::file_addr() + offset as u32)
        .read("a word of an object's file")
}

// Converted: `rc2k_core::scene::objects::{texture_list, texture_record}`
pub fn list_object_textures() -> Result<(), BadInput> {
    use rc2k_core::scene::objects::{texture_list, texture_record};

    let textures = texture_list(|offset| file_dword(offset).map_err(|e| rc2k_core::Invalid(e.0)))?;
    let slot: u32 = OBJECT_WORK.loading_slot.bits();
    let record = texture_record(slot, textures.len())?;
    crate::scene::object_slots_store::OBJECT_SLOTS
        .texture_count
        .set(textures.len() as u32);
    for (n, &texture) in textures.iter().enumerate() {
        crate::scene::object_slots_store::OBJECT_SLOTS.texture_list[n].set(texture);
    }
    if let Some(record) = record {
        // `unk_5DCEBC`: the records (`record` a byte offset in 64 of 0x40 bytes).
        let records = &TEXTURE_PLACES.object_texture_records;
        records[record / 4].set(textures.len() as u32);
        for (n, &texture) in textures.iter().enumerate() {
            records[record / 4 + 1 + n].set(texture);
        }
    }
    Ok(())
}
