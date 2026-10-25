//! `physics::share_engine_sound` (0x45D294): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`): when `flt_71A208` is above its +0xD0 (`fcomp`:
//! not equal, not below, not unordered), the work car's 0x71A1FC .. 0x71A208 into its
//! +0xC4 .. +0xD0; always `dword_71A1E0` into +0xD4 and +0x10 = 1. By the look of it the
//! engine's figures for the sounds (a reading).
//!
//! In:  nothing
//! Out: edi the sound object; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B99F), every physics step. Refuses (before anything) a null
//! sound object.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};
use rc2k_core::physics::engine::shares_figures;

/// The current car, the objects' table, the work car's 0x71A1FC .. 0x71A208, 0x71A1E0.
const VARIABLES: [usize; 7] = [
    0x71B3F8, 0, 0x71A1FC, 0x71A200, 0x71A204, 0x71A208, 0x71A1E0,
];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core on the variables at `v` (`VARIABLES`' order; a compare of floats only:
/// the same at any precision).
// Converted: `rc2k_core::physics::engine::shares_figures`.
pub unsafe fn share_at(v: &[u32]) -> Result<(), Refused> {
    unsafe {
        let o = get(v[1] + 4 * (get(v[0]) & 7));
        if o == 0 {
            return Err(BadInput("a null sound object").into());
        }
        let f = |a: u32| f32::from_bits(get(a));
        let so = crate::sound::store::SoundObject::at(o);
        if shares_figures(f(v[5]), f32::from_bits(so.knock_in[3].get())) {
            for k in 0..4 {
                so.knock_in[k].set(get(v[2 + k]));
            }
        }
        so.knock_d4.set(get(v[6]));
        so.new_revs.set(1);
        Ok(())
    }
}

fn game() -> Vec<u32> {
    let mut v: Vec<u32> = VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect();
    // The objects' table (`g_raceSoundObjects`) is the sound module's.
    v[1] = crate::sound::store::RACE_SOUNDS.table.as_ptr() as usize as u32;
    v
}

/// The Rust version.
pub unsafe fn share_engine_sound() -> Result<(), Refused> {
    unsafe { share_at(&game()) }
}
