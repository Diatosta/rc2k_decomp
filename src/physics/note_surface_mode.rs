//! `physics::note_surface_mode` (0x43F180): `dword_71A6C8` = 1 when `dword_71A65C` is 2
//! or 3, else 0. What the mode is is not known.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `physics::feel_ground` (0x43F1A8), first.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), modes 0 .. 5.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::cells::{get, put};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::ground::surface_flag;

const VARIABLES: [usize; 2] = [0x71A65C, 0x71A6C8];

/// The Rust version on the variables at `v`.
// Converted: `rc2k_core::physics::ground::surface_flag`.
pub unsafe fn note_at(v: &[u32]) {
    unsafe { put(v[1], surface_flag(get(v[0]))) }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn note_surface_mode() {
    unsafe { note_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { note_surface_mode() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
