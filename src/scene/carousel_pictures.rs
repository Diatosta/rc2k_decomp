//! The pictures the carousel's entries show (the cars, the stages): which four are
//! wanted, which four are held, and the slots they go to. Shared by
//! `scene::choose_carousel_pictures` and `scene::update_carousel_pictures`.

use crate::errors::BadInput;
use crate::menu::state_store::MENU_STATE;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::texture_places_store::TEXTURE_PLACES;
use crate::scene::textures::SLOTS;
use crate::session::{read_all, write_all};

const KIND: usize = 0x2BC;
const ENTRIES: u32 = 0x100;

/// `scene::update_carousel_pictures` (0x49029C) for the four entries given: each
/// entry's picture (the first dword of its record) is wanted and shown; the held
/// pictures that are still wanted stay where they are, the others give their places to
/// the newly wanted ones (`scene::keep_held_pictures` 0x49036F); and each held
/// picture's eight textures go to its eight slots (`scene::assign_picture_slots`
/// 0x4903E3). Nothing is written before a refusal.
pub fn update(entries: [u32; 4]) -> Result<(), BadInput> {
    let Some(table) = MENU_STATE.carousel.list()? else {
        return Err(BadInput("no carousel entries' records"));
    };
    let table = crate::menu::lists_store::MENU_LISTS.cars(table);
    let mut wanted = [0u32; 4];
    let mut record = 0;
    for (n, &entry) in entries.iter().enumerate() {
        if entry >= ENTRIES {
            return Err(BadInput("a carousel entry past the records"));
        }
        record = table.offset(entry * 4).read("a carousel entry's record")?;
        if record == 0 {
            return Err(BadInput("a carousel entry without its record"));
        }
        wanted[n] = crate::menu::cars_store::description(record)?.get().car;
    }
    let first_slot: u32 = KIND_TEXTURES.kind_first_slots[KIND].get();
    if first_slot.checked_add(32).is_none_or(|end| end > SLOTS) {
        return Err(BadInput("the pictures' slots lie past the table"));
    }
    let first_texture: u32 = KIND_TEXTURES.kind_first_textures[KIND].get();
    write_all(&MENU_STATE.carousel_pictures.shown, wanted);

    // Converted: `rc2k_core::scene::carousel::{keep_held, picture_textures}`.
    let (wanted, held) =
        rc2k_core::scene::carousel::keep_held(wanted, read_all(&MENU_STATE.carousel_pictures.held));
    write_all(&MENU_STATE.carousel_pictures.wanted, wanted);
    write_all(&MENU_STATE.carousel_pictures.held, held);
    let textures = rc2k_core::scene::carousel::picture_textures(held, first_texture);
    for (n, texture) in textures.into_iter().enumerate() {
        TEXTURE_PLACES.slot_wanted[first_slot as usize + n].set(texture);
    }
    // What the original leaves in eax, ebx, ecx, esi and edi its callers' later calls
    // overwrite or never read (the carousel takes ebx and ecx anew).
    let _ = record;
    Ok(())
}
