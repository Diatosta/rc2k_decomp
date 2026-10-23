//! `physics::mark_car_started` (0x436AA7): `g_car` +0xA0 (`unk_71A078`) set to 1 when
//! it is 0 and +0x29C (`unk_71A274`) is not (a reading: the car has got going).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the callers return next)
//!
//! Caller: `sub_436790` (four calls).
//! Self-test: `physics/copy_hooks_test.rs`, 8 fillings over both words' zero and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::cells::{get, put_changed};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::start::CarStart;

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The variables (IDA addresses).
const VARIABLES: [usize; 2] = [0x71A078, 0x71A274];

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::start::CarStart::mark_started`.
unsafe fn core(v: &[u32]) -> Left {
    unsafe {
        let was = CarStart {
            started: get(v[0]),
            moving: get(v[1]),
            ..CarStart::default()
        };
        let mut c = was;
        c.mark_started();
        put_changed(v[0], was.started, c.started);
        Left::nothing()
    }
}

/// The Rust version.
pub unsafe fn mark_car_started() -> Left {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { mark_car_started() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
