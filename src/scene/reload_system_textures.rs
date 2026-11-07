//! `scene::reload_system_textures` (0x4254B4): every system texture gets the picture
//! it should have ("reload system plates").
//!
//! - Two lines in the game's log (a rule and "Reload system plates - start").
//! - Each kind's first texture (`g_kindFirstTextures`), where it has one, wants the
//!   kind's picture (`dword_5D7968`; 0 counts as none, -1).
//! - For each of the `dword_5DC974` system textures: `dword_4F4A24` = 1 and the size
//!   pictures are read at (`dword_4F4A2C`, `dword_4F4A28`) = the scene's
//!   (`dword_609300`, `dword_609304`), or 256 x 256 for picture 0x3DE and pictures
//!   0x706 .. 0x7CF; `scene::load_changed_texture_picture`;
//!   `scene::step_loading_bar`.
//! - `dword_4F4A24` = 0 and the size 256 x 256 again; `scene::load_hud_car_icons`;
//!   two lines in the log.
//!
//! Two callers. Refuses no system textures (the original goes through 2^32) or more than 0x400,
//! and a first texture past the table.
//!
//! Converted 2026-10-21: the kinds' pictures and the read sizes through
//! `rc2k_core::scene::pictures`.

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::gfx;
use crate::race::owner::race;
use crate::scene::load_changed_texture_picture::{TEXTURES, load_changed_texture_picture_in_game};
use crate::scene::load_hud_car_icons::load_hud_car_icons;
use crate::scene::step_loading_bar::step_loading_bar;
use crate::scene::textures::KINDS;
use rc2k_core::scene::pictures::{kind_picture, read_size};

// ---- game data ----

/// `dword_609300`, `dword_609304`: the scene's size for them.
// scene::load_hud_car_icons
const TEXTS: [&std::ffi::CStr; 4] = [
    c"-----------------------------",
    c"Reload system plates - start",
    c"Reload system plates - finis",
    c"-----------------------------",
];

// ---- Rust version ----

/// The Rust version. The registers the original leaves (those of `load_hud_car_icons`) are
/// read by no caller (taint at the site in `race::set_all_plates`: 0 read points; the other
/// is the game's own code, `sub_443478`, not Rust). It reads no register at entry but edi,
/// esi (for `load_changed_texture_picture` and `step_loading_bar`, which set them
/// themselves).
// Converted: `rc2k_core::scene::pictures::{kind_picture, read_size}`
pub fn reload_system_textures(scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let gfx_grading = &gfx().grading;
    let race = race();
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
        load_changed_texture_picture_in_game(race, texture, scene)?;
        step_loading_bar(race, texture)?;
    }
    let grading = &gfx_grading;
    grading.reading.set(0u32);
    grading.scale.set(0x100);
    grading.saturation.set(0x100);
    load_hud_car_icons(scene, &race.shared)?;
    crate::logfile::log_write::log_nul(&TEXTS[2], true, 1);
    crate::logfile::log_write::log_nul(&TEXTS[3], true, 1);
    Ok(())
}
