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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::listed_car_flags::{CURRENT_CAR, TABLE, set_flag_at, twin_copy};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use rc2k_core::physics::progress::{GO_NOTICE, GoFacts};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::start_go_notice", rva(0x436A4B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
        [g(CURRENT_CAR), g(TABLE), g(0x5165A4)],
    )
}

/// The Rust version.
pub unsafe fn start_go_notice() -> Result<Left, Refused> {
    let (v, twin) = game();
    unsafe { start_go_notice_at(&v, twin) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { start_go_notice() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let (v, twin) = game();
    let regions: Vec<Region> = vec![(v[2], 4), (v[5], 4), (v[6], 4), (v[7], 4), (twin[2], 4)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The 8 variables, then current, flag, 8 listed.
    let mut cells = vec![0u32; 8 + 2 + 8];
    let base = cells.as_mut_ptr() as u32;
    let v: Vec<u32> = (0..8).map(|k| base + 4 * k).collect();
    let twin = [base + 0x20, base + 0x28, base + 0x24];
    let inner = match twin_copy(
        &crate::physics::set_flag_5165a4::HOOK,
        0x45D189,
        0x5165A4,
        twin[0],
        twin[1],
        twin[2],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let vars: Vec<(usize, u32)> = VARIABLES.iter().copied().zip(v.iter().copied()).collect();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x436A4B,
            len: 0x5C,
            entry: &[0x83, 0x3D, 0x34, 0xA2, 0x71, 0x00, 0x01],
            calls: &[0x2E],
            jumps_out: &[],
        },
        &|r| (r == rva(0x45D189)).then_some(target),
        &vars,
        9,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    // Case 0 runs; each other case breaks one condition.
    for case in 0..7u32 {
        let fill = || unsafe {
            for k in 0..18u32 {
                put(base + 4 * k, 0x5A5A_0000 | k);
            }
            put(v[0], if case == 1 { 2 } else { 1 });
            put(v[1], if case == 2 { 5 } else { 0 });
            put(v[2], if case == 3 { 1 } else { 0 });
            put(v[3], if case == 4 { 1 } else { 0 });
            put(v[4], if case == 5 { 0x0001_0000 } else { 0xFFFE_FFFF });
            put(twin[0], case % 8);
            if case == 6 {
                put(twin[1] + 4 * (case % 8), u32::MAX);
            }
        };
        fill();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        fill();
        let left = unsafe { start_go_notice_at(&v, twin) };
        let mine = capture(&all);
        let regs_ok = left.is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
        t.check(regs_ok && theirs == mine, || {
            format!("case {case}: regs_ok={regs_ok}; the memory differs")
        });
    }
    drop((copy, inner, cells));
    t.finish()
}
