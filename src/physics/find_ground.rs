//! `physics::find_ground` (0x441E29): `call physics::ground_search; retn`.
//!
//! In:  the point (`dword_639824`, `dword_63982C`), the face mask `dword_620470`, the
//!      last face `dword_5CCC78`
//! Out: every register kept; `g_groundFound`, `dword_5CCC74`, the face's shares and
//!      height (`dword_639828`); CF not read by its callers
//!
//! See `physics/ground.rs`. Callers (7): `sub_45D305` (the car's ground points),
//! `sub_462485`, `sub_45EB06`, `sub_458078`, `sub_44ECE4`, `sub_43A1CA`, `sub_438E22`.
//! Refuses what `ground_search` refuses.
//! Self-test: a copy of the original (its call to a copy of `ground_search`) on the
//! test's ground, as `ground_search`'s test.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::ground_search::ground_search;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: the carry (the registers are kept).
pub unsafe fn find_ground() -> Result<bool, Refused> {
    unsafe { ground_search() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { find_ground() }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
