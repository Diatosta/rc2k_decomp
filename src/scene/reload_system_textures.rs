//! `scene::reload_system_textures` (0x4254B4): every system texture gets the picture
//! it should have ("reload system plates").
//!
//! - Two lines in the game's log (a rule and "Reload system plates - start").
//! - Each kind's first texture (`g_kindFirstTextures`), where it has one, wants the
//!   kind's picture (`TexturePlaces.kind_pictures[0]`; 0 counts as none, -1).
//! - For each of the `TexturePlaces.totals[3]` system textures: `Grading.reading` = 1 and the size
//!   pictures are read at (`Grading.saturation`, `Grading.scale`) = the scene's
//!   (`StagePrefs.scene_size_a`, `StagePrefs.scene_size_b`), or 256 x 256 for picture 0x3DE and pictures
//!   0x706 .. 0x7CF; `scene::load_changed_texture_picture`;
//!   `scene::step_loading_bar`.
//! - `Grading.reading` = 0 and the size 256 x 256 again; `scene::load_hud_car_icons`;
//!   two lines in the log.
//!
//! Two callers. Refuses no system textures (the original goes through 2^32) or more than 0x400,
//! and a first texture past the table.
//!
//! The logic is in the core crate: the kinds' pictures and the read sizes through
//! `rc2k_core::scene::pictures`.

use crate::errors::{BadInput, Refused};
use crate::scene::load_changed_texture_picture::{TEXTURES, load_changed_texture_picture_in_game};
use crate::scene::load_hud_car_icons::load_hud_car_icons;
use crate::scene::step_loading_bar::step_loading_bar;
use crate::scene::textures::KINDS;
use rc2k_core::scene::pictures::{kind_picture, read_size};

// ---- game data ----

/// `StagePrefs.scene_size_a`, `StagePrefs.scene_size_b`: the scene's size for them.
// scene::load_hud_car_icons
const TEXTS: [&std::ffi::CStr; 4] = [
    c"-----------------------------",
    c"Reload system plates - start",
    c"Reload system plates - finis",
    c"-----------------------------",
];

// ---- Rust version ----

// Converted: `rc2k_core::scene::pictures::{kind_picture, read_size}`
pub fn reload_system_textures(
    race: &'static crate::race::owner::Race,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    d3d: &crate::d3d::owner::D3d,
    net: &crate::net::owner::Net,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let gfx_grading = &gfx.grading;
    let course = &race.course;
    let textures: u32 = scene.textures.places.totals[3].get();
    if textures == 0 || textures > TEXTURES {
        return Err(BadInput("no system textures, or more than the table has").into());
    }
    let first = |kind: u32| scene.textures.kinds.kind_first_textures[kind as usize].get();
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
        let picture = kind_picture(scene.textures.places.kind_pictures[kind as usize].get());
        scene.textures.places.wanted_pictures[texture as usize].set(picture);
    }
    let size = [
        course.stage_prefs.scene_size_a.get(),
        course.stage_prefs.scene_size_b.get(),
    ];
    for texture in 0..textures {
        gfx_grading.reading.set(1u32);
        let wanted = scene.textures.places.wanted_pictures[texture as usize].get();
        let [scale, saturation] = read_size(wanted, size);
        let grading = &gfx_grading;
        grading.scale.set(scale);
        grading.saturation.set(saturation);
        load_changed_texture_picture_in_game(race, gfx, texture, scene, system)?;
        step_loading_bar(race, system, gfx, d3d, scene, texture)?;
    }
    let grading = &gfx_grading;
    grading.reading.set(0u32);
    grading.scale.set(0x100);
    grading.saturation.set(0x100);
    load_hud_car_icons(scene, race, net, gfx, system)?;
    crate::logfile::log_write::log_nul(&TEXTS[2], true, 1);
    crate::logfile::log_write::log_nul(&TEXTS[3], true, 1);
    Ok(())
}
