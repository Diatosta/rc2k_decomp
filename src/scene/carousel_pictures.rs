//! The pictures the carousel's entries show (the cars, the stages): which four are
//! wanted, which four are held, and the slots they go to. Shared by
//! `scene::choose_carousel_pictures` and `scene::update_carousel_pictures`.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write};
use crate::menu::state_store::MENU_STATE;
use crate::scene::textures::{KIND_SLOT_RVA, KIND_TEXTURE_RVA, SLOTS, WANTED_RVA};
use crate::session::{read_all, write_all};

const KIND: usize = 0x2BC;
pub const FIRST_TEXTURE_RVA: usize = KIND_TEXTURE_RVA + KIND * 4; // dword_5D15D8
pub const FIRST_SLOT_RVA: usize = KIND_SLOT_RVA + KIND * 4; // dword_5D25D8
const ENTRIES: u32 = 0x100;

/// `scene::update_carousel_pictures` (0x49029C) for the four entries given: each
/// entry's picture (the first dword of its record) is wanted and shown; the held
/// pictures that are still wanted stay where they are, the others give their places to
/// the newly wanted ones (`scene::keep_held_pictures` 0x49036F); and each held
/// picture's eight textures go to its eight slots (`scene::assign_picture_slots`
/// 0x4903E3). Nothing is written before a refusal.
pub unsafe fn update(entries: [u32; 4]) -> Result<(), BadInput> {
    unsafe {
        let table: u32 = MENU_STATE.carousel.entries.get();
        if table == 0 {
            return Err(BadInput("no carousel entries' records"));
        }
        let mut wanted = [0u32; 4];
        let mut record = 0;
        for (n, &entry) in entries.iter().enumerate() {
            if entry >= ENTRIES {
                return Err(BadInput("a carousel entry past the records"));
            }
            record = ((table + entry * 4) as *const u32).read_unaligned();
            if record == 0 {
                return Err(BadInput("a carousel entry without its record"));
            }
            wanted[n] = (record as *const u32).read_unaligned();
        }
        let first_slot: u32 = game_read(FIRST_SLOT_RVA);
        if first_slot.checked_add(32).is_none_or(|end| end > SLOTS) {
            return Err(BadInput("the pictures' slots lie past the table"));
        }
        let first_texture: u32 = game_read(FIRST_TEXTURE_RVA);
        write_all(&MENU_STATE.carousel_pictures.shown, wanted);

        // Converted: `rc2k_core::scene::carousel::{keep_held, picture_textures}`.
        let (wanted, held) = rc2k_core::scene::carousel::keep_held(
            wanted,
            read_all(&MENU_STATE.carousel_pictures.held),
        );
        write_all(&MENU_STATE.carousel_pictures.wanted, wanted);
        write_all(&MENU_STATE.carousel_pictures.held, held);
        let textures = rc2k_core::scene::carousel::picture_textures(held, first_texture);
        game_write(WANTED_RVA + first_slot as usize * 4, textures);
        // What the original leaves in eax, ebx, ecx, esi and edi its callers' later calls
        // overwrite or never read (the carousel takes ebx and ecx anew).
        let _ = record;
        Ok(())
    }
}
