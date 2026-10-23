//! `marks::load_splines` (0x4B96CD): in step 1, `files\m\spline.dat` read once and a spline
//! made for each of the settings' objects.
//!
//! In:  nothing
//! Out: esi the settings, the rest as the last `marks::make_spline` leaves them
//!
//! See `marks/splines.rs`. One caller, `sub_4C1235` (`init_marks`' step 1). Its once-only
//! mark (`g_marksSplinesLoaded`) holds `loc_4B9706` after the first call. Refuses a mark
//! neither 0 nor that, a kind of 0x22 or more, more than 19 objects, and what
//! `make_spline` refuses for any object made, before writing anything. Self-test: the
//! original and the Rust version on the scene of `marks::measure_lines` (0, 1, 2 and 9
//! objects; the file read and not), the game's file and tables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let resume = game_addr(RESUME_RVA) as u32;
    Ok(left_of(&unsafe {
        load_splines(r, resume, &mut crate::race::level_init::game_part)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
