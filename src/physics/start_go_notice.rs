//! `physics::start_go_notice` (0x436A4B): the start's "go". With the grades' mode
//! (`g_car` +0x25C) 1, the count +0x8C, +0xA4 (`unk_71A07C`) and +0xA8 all 0, and cheat
//! bit 16 of `g_cheats` clear (`bt`): `physics::set_flag_5165a4`, then +0xA4 1, the
//! notice +0x71C 0xC7, +0x730 0x32 and +0x72C -1.
//!
//! In:  nothing
//! Out: when it ran, eax as `set_flag_5165a4` leaves it; the rest kept; no flags read
//!      (the caller's next is a call)
//!
//! Caller: `sub_436790` (0x4367E3).
//! Refuses what `set_flag_5165a4` refuses.
//! Self-test: a copy of the original (its variables the test's, its call to the twin's
//! test copy on test cells) against the core, over each condition.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::physics::listed_car_flags::{CURRENT_CAR, TABLE, set_flag_at};
use crate::screens::harness::Left;
use rc2k_core::physics::progress::{GO_NOTICE, GoFacts};

/// The variables (IDA addresses) in the order the core takes them.
const VARIABLES: [usize; 8] = [
    0x71A234, 0x71A064, 0x71A07C, 0x71A080, 0x75870C, 0x71A6F4, 0x71A708, 0x71A704,
];

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The core: `v` the variables' addresses, then the twin's current, listed and flag.
// Converted: `rc2k_core::physics::progress::GoFacts::go_due`.
pub unsafe fn start_go_notice_at(v: &[u32], twin: [u32; 3]) -> Result<Left, Refused> {
    unsafe {
        let facts = GoFacts {
            mode: at(v[0]),
            countdown: at(v[1]),
            gone: at(v[2]),
            finished: at(v[3]),
            cheats: at(v[4]),
        };
        if !facts.go_due() {
            return Ok(Left::nothing());
        }
        let left = set_flag_at(twin[0], twin[1], twin[2])?;
        put(v[2], 1);
        put(v[5], GO_NOTICE);
        put(v[6], 0x32);
        put(v[7], u32::MAX);
        Ok(left)
    }
}

fn game() -> (Vec<u32>, [u32; 3]) {
    let g = |a: usize| game_addr(rva(a)) as u32;
    (
        VARIABLES.iter().map(|&x| g(x)).collect(),
        [
            g(CURRENT_CAR),
            g(TABLE),
            crate::sound::store::StageSlot::at(0x5165A4).address() as u32,
        ],
    )
}

/// The Rust version.
pub unsafe fn start_go_notice() -> Result<Left, Refused> {
    let (v, twin) = game();
    unsafe { start_go_notice_at(&v, twin) }
}
