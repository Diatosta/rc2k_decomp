//! `scene::reload_slot_textures` (0x425669): every texture slot is loaded anew ("reload video plates").
//!
//! In:  nothing
//! Out: as `scene::load_wanted_textures` leaves them
//!
//! Two lines in the game's log (a rule and "reload video plates - start");
//! `scene::clear_slot_textures`; `scene::load_wanted_textures`; two lines more ("reload video
//! plates - finis" and a rule). Two callers (the front end's 3D set-up and a race's).

use crate::errors::BadInput;
use crate::logfile::log_write::log_write;
use crate::scene::clear_slot_textures::clear_slot_textures;
use crate::scene::load_wanted_textures::load_wanted_textures;

// ---- Rust version ----

/// Its calls in order: two log lines, the slots cleared, the wanted textures loaded, two
/// log lines. What it leaves in the registers no caller reads (register liveness,
/// 2026-10-23: `race::set_all_plates` and `scene::set_up_front_end_scene`).
pub unsafe fn reload_slot_textures() -> Result<(), BadInput> {
    unsafe {
        log_write(
            c"------------------------------------"
                .as_ptr()
                .cast_mut()
                .cast(),
            true,
            1,
        );
        log_write(
            c"reload video plates - start".as_ptr().cast_mut().cast(),
            true,
            1,
        );
        clear_slot_textures();
        load_wanted_textures()?;
        log_write(
            c"reload video plates - finis".as_ptr().cast_mut().cast(),
            true,
            1,
        );
        log_write(
            c"------------------------------------"
                .as_ptr()
                .cast_mut()
                .cast(),
            true,
            1,
        );
        Ok(())
    }
}
