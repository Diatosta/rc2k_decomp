//! `saves::default_class_times` (0x47C8E3): fills the table at `dword_7556DC` with made-up
//! entries.
//!
//! In:  nothing
//! Out: every register preserved
//! The table (0x15E0 bytes) is seven groups of ten entries of 0x50 bytes. In each group
//! the entries get 100,000 and then 0xAB3 less each time at +0x14, and a random driver
//! (`sub_47CB2D`, a plain function here: a number under 0xAA): the driver's +8 to the
//! entry's +0x1C, +0x14 to +8, +0x18 to +0x10, the id of the name's string to +0xC and
//! the name (without a NUL) to +0x20.
//! What the table is has not been worked out; "class times" is a guess.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table and a name that does not fit.
//! Converted: `rc2k_core::saves::defaults::class_times`.

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::saves::default_times::{GameSource, to_nowhere};
use rc2k_core::saves::defaults::{CLASS_TABLE_LEN, class_times, write_class_times};

// ---- game data ----

// ---- Rust version ----

pub unsafe fn default_class_times() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::defaults::class_times`.
    unsafe {
        let table: u32 = CHAMP_VIEW.level_best_scores.get();
        if table == 0 {
            return Err(BadInput("no table").into());
        }
        let entries = class_times(&mut GameSource {
            log: &mut to_nowhere,
        })?;
        write_class_times(
            std::slice::from_raw_parts_mut(table as *mut u8, CLASS_TABLE_LEN),
            &entries,
        );
        Ok(())
    }
}
