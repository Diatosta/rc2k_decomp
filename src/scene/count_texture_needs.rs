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
use crate::framework::{game_read, game_write, rva};
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::scene::texture_needs::{LEVEL_SIZES_RVA, SYSTEM_LEVELS_RVA, TEXTURES, VIDEO_LEVELS_RVA};

// ---- game data ----

/// `dword_5DC968` video k, `dword_5DC96C` system k, `dword_5DC970` video textures,
/// `dword_5DC974` system textures.
const TOTALS_RVA: usize = rva(0x5DC968);

// ---- Rust version ----

/// The four totals. Reads only.
unsafe fn totals() -> Result<[u32; 4], BadInput> {
    // Converted: `rc2k_core::scene::textures::count_needs`.
    use crate::scene::textures::slice;
    let n = TEXTURES as u32;
    let (totals, _) = unsafe {
        rc2k_core::scene::textures::count_needs(
            slice(SYSTEM_LEVELS_RVA, n),
            slice(VIDEO_LEVELS_RVA, n),
            |level| game_read(LEVEL_SIZES_RVA + level * 4),
        )
    }?;
    Ok(totals.to_array())
}

/// `log`: what writes a piece of the line: a text, or a number.
pub unsafe fn count_texture_needs(
    log: &mut dyn FnMut(Result<&'static std::ffi::CStr, u32>, bool),
) -> Result<(), BadInput> {
    unsafe {
        let totals = totals()?;
        game_write(TOTALS_RVA, totals);
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
}

/// Into the game's log.
pub unsafe fn count_texture_needs_in_game() -> Result<(), BadInput> {
    unsafe {
        count_texture_needs(&mut |piece, newline| match piece {
            Ok(text) => log_write(text.as_ptr().cast(), newline, 1),
            Err(number) => log_int(number as i32, newline, 1),
        })
    }
}
