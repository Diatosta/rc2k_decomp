//! `physics::note_time_mark` (0x440C38): when `flt_71AEE0` is not below `flt_5C5D5C`
//! (60.0; `fcomp`, C0 clear: also not unordered) and `dword_6140EC` is 0:
//! `dword_6140EC` = `dword_719FE0` (the work copy's +8, a step count by `sub_460835`'s
//! `inc`) less `dword_6140F0`. By the look of it the step at which a figure first
//! reached 60 (a reading).
//!
//! In:  nothing
//! Out: eax the difference when it is stored, else kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B9A4), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::framework::{game_addr, game_read, rva};
use rc2k_core::physics::motion::time_mark;

/// The figure, the mark, the count, the start (IDA addresses).
const VARIABLES: [usize; 4] = [0x71AEE0, 0x6140EC, 0x719FE0, 0x6140F0];
const LIMIT_RVA: usize = rva(0x5C5D5C); // flt_5C5D5C, 60.0

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The converted version (`rc2k_core::physics::motion`) on the variables at `v`. The eax
/// it leaves (the mark) is read by none of its callers.
// Converted: `rc2k_core::physics::motion::time_mark`.
unsafe fn core(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let limit = f32::from_bits(game_read(LIMIT_RVA));
        // The mark, count and start read only where the original reads them.
        if let Some(mark) = time_mark(
            f32::from_bits(get(0)),
            limit,
            || get(1),
            || (get(2), get(3)),
        ) {
            (v[1] as *mut u32).write_unaligned(mark);
        }
    }
}

/// The Rust version (a compare of floats only: the same at any precision); `r` as the
/// original leaves are read by none of its callers.
pub unsafe fn note_time_mark() {
    unsafe { core(&game()) }
}
