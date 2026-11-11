//! `scene::count_texture_needs` (0x423EB1): how many textures are needed and how much
//! memory they take, for the log.
//!
//! Over the 0x400 textures: those needed in system memory (the system need not -1) are counted
//! in the system textures total and their levels' sizes added to the system k total; those
//! needed in video memory in the video textures total and the video k total. Then one line
//! in the game's log: "VID <k> k in <n> texs - SYS <k> k in <n> texs". The video textures
//! total is also
//! what `scene::load_slot_textures` takes for the slots' number. Two call sites in
//! `scene::build_texture_catalogue`. Refuses a level past the sizes' table.

use crate::errors::BadInput;
use crate::logfile::log_int::log_int;
use crate::scene::owner::scene;

// ---- game data ----

// `TEXTURE_PLACES.totals`: video k, system k, video textures, system textures.

// ---- Rust version ----

/// The four totals. Reads only.
fn totals() -> Result<[u32; 4], BadInput> {
    let objects = &scene().objects;
    // Converted: `rc2k_core::scene::textures::count_needs`.
    let (totals, _) = crate::scene::textures::with_cells(
        [&objects.tables.system_levels, &objects.tables.video_levels],
        |[system, video]| {
            rc2k_core::scene::textures::count_needs(system, video, |level| {
                crate::scene::texture_places_store::cell_at(
                    &objects.tables.level_sizes[0],
                    level as u32,
                )
                .get()
            })
        },
    )?;
    Ok(totals.to_array())
}

/// `log`: what writes a piece of the line: a text, or a number.
pub fn count_texture_needs(
    log: &mut dyn FnMut(Result<&'static std::ffi::CStr, u32>, bool),
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let totals = totals()?;
    for (cell, total) in scene.textures.places.totals.iter().zip(totals) {
        cell.set(total);
    }
    log(Ok(c"VID"), false);
    log(Err(totals[0]), false);
    log(Ok(c"k in"), false);
    log(Err(totals[2]), false);
    log(Ok(c"texs - SYS"), false);
    log(Err(totals[1]), false);
    log(Ok(c"k in"), false);
    log(Err(totals[3]), false);
    log(Ok(c"texs"), true);
    Ok(())
}

/// Into the game's log.
pub fn count_texture_needs_in_game(scene: &crate::scene::owner::Scene) -> Result<(), BadInput> {
    count_texture_needs(
        &mut |piece, newline| match piece {
            Ok(text) => crate::logfile::log_write::log_nul(&text, newline, 1),
            Err(number) => log_int(number as i32, newline, 1),
        },
        scene,
    )
}
