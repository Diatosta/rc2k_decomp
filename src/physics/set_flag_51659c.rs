//! `physics::set_flag_51659c` (0x45D151): `dword_51659C` 1 when the current car's entry in
//! `dword_641680` is not -1. See `physics/listed_car_flags.rs` (the core, the test).
//!
//! In:  nothing
//! Out: eax the entry; the rest kept
//!
//! Caller: `sub_436833`. Refuses (before anything) a current car past 7.

use crate::errors::Refused;
use crate::physics::listed_car_flags::set_flag;

/// The Rust version.
pub unsafe fn set_flag_51659c() -> Result<(), Refused> {
    unsafe { set_flag(&crate::sound::store::STAGE_SOUNDS.once[0]) }
}
