//! `physics::keep_split_time` (0x436B3D): the work car's time (`g_car` +0x88,
//! `unk_71A060`) kept in `dword_601C68[k]`, k = +0x90 (`unk_71A068`) less one, held to 0
//! ..= `g_splitColourCount` (both by the sign of the difference). A reading: a split
//! time at a checkpoint.
//!
//! In:  nothing
//! Out: eax the time, ebx k; the rest kept; no flags read (the caller `popa`s)
//!
//! Caller: `sub_436833`.
//! Refuses (before anything) a k past the table's 0x40 entries (the original writes past
//! it; race init clears 0x40).
//! Self-test: a copy of the original (its variables and table the test's) against the
//! core: k below, inside and past the count.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::splits::kept_split_slot;

/// Where the variables are: the time, the checks, the count, the table.
#[derive(Clone, Copy)]
pub struct SplitPlaces {
    pub time: u32,
    pub checks: u32,
    pub count: u32,
    pub table: u32,
}

impl SplitPlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            time: g(0x71A060),
            checks: g(0x71A068),
            count: g(0x601830),
            table: g(0x601C68),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version on the variables at `p`.
// Converted: `rc2k_core::physics::splits::kept_split_slot`.
pub unsafe fn keep_split_time_at(p: &SplitPlaces) -> Result<Left, Refused> {
    unsafe {
        let time = at(p.time);
        let k = kept_split_slot(at(p.checks), at(p.count))?;
        ((p.table + 4 * k) as *mut u32).write_unaligned(time);
        Ok(Left::nothing().with(EAX, time).with(EBX, k))
    }
}

/// The Rust version.
pub unsafe fn keep_split_time() -> Result<Left, Refused> {
    unsafe { keep_split_time_at(&SplitPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { keep_split_time() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
