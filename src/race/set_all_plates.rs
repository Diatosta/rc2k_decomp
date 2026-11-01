//! `race::set_all_plates` (0x42518B): the texture kinds' pictures for the stage
//! (`LOADLEVEL_INIT : setplatesall`), then the system and slot textures reloaded.
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
//!
//! - The stage prefs' (`localdat`) picture lists copied to `g_kindPictures` (0x5D7968), 0x32C14
//!   bytes below: 0x60A57C .. 0x60A64C, 0x60A67C .. 0x60A73C, 0x60A77C .. 0x60A79C (the
//!   original in 4-dword `rep movsd` pieces; esi, edi and ecx left as its last).
//! - 23 kinds' pictures fixed (`FIXED_PICTURES`).
//! - With cars, each car's eight from 0x5D7D68: its kind's picture
//!   (`scene::known_kind_picture` of `dword_623858`[car]) and the seven after it.
//! - `scene::reload_system_textures`, `scene::reload_slot_textures`.
//!
//! Callers: `sub_44426F` (loadlevel_init) and the code at 0x425B29. Refuses more than 8 cars.

use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::scene::texture_places_store::TEXTURE_PLACES;

/// `race::set_all_plates` (see the header).
pub fn set_all_plates() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage_objects::{check_picture_cars, PREF_PIECES,
    // FIXED_PICTURES, car_pictures}`.
    use rc2k_core::race::stage_objects::{
        CAR_PICTURE_KINDS, FIXED_PICTURES, PREF_PIECES, car_pictures, check_picture_cars,
    };
    let cars: u32 = RACE_VIEW.race_cars.bits();
    check_picture_cars(cars)?;
    let pictures = GamePtr::to(&TEXTURE_PLACES.kind_pictures);
    for (kind, dwords) in PREF_PIECES {
        // The prefs' picture lists (`PREF_PIECES`' kinds) start at `picture_lists_a`
        // (0x60A57C), 0x32C14 bytes above the pictures; a piece runs past the field.
        let from = GamePtr::to(&STAGE_PREFS.picture_lists_a)
            .unbounded()
            .offset(4 * kind as u32);
        let to = pictures.offset(4 * kind as u32);
        from.copy_to(to, 4 * dwords, "the picture lists")?;
    }
    for (kind, picture) in FIXED_PICTURES {
        crate::scene::texture_places_store::cell_at(&TEXTURE_PLACES.kind_pictures[0], kind as u32)
            .set(picture);
    }
    let mut at = pictures.offset(4 * CAR_PICTURE_KINDS as u32);
    for car in 0..cars {
        let first = crate::scene::known_kind_picture::known_kind_picture(
            RACE_VARS.race_car_kinds[car as usize].get(), // g_raceCarKinds
        );
        for picture in car_pictures(first) {
            at.write(picture, "a car's pictures")?;
            at = at.offset(4);
        }
    }
    crate::scene::reload_system_textures::reload_system_textures()?;
    crate::scene::reload_slot_textures::reload_slot_textures()?;
    Ok(())
}
