//! The pictures the carousel's entries show (the cars, the stages): which four are
//! wanted, which four are held, and the slots they go to. Shared by
//! `scene::choose_carousel_pictures` and `scene::update_carousel_pictures`.

use crate::errors::BadInput;
use crate::framework::{EAX, EBX, ECX, EDI, ESI, game_addr, game_read, game_write, rva};
use crate::scene::textures::{KIND_SLOT_RVA, KIND_TEXTURE_RVA, SLOTS, WANTED_RVA};
use crate::screens::harness::Left;
use crate::service::harness::Region;

pub const WANTED_PICTURES_RVA: usize = rva(0x75AAA8); // dword_75AAA8 .. dword_75AAB4
pub const SHOWN_RVA: usize = rva(0x75AAB8); // dword_75AAB8 .. dword_75AAC4
pub const HELD_RVA: usize = rva(0x75AAC8); // dword_75AAC8 .. dword_75AAD4
pub const ENTRIES_RVA: usize = rva(0x75A65C); // dword_75A65C: the entries' records
const KIND: usize = 0x2BC;
pub const FIRST_TEXTURE_RVA: usize = KIND_TEXTURE_RVA + KIND * 4; // dword_5D15D8
pub const FIRST_SLOT_RVA: usize = KIND_SLOT_RVA + KIND * 4; // dword_5D25D8
const ENTRIES: u32 = 0x100;

/// The 32 slots of the four pictures, where they lie in the table.
pub fn slot_regions() -> Vec<Region> {
    let slot: u32 = unsafe { game_read(FIRST_SLOT_RVA) };
    let mut regions = vec![(game_addr(WANTED_PICTURES_RVA) as u32, 0x30)];
    if slot.checked_add(32).is_some_and(|end| end <= SLOTS) {
        regions.push((game_addr(WANTED_RVA) as u32 + slot * 4, 32 * 4));
    }
    regions
}

/// `scene::update_carousel_pictures` (0x49029C) for the four entries given: each
/// entry's picture (the first dword of its record) is wanted and shown; the held
/// pictures that are still wanted stay where they are, the others give their places to
/// the newly wanted ones (`scene::keep_held_pictures` 0x49036F); and each held
/// picture's eight textures go to its eight slots (`scene::assign_picture_slots`
/// 0x4903E3). Nothing is written before a refusal.
pub unsafe fn update(entries: [u32; 4]) -> Result<Left, BadInput> {
    unsafe {
        let table: u32 = game_read(ENTRIES_RVA);
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
        game_write(SHOWN_RVA, wanted);

        let mut held: [u32; 4] = game_read(HELD_RVA);
        for held in &mut held {
            match wanted.iter().position(|wanted| wanted == held) {
                Some(place) => wanted[place] = u32::MAX,
                None => *held = u32::MAX,
            }
        }
        for &wanted in &wanted {
            if wanted != u32::MAX
                && let Some(free) = held.iter().position(|&held| held == u32::MAX)
            {
                held[free] = wanted;
            }
        }
        game_write(WANTED_PICTURES_RVA, wanted);
        game_write(HELD_RVA, held);

        let mut texture = 0;
        for (place, &picture) in held.iter().enumerate() {
            texture = (picture << 3).wrapping_add(first_texture);
            for n in 0..8 {
                game_write(
                    WANTED_RVA + (first_slot as usize + place * 8 + n) * 4,
                    texture,
                );
                texture = texture.wrapping_add(1);
            }
        }
        Ok(Left::nothing()
            .with(EAX, texture)
            .with(EBX, first_slot + 32)
            .with(ECX, 0)
            .with(ESI, 24)
            .with(EDI, record))
    }
}
