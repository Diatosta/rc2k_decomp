//! `scene::list_object_textures`: the different textures the faces of the file just read use.
//!
//! Goes through the file's faces (after its points; a face is 0x10 bytes and as many again for each
//! vertex) and lists each face's texture (its word +4) once, in the order met, in the texture list;
//! their number is kept. For a slot under 0x40 the number and the list also go to the slot's record
//! of 0x40 bytes; more than fifteen textures run on into the next slots' records, while that stays
//! inside the 0x40 records. Refuses a file of no faces, faces that run past the file's room, more
//! than 0x100 textures, and a list that would leave the records.

use crate::errors::BadInput;
use crate::scene::objects::FILE_ROOM;

// ---- game data ----

/// The file's bytes at an offset, if they lie in its room.
pub fn file_dword(race: &crate::race::owner::Race, offset: usize) -> Result<u32, BadInput> {
    if offset + 4 > FILE_ROOM {
        return Err(BadInput("the object's file runs past its room"));
    }
    Ok(u32::from_le_bytes(
        crate::scene::objects::file(race)
            .read_at(offset, 4, "a word of an object's file")?
            .try_into()
            .unwrap(),
    ))
}

// Logic: `rc2k_core::scene::objects::{texture_list, texture_record}`
pub fn list_object_textures(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    use rc2k_core::scene::objects::{texture_list, texture_record};

    let textures =
        texture_list(|offset| file_dword(race, offset).map_err(|e| rc2k_core::Invalid(e.0)))?;
    let slot: u32 = scene.object_work().loading_slot;
    let record = texture_record(slot, textures.len())?;
    {
        let mut slots = scene.loaded_slots_mut();
        slots.texture_count = textures.len() as u32;
        for (n, &texture) in textures.iter().enumerate() {
            slots.texture_list[n] = texture;
        }
    }
    if let Some(record) = record {
        // `TexturePlaces.object_texture_records[0]`: the records (`record` a byte offset in
        // 64 of 0x40 bytes).
        let mut places = scene.texture_places_mut();
        let records = &mut places.object_texture_records;
        records[record / 4] = textures.len() as u32;
        for (n, &texture) in textures.iter().enumerate() {
            records[record / 4 + 1 + n] = texture;
        }
    }
    Ok(())
}
