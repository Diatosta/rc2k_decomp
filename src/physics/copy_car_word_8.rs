//! `physics::copy_car_word_8` (0x440C66): `g_car` +8 (`dword_719FE0`) into
//! `dword_6140F0`, and `dword_6140EC` 0 (in the gearbox: by the look of it the moment a
//! gear change began; a reading).
//!
//! In:  nothing
//! Out: eax the word; the rest kept; no flags read
//!
//! Caller: `sub_43C8AC`.
//! Self-test: `physics/copy_hooks_test.rs`, 6 fillings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The word, its copy, the cleared one (IDA addresses).
const VARIABLES: [usize; 3] = [0x719FE0, 0x6140F0, 0x6140EC];

unsafe fn core(v: &[u32]) -> Left {
    unsafe {
        let w = (v[0] as *const u32).read_unaligned();
        (v[1] as *mut u32).write_unaligned(w);
        (v[2] as *mut u32).write_unaligned(0);
        Left::nothing().with(EAX, w)
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn copy_car_word_8() -> Left {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { copy_car_word_8() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
