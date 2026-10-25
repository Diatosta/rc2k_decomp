//! The race's split times and timers under `sub_43B635` (IDA 2026-10-18):
//! `physics::race_timers` (0x436790) by the race mode (`dword_62389C`) runs the start's
//! counters and, at each new checkpoint split, `physics::take_split` (0x436900) or for
//! mode 2 `physics::take_split_counted` (0x436833); those ask `physics::split_by_mode`
//! (0x436B99) for the split's figure: mode 2 the car's place, mode 1
//! `physics::rank_split_time` (0x436C4F, against the 90 rival times of
//! `physics::fill_rival_table`, 0x436BD6), mode 3 `physics::compare_split_times` (0x436E11,
//! against ten times kept per split). This file: the shared names and the test.

use crate::framework::{game_read, game_write, rva};

pub const MODE: usize = 0x62389C; // dword_62389C
pub const COUNT: usize = 0x601830; // g_splitColourCount

/// `g_car + 0x140 + 4 split`: the splits run to 0x3F, past the sixteen figures of
/// `split_times` (into the car's next fields), as the original writes them.
pub fn put_split_time(split: usize, v: u32) {
    unsafe {
        (crate::physics::car_store::CAR.split_times.as_ptr() as *mut u32)
            .add(split)
            .write_unaligned(v)
    }
}

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}
