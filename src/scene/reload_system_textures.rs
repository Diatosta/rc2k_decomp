//! `scene::reload_system_textures`: every system texture gets the picture it should have
//! ("reload system plates").
//!
//! - Two lines in the game's log (a rule and "Reload system plates - start").
//! - Each kind's first texture, where it has one, wants the kind's picture
//!   (`TexturePlaces.kind_pictures`; 0 counts as none, -1).
//! - For each of the `TexturePlaces.totals[3]` system textures: `Grading.reading` = 1 and the
//!   size pictures are read at (`Grading.saturation`, `Grading.scale`) = the scene's
//!   (`StagePrefs.scene_size_a`, `StagePrefs.scene_size_b`), or 256 x 256 for picture 0x3DE and
//!   pictures 0x706 .. 0x7CF; `scene::load_changed_texture_picture`; the loading bar moves on.
//! - `Grading.reading` = 0 and the size 256 x 256 again; `scene::load_hud_car_icons`; two lines
//!   in the log.
//!
//! Refuses more than 0x400 system textures and a first texture past the table.
//!
//! The kinds' pictures and the read sizes are in the core crate (`rc2k_core::scene::pictures`).

use crate::errors::{BadInput, Refused};
use crate::scene::load_changed_texture_picture::{TEXTURES, load_changed_texture_picture_in_game};
use crate::scene::load_hud_car_icons::load_hud_car_icons;
use crate::scene::textures::KINDS;
use rc2k_core::scene::pictures::{kind_picture, read_size};

// ---- game data ----

/// `StagePrefs.scene_size_a`, `StagePrefs.scene_size_b`: the scene's size for them.
const TEXTS: [&std::ffi::CStr; 4] = [
    c"-----------------------------",
    c"Reload system plates - start",
    c"Reload system plates - finis",
    c"-----------------------------",
];

// Logic: `rc2k_core::scene::pictures::{kind_picture, read_size}`
pub fn reload_system_textures(
    o: crate::owners::Owners<'static>,
    progress: &mut dyn crate::race::loading_bar_frame::Progress,
) -> Result<(), Refused> {
    let (race, system, gfx, scene) = (o.race, o.system, o.gfx, o.scene);
    let gfx_grading = &gfx.grading;
    let course = &race.course;
    let textures: u32 = scene.textures.places.get().totals[3].get();
    if textures == 0 || textures > TEXTURES {
        return Err(BadInput("no system textures, or more than the table has").into());
    }
    let first = |kind: u32| scene.textures.kinds.get().kind_first_textures[kind as usize].get();
    if (0..KINDS).any(|kind| first(kind) != u32::MAX && first(kind) >= TEXTURES) {
        return Err(BadInput("a kind's first texture past the pictures' table").into());
    }
    crate::logfile::log_write::log_nul(&TEXTS[0], true, 1);
    crate::logfile::log_write::log_nul(&TEXTS[1], true, 1);
    for kind in 0..KINDS {
        let texture = first(kind);
        if texture == u32::MAX {
            continue;
        }
        let picture = kind_picture(scene.textures.places.get().kind_pictures[kind as usize].get());
        scene.textures.places.get().wanted_pictures[texture as usize].set(picture);
    }
    let size = [
        course.stage_prefs.get().scene_size_a.get(),
        course.stage_prefs.get().scene_size_b.get(),
    ];
    for texture in 0..textures {
        gfx_grading.reading.set(1u32);
        let wanted = scene.textures.places.get().wanted_pictures[texture as usize].get();
        let [scale, saturation] = read_size(wanted, size);
        let grading = &gfx_grading;
        grading.scale.set(scale);
        grading.saturation.set(saturation);
        load_changed_texture_picture_in_game(race, gfx, texture, scene, system)?;
        progress.textures_done(textures, texture)?;
    }
    let grading = &gfx_grading;
    grading.reading.set(0u32);
    grading.scale.set(0x100);
    grading.saturation.set(0x100);
    crate::race::load_times::mark("load_hud_car_icons");
    load_hud_car_icons(scene, race, gfx, system, &o.net.race_slot)?;
    crate::logfile::log_write::log_nul(&TEXTS[2], true, 1);
    crate::logfile::log_write::log_nul(&TEXTS[3], true, 1);
    Ok(())
}
