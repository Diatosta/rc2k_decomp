//! `physics::set_flag_51659c` (0x45D151): `dword_51659C` 1 when the current car's entry in
//! `dword_641680` is not -1. See `physics/listed_car_flags.rs` (the core, the test).
//!
//! In:  nothing
//! Out: eax the entry; the rest kept
//!
//! Caller: `sub_436833`.
//! Refuses (before anything) a current car past 7.
//! Self-test: `listed_car_flags::test_twin`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::physics::listed_car_flags::set_flag;
use crate::screens::harness::Left;

const FLAG: usize = 0x51659C; // dword_51659C

/// The Rust version.
pub unsafe fn set_flag_51659c() -> Result<Left, Refused> {
    unsafe { set_flag(FLAG) }
}
