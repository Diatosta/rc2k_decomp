//! `race::wake_part_sounds` (0x45D1EA): for each race car (car 0 at least) and each of
//! its six slots, when +0x48 is 0 and +0x700 at least 10, +0x48 = 1; when +0x700 was
//! exactly 10 (the `jnz` after the `mov` reads the `cmp`'s flags) and the current car is
//! shown (`dword_641680[g_currentCar]` not -1), the car's sound object
//! (`g_raceSoundObjects[car & 7]`) +0x264 = 1 and +0x10 = 1.
//!
//! In:  nothing
//! Out: ecx the cars done, ebx 6, esi the last block, edi its sound object, eax the last
//!      view read (a `mov` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45429B` (0x454747), each frame drawn. What the six
//! slots are (+0x700 counts up; a part's sound, a reading) is not settled. Refuses (before
//! anything) more than eight cars, a null block, a null sound object it would write, and
//! `g_currentCar` past 7 where it is read.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// g_raceSoundObjects
// dword_641680[8]

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { wake_part_sounds() }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
