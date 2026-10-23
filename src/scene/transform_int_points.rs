//! `scene::transform_int_points` (0x465829): ecx points of three whole numbers from esi
//! turned by the 3 x 3 matrix at ebp into edi.
//!
//! In:  ecx the count, esi from, edi to, ebp the matrix
//! Out: ecx -1 (one less with a negative count), esi and edi past the points
//!
//! See `scene/matrix3.rs`. Callers: `sub_4BA0AE`, `sub_4BA16C`, `sub_4BD1A1`, `sub_4BD5ED`,
//! `sub_4C1CC8`, `sub_4C2AA5` and code outside a function. Refuses ecx 0x80000000. Manual
//! patch with a private copy (0x53 bytes): its first instruction jumps to the loop's test,
//! and the loop starts at the third byte. Self-test: the original and the Rust version on
//! points and matrices by lot (apart and in place), the refusal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::*;
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { transform_int_points_plain(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
