//! The race's split times and timers under `sub_43B635` (IDA 2026-10-18):
//! `physics::race_timers` (0x436790) by the race mode (`dword_62389C`) runs the start's
//! counters and, at each new checkpoint split, `physics::take_split` (0x436900) or for
//! mode 2 `physics::take_split_counted` (0x436833); those ask `physics::split_by_mode`
//! (0x436B99) for the split's figure: mode 2 the car's place, mode 1
//! `physics::rank_split_time` (0x436C4F, against the 90 rival times of
//! `physics::fill_rival_table`, 0x436BD6), mode 3 `physics::compare_split_times` (0x436E11,
//! against ten times kept per split). This file: the shared names and the test.

use crate::errors::Refused;
use crate::framework::{Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, restore};

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

/// What the tests save, set and compare.
pub fn test_regions(source: u32) -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x719FD8), 0xFE0),
        (a(0x6043F8), 4),
        (a(0x75B02C), 0xE0),
        (a(0x6D1838), 0x14),
        (a(TEXT), 0x40),
        (a(0x601D68), 0x168),
        (a(0x601ED0), 0x168),
        (a(0x602038), 0x168),
        (a(0x6021A0), 0x16C),
        (a(0x614170), 4),
        (a(0x51659C), 12),
        (a(0x601C68), 0x100),
        (a(0x6238BC), 4),
        (a(0x604134), 0x118),
        (a(0x60410C), 0x28),
        (a(0x60102C), 0x110),
        (a(COUNT), 4),
        (a(0x601434), 0x100),
        (a(0x601028), 4),
        (a(MODE), 4),
        (a(0x71B3F8), 4),
        (a(0x641680), 0x20),
        (a(0x70FA40), 4),
        (a(0x6D1AEC), 4),
        (source, 0x5A * 0x50),
    ]
}

/// What the steps write, saved before a step and put back when it refuses.
pub fn written() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x719FD8), 0xFE0),
        (a(0x6043F8), 4),
        (a(0x75B02C), 0xE0),
        (a(0x6D1838), 0x14),
        (a(TEXT), 0x40),
        (a(0x601D68), 0x168),
        (a(0x601ED0), 0x168),
        (a(0x602038), 0x168),
        (a(0x6021A0), 0x16C),
        (a(0x614170), 4),
        (a(0x51659C), 12),
        (a(0x601C68), 0x100),
        (a(0x6D1AEC), 4),
    ]
}

/// Runs `step` on a copy of `entry`; on a refusal what was written is put back (the
/// text the strings may have run past 0x40 into is not).
pub fn guarded(
    entry: &Regs,
    step: &dyn Fn(&mut Regs) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    let all = written();
    let saved = capture(&all);
    let mut r = *entry;
    match step(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

/// One case's settings.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub mode: u32,
    pub car: u32,
    pub split: u32,
    pub checks: u32,
    pub count: u32,
    pub time: u32,
    pub done: u32,
    pub table: bool,
}

/// Puts a case in place (`source` a test rival table of 90 x 0x50 bytes).
pub fn set_case(c: &Case, source: u32) {
    p(MODE, c.mode);
    p(0x71B3F8, c.car);
    p(SPLIT, c.split);
    p(CHECKS, c.checks);
    p(COUNT, c.count);
    p(TIME, c.time);
    p(DONE, c.done);
    p(0x71A078, 0);
    p(0x71A07C, 1);
    p(0x71A064, 0);
    p(0x71A234, 1);
    p(0x719FF4, 0);
    p(0x719FEC, 0x400);
    p(0x71A19C, 0.5f32.to_bits());
    p(0x601028, 0.75f32.to_bits());
    p(0x70FA40, 3);
    for k in 0..8usize {
        p(
            0x641680 + 4 * k,
            if k % 3 == 0 { u32::MAX } else { k as u32 },
        );
    }
    for k in 0..0x44usize {
        p(0x60102C + 4 * k, 1000 + 37 * k as u32);
    }
    for k in 0..0x40usize {
        p(0x601434 + 4 * k, k as u32 % 7);
    }
    for k in 0..0x46usize {
        p(0x604134 + 4 * k, 9000 + (k as u32 * 977) % 20000);
    }
    for k in 0..10usize {
        p(0x60410C + 4 * k, 0x100 + k as u32);
    }
    unsafe {
        for i in 0..0x5Au32 {
            let e = source + 0x50 * i;
            for k in 0..20u32 {
                ((e + 4 * k) as *mut u32).write(0);
            }
            (e as *mut u32).write(0x5A5 + (i * 7) % 0x5A);
            ((e + 8) as *mut u32).write(i % 23);
            ((e + 12) as *mut u32).write(20_000 + 300 * i);
        }
    }
    p(0x6238BC, if c.table { source } else { 0 });
}

/// The cases.
pub fn cases() -> Vec<Case> {
    let mut out = vec![];
    for mode in 0..5u32 {
        for (split, checks, count) in [
            (0u32, 1u32, 5u32),
            (2, 3, 5),
            (4, 5, 5),
            (3, 3, 5),
            (1, 2, 2),
        ] {
            for (n, time) in [500u32, 15_000, 40_000, 200_000].into_iter().enumerate() {
                out.push(Case {
                    mode,
                    car: (n as u32 + split) % 3,
                    split,
                    checks,
                    count,
                    time,
                    done: (n == 3) as u32,
                    table: n % 2 == 1,
                });
            }
        }
    }
    out
}

/// The test of one of them on the game's variables (saved and put back).
pub fn test_splits(
    t: &mut SelfTest,
    original: unsafe extern "C" fn(*mut Regs),
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
    only: &dyn Fn(&Case) -> bool,
) {
    let mut source = vec![0u32; 0x5A * 0x50 / 4];
    let s = source.as_mut_ptr() as u32;
    let all = test_regions(s);
    let saved = capture(&all);
    for c in cases().iter().filter(|c| only(c)) {
        check_state(
            t,
            &format!("{c:?}"),
            original,
            Regs::SENTINEL,
            &|| set_case(c, s),
            &all,
            rust,
        );
    }
    restore(&all, &saved);
    drop(source);
}
