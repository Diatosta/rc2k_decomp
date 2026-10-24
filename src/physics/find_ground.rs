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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::ground_search::ground_search;

/// The Rust version: the carry (the registers are kept).
pub unsafe fn find_ground() -> Result<bool, Refused> {
    unsafe { ground_search() }
}
