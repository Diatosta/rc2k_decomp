//! `marks::make_spline` (0x4B9A32): the next record of each of the four spline tables made
//! for object eax.
//!
//! In:  eax the object
//! Out: as the last steps leave them, ebp the first table's record
//!
//! See `marks/splines.rs`. One caller, `sub_4B96CD` (each of the settings' objects). The
//! second and third tables get the first table's record 0's centre and angle, whatever the
//! place (section 16). Refuses a negative place, an object past 0xFF or faces past
//! `g_faces`, an angle `fsincos` cannot take, before writing anything but the object.
//! Self-test: the original and the Rust version on the scene of `marks::measure_lines`
//! (places 0, 3, 7 and 8, kinds and angles by lot), the tables filled by lot; everything
//! put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_spline(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
