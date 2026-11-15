//! The pictures the carousel's entries show (the cars, the stages): which four are wanted, which
//! four are held, and the slots they go to. Shared by `scene::choose_carousel_pictures` and
//! `update`.

use crate::errors::BadInput;
use crate::menu::lists_store::CarListParts;
use crate::menu::owner::Menus;
use crate::scene::textures::SLOTS;
use crate::session::{read_all, write_all};

const KIND: usize = 0x2BC;
const ENTRIES: u32 = 0x100;

/// Updates the carousel pictures for the four entries given: each entry's picture (the
/// first dword of its record) is wanted and shown; the held pictures that are still wanted stay
/// where they are, the others give their places to the newly wanted ones
/// (`rc2k_core::scene::carousel::keep_held`); and each held picture's eight textures go to its
/// eight slots (`rc2k_core::scene::carousel::picture_textures`). Nothing is written before a
/// refusal.
pub fn update(
    menus: &Menus,
    scene: &crate::scene::owner::Scene,
    entries: [u32; 4],
) -> Result<(), BadInput> {
    let (carousel, cars) = (&menus.screens.carousel, &menus.cars.table);
    let scene_textures = &scene.textures;
    let Some(table) = carousel.list()? else {
        return Err(BadInput("no carousel entries' records"));
    };
    let mut wanted = [0u32; 4];
    let mut record = 0;
    for (n, &entry) in entries.iter().enumerate() {
        if entry >= ENTRIES {
            return Err(BadInput("a carousel entry past the records"));
        }
        // The list is a run of record pointers, one dword per entry.
        record = menus
            .lists
            .car_entry(CarListParts::of(menus), table, entry)?;
        if record == 0 {
            return Err(BadInput("a carousel entry without its record"));
        }
        wanted[n] = crate::menu::cars_store::description(cars, record)?
            .get()
            .car;
    }
    let first_slot: u32 = scene_textures.kinds.get().kind_first_slots[KIND].get();
    if first_slot.checked_add(32).is_none_or(|end| end > SLOTS) {
        return Err(BadInput("the pictures' slots lie past the table"));
    }
    let first_texture: u32 = scene_textures.kinds.get().kind_first_textures[KIND].get();
    write_all(&carousel.pictures.shown, wanted);

    // Logic: `rc2k_core::scene::carousel::{keep_held, picture_textures}`.
    let (wanted, held) =
        rc2k_core::scene::carousel::keep_held(wanted, read_all(&carousel.pictures.held));
    write_all(&carousel.pictures.wanted, wanted);
    write_all(&carousel.pictures.held, held);
    let textures = rc2k_core::scene::carousel::picture_textures(held, first_texture);
    for (n, texture) in textures.into_iter().enumerate() {
        scene_textures.places.get().slot_wanted[first_slot as usize + n].set(texture);
    }
    let _ = record;
    Ok(())
}
