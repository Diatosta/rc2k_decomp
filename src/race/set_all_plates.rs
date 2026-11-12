//! `race::set_all_plates` (0x42518B): the texture kinds' pictures for the stage (`LOADLEVEL_INIT :
//! setplatesall`), then the system and slot textures reloaded.
//!
//! - The stage prefs' (`localdat`) picture lists copied to `g_kindPictures` (0x5D7968), 0x32C14
//!   bytes below: 0x60A57C .. 0x60A64C, 0x60A67C .. 0x60A73C, 0x60A77C .. 0x60A79C (the
//!   original in 4-dword `rep movsd` pieces).
//! - 23 kinds' pictures fixed (`FIXED_PICTURES`).
//! - With cars, each car's eight from 0x5D7D68: its kind's picture
//!   (`scene::known_kind_picture` of `race_car_kinds`[car]) and the seven after it.
//! - `scene::reload_system_textures`, `scene::reload_slot_textures`.
//!
//! Callers: `race::loadlevel_init` (loadlevel_init) and the code at 0x425B29. Refuses more than 8
//! cars.

use crate::errors::Refused;
use crate::race::owner::Race;

/// `race::set_all_plates` (see the header).
pub fn set_all_plates(
    race: &'static Race,
    gfx: &crate::gfx::owner::Gfx,
    net: &crate::net::owner::Net,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let course = &race.course;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::stage_objects::{check_picture_cars, PREF_PIECES,
    // FIXED_PICTURES, car_pictures}`.
    use rc2k_core::race::stage_objects::{
        CAR_PICTURE_KINDS, FIXED_PICTURES, PREF_PIECES, car_pictures, check_picture_cars,
    };
    let cars: u32 = shared.view.race_cars.get();
    check_picture_cars(cars)?;
    let pictures = &scene.textures.places.kind_pictures;
    let prefs = &course.stage_prefs;
    for (kind, dwords) in PREF_PIECES {
        // The prefs' picture lists (`PREF_PIECES`' kinds) lie one after the other (`picture_lists_a`,
        // `_b`, `_c`, 64 kinds each): a piece starts in the list its first kind falls in.
        let list = match kind / 64 {
            0 => &prefs.picture_lists_a,
            1 => &prefs.picture_lists_b,
            _ => &prefs.picture_lists_c,
        };
        for n in 0..dwords {
            pictures[kind + n].set(list[kind % 64 + n].get());
        }
    }
    for (kind, picture) in FIXED_PICTURES {
        pictures[kind].set(picture);
    }
    let mut at = CAR_PICTURE_KINDS;
    for car in 0..cars {
        let first = crate::scene::known_kind_picture::known_kind_picture(
            race.kind_tables(),
            shared.vars.race_car_kinds[car as usize].get(), // g_raceCarKinds
        );
        for picture in car_pictures(first) {
            pictures[at].set(picture);
            at += 1;
        }
    }
    crate::scene::reload_system_textures::reload_system_textures(race, gfx, net, scene)?;
    crate::scene::reload_slot_textures::reload_slot_textures(&race.frame, scene)?;
    Ok(())
}
