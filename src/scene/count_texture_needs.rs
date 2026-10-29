//! `scene::count_texture_needs` (0x423EB1): how many textures are needed and how much
//! memory they take, for the log.
//!
//! In:  nothing
//! Out: eax = the last number looked at, ecx = 0x400
//!
//! Over the 0x400 textures: those needed in system memory (`dword_5CEAE0` not -1) are counted
//! in `dword_5DC974` and their levels' sizes (`dword_5CDAAC`) added in `dword_5DC96C`; those
//! needed in video memory (`dword_5CFAE0`) in `dword_5DC970` and `dword_5DC968`. Then one line
//! in the game's log: "VID <k> k in <n> texs - SYS <k> k in <n> texs". `dword_5DC970` is also
//! what `scene::load_slot_textures` takes for the slots' number. Two call sites in
//! `sub_42274B`. Refuses a level past the sizes' table.

use crate::errors::BadInput;
use crate::logfile::log_int::log_int;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::texture_places_store::TEXTURE_PLACES;

// ---- game data ----

// `TEXTURE_PLACES.totals`: `dword_5DC968` video k, `dword_5DC96C` system k, `dword_5DC970`
// video textures, `dword_5DC974` system textures.

// ---- Rust version ----

/// The four totals. Reads only.
fn totals() -> Result<[u32; 4], BadInput> {
    // Converted: `rc2k_core::scene::textures::count_needs`.
    let (totals, _) = crate::scene::textures::with_cells(
        [&OBJECT_TABLES.system_levels, &OBJECT_TABLES.video_levels],
        |[system, video]| {
            rc2k_core::scene::textures::count_needs(system, video, |level| {
                crate::scene::texture_places_store::cell_at(
                    &OBJECT_TABLES.level_sizes[0],
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
) -> Result<(), BadInput> {
    let totals = totals()?;
    for (cell, total) in TEXTURE_PLACES.totals.iter().zip(totals) {
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
pub fn count_texture_needs_in_game() -> Result<(), BadInput> {
    count_texture_needs(&mut |piece, newline| match piece {
        Ok(text) => crate::logfile::log_write::log_nul(&text, newline, 1),
        Err(number) => log_int(number as i32, newline, 1),
    })
}
