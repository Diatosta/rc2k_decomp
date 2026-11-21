//! `race::set_all_plates`: sets the texture kinds' pictures for the stage, then reloads the
//! system and slot textures.
//!
//! - The stage prefs' picture lists are copied into `kind_pictures` in 4-dword pieces.
//! - 23 kinds' pictures are fixed (`FIXED_PICTURES`).
//! - Each car gets eight pictures: its kind's picture (`scene::known_kind_picture` of
//!   `race_car_kinds`[car]) and the seven after it.
//!
//! Refuses more than 8 cars.

use crate::errors::Refused;

/// Sets the stage's texture pictures and reloads the textures (see the header).
pub fn set_all_plates(
    o: crate::owners::Owners<'static>,
    progress: &mut dyn crate::race::loading_bar_frame::Progress,
) -> Result<(), Refused> {
    let (race, scene) = (o.race, o.scene);
    let course = &race.course;
    let shared = &race.shared;
    use rc2k_core::race::stage_objects::{
        CAR_PICTURE_KINDS, FIXED_PICTURES, PREF_PIECES, car_pictures, check_picture_cars,
    };
    let cars: u32 = shared.view.get().race_cars.get();
    check_picture_cars(cars)?;
    let mut places = scene.texture_places_mut();
    let pictures = &mut places.kind_pictures;
    let prefs = course.stage_prefs.borrow();
    for (kind, dwords) in PREF_PIECES {
        // The prefs hold three picture lists of 64 kinds each (`picture_lists_a`, `_b`, `_c`):
        // a piece is read from the list its first kind falls in.
        let list = match kind / 64 {
            0 => &prefs.picture_lists_a,
            1 => &prefs.picture_lists_b,
            _ => &prefs.picture_lists_c,
        };
        for n in 0..dwords {
            pictures[kind + n] = list[kind % 64 + n];
        }
    }
    for (kind, picture) in FIXED_PICTURES {
        pictures[kind] = picture;
    }
    let mut at = CAR_PICTURE_KINDS;
    for car in 0..cars {
        let first = crate::scene::known_kind_picture::known_kind_picture(
            race.kind_tables(),
            shared.vars.get().race_car_kinds[car as usize].get(),
        );
        for picture in car_pictures(first) {
            pictures[at] = picture;
            at += 1;
        }
    }
    drop(places);
    crate::logfile::load_times::mark("reload_system_textures");
    crate::scene::reload_system_textures::reload_system_textures(o, progress)?;
    crate::logfile::load_times::mark("reload_slot_textures");
    crate::scene::reload_slot_textures::reload_slot_textures(&race.frame, scene)?;
    Ok(())
}
