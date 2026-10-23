//! `drive::checkpoint_taken` (0x4375A5): whether checkpoint eax is no place to start a
//! car: CF when it is within 8 of the stage's start or end (`g_checkpointCount` - 8; the
//! sign of the differences) or `drive::car_near_checkpoint` finds one of the four cars
//! near it (blocks 0x70F3EC, 0x7103CC, 0x7113AC, 0x71238C). Inside `pusha` / `popa`.
//!
//! In:  eax the checkpoint
//! Out: CF; every register kept (ZF not read: `jb`)
//!
//! A `js` is in its first five bytes: `.private_copy` (its four calls relocated) with
//! `.manual_patch`.
//! Caller: `sub_43703E` (0x4371A1, 0x437284). Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original (its four blocks and the count the test's, its
//! calls to `car_near_checkpoint` kept) against the Rust core, 60 cases.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::car_near_checkpoint::near_at;
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::physics::flag_result::FlagResult;
use rc2k_core::physics::line_places;

/// The four cars' blocks (IDA addresses).
const CARS: [usize; 4] = [0x70F3EC, 0x7103CC, 0x7113AC, 0x71238C];
const COUNT_RVA: usize = rva(0x623640); // g_checkpointCount

/// The Rust core: the four cars at `cars`, the count `count`; `r` at entry.
// Converted: `rc2k_core::physics::line_places::checkpoint_taken`.
pub unsafe fn taken_at(cars: [u32; 4], count: u32, r: &Regs) -> FlagResult {
    let result = |carry| FlagResult {
        eax: r.eax,
        ecx: None,
        ebx: None,
        carry,
        zero: None,
    };
    result(line_places::checkpoint_taken(r.eax, count, || {
        cars.iter().any(|&car| unsafe { near_at(car, r) }.carry)
    }))
}

/// The Rust version.
pub unsafe fn checkpoint_taken(r: &Regs) -> FlagResult {
    let cars = CARS.map(|a| game_addr(rva(a)) as u32);
    unsafe { taken_at(cars, game_read(COUNT_RVA), r) }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { checkpoint_taken(r) })
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::physics::flag_result::flags_entry(r, &rust)
}
