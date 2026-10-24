//! `physics::share_engine_sound` (0x45D294): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`): when `flt_71A208` is above its +0xD0 (`fcomp`:
//! not equal, not below, not unordered), the work car's 0x71A1FC .. 0x71A208 into its
//! +0xC4 .. +0xD0; always `dword_71A1E0` into +0xD4 and +0x10 = 1. By the look of it the
//! engine's figures for the sounds (a reading).
//!
//! In:  nothing
//! Out: edi the sound object; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B99F), every physics step.
//! Refuses (before anything) a null sound object.
//! Self-test: a copy of the original (the current car, the objects' table and the work
//! car's five the test's) on a test object against the converted version, values
//! above, at, below and NaN.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EDI, game_addr, rva};
use crate::screens::harness::Left;
use rc2k_core::physics::engine::shares_figures;

/// The current car, the objects' table, the work car's 0x71A1FC .. 0x71A208, 0x71A1E0.
const VARIABLES: [usize; 7] = [
    0x71B3F8, 0, 0x71A1FC, 0x71A200, 0x71A204, 0x71A208, 0x71A1E0,
];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `v` (`VARIABLES`' order; a compare of floats only:
/// the same at any precision).
// Converted: `rc2k_core::physics::engine::shares_figures`.
pub unsafe fn share_at(v: &[u32]) -> Result<Left, Refused> {
    unsafe {
        let o = get(v[1] + 4 * (get(v[0]) & 7));
        if o == 0 {
            return Err(BadInput("a null sound object").into());
        }
        let f = |a: u32| f32::from_bits(get(a));
        if shares_figures(f(v[5]), f(o + 0xD0)) {
            for k in 0..4 {
                put(o + 0xC4 + 4 * k, get(v[2 + k as usize]));
            }
        }
        put(o + 0xD4, get(v[6]));
        put(o + 0x10, 1);
        Ok(Left::nothing().with(EDI, o))
    }
}

fn game() -> Vec<u32> {
    let mut v: Vec<u32> = VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect();
    // The objects' table (`g_raceSoundObjects`) is the sound module's.
    v[1] = crate::sound::store::OBJECT_TABLE.address() as u32;
    v
}

/// The Rust version.
pub unsafe fn share_engine_sound() -> Result<Left, Refused> {
    unsafe { share_at(&game()) }
}
