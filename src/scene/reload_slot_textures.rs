//! `scene::reload_slot_textures` (0x425669): every texture slot is loaded anew ("reload video plates").
//!
//! In:  nothing
//! Out: as `scene::load_wanted_textures` leaves them
//!
//! Two lines in the game's log (a rule and "reload video plates - start");
//! `scene::clear_slot_textures`; `scene::load_wanted_textures`; two lines more
//! ("reload video plates - finis" and a rule).
//! Two callers (the front end's 3D set-up and a race's).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::game_ptr;
use crate::framework::rva;
use crate::logfile::log_write::log_write;
use crate::scene::clear_slot_textures::clear_slot_textures;
use crate::scene::load_wanted_textures::load_wanted_textures;

// ---- Rust version ----

/// Its calls in order: two log lines, the slots cleared, the wanted textures loaded, two
/// log lines. What it leaves in the registers no caller reads (register liveness,
/// 2026-10-23: `race::set_all_plates` and `scene::set_up_front_end_scene`).
pub unsafe fn reload_slot_textures() -> Result<(), BadInput> {
    unsafe {
        log_write(game_ptr(rva(0x4DB858)), true, 1);
        log_write(game_ptr(rva(0x4DB87D)), true, 1);
        clear_slot_textures();
        load_wanted_textures()?;
        log_write(game_ptr(rva(0x4DB899)), true, 1);
        log_write(game_ptr(rva(0x4DB8B5)), true, 1);
        Ok(())
    }
}
