//! The race's split times and timers under `sub_43B635` (IDA 2026-10-18):
//! `physics::race_timers` (0x436790) by the race mode (`dword_62389C`) runs the start's
//! counters and, at each new checkpoint split, `physics::take_split` (0x436900) or for
//! mode 2 `physics::take_split_counted` (0x436833); those ask `physics::split_by_mode`
//! (0x436B99) for the split's figure: mode 2 the car's place, mode 1
//! `physics::rank_split_time` (0x436C4F, against the 90 rival times of
//! `physics::fill_rival_table`, 0x436BD6), mode 3 `physics::compare_split_times` (0x436E11,
//! against ten times kept per split). This file: the shared names and the test.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::screens::harness::Left;

pub const MODE: usize = 0x62389C; // dword_62389C
pub const TIME: usize = 0x71A060; // g_car + 0x88
pub const CHECKS: usize = 0x71A068; // g_car + 0x90
pub const SPLIT: usize = 0x71A084; // g_car + 0xAC: splits taken
pub const DONE: usize = 0x71A080; // g_car + 0xA8
pub const FIGURE: usize = 0x71A9E4; // g_car + 0xA0C: the split's figure, then six more
pub const SPLITS: usize = 0x71A118; // g_car + 0x140: a figure per split
pub const COUNT: usize = 0x601830; // g_splitColourCount
pub const TEXT: usize = 0x93E430; // text

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// Runs `step` on a copy of `entry`; on a refusal what was written is put back (the
/// text the strings may have run past 0x40 into is not).
pub fn guarded(
    entry: &Regs,
    step: &dyn Fn(&mut Regs) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    let mut r = *entry;
    match step(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => Err(e),
    }
}
