//! `physics::keep_split_time` (0x436B3D): the work car's time (`g_car` +0x88,
//! `unk_71A060`) kept in `dword_601C68[k]`, k = +0x90 (`unk_71A068`) less one, held to 0
//! ..= `g_splitColourCount` (both by the sign of the difference). A reading: a split
//! time at a checkpoint.
//!
//! In:  nothing
//! Out: eax the time, ebx k; the rest kept; no flags read (the caller `popa`s)
//!
//! Caller: `sub_436833`.
//! Refuses (before anything) a k past the table's 0x40 entries (the original writes past
//! it; race init clears 0x40).
//! Self-test: a copy of the original (its variables and table the test's) against the
//! core: k below, inside and past the count.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use rc2k_core::physics::splits::kept_split_slot;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::keep_split_time", rva(0x436B3D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Where the variables are: the time, the checks, the count, the table.
#[derive(Clone, Copy)]
pub struct SplitPlaces {
    pub time: u32,
    pub checks: u32,
    pub count: u32,
    pub table: u32,
}

impl SplitPlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            time: g(0x71A060),
            checks: g(0x71A068),
            count: g(0x601830),
            table: g(0x601C68),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version on the variables at `p`.
// Converted: `rc2k_core::physics::splits::kept_split_slot`.
pub unsafe fn keep_split_time_at(p: &SplitPlaces) -> Result<Left, Refused> {
    unsafe {
        let time = at(p.time);
        let k = kept_split_slot(at(p.checks), at(p.count))?;
        ((p.table + 4 * k) as *mut u32).write_unaligned(time);
        Ok(Left::nothing().with(EAX, time).with(EBX, k))
    }
}

/// The Rust version.
pub unsafe fn keep_split_time() -> Result<Left, Refused> {
    unsafe { keep_split_time_at(&SplitPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { keep_split_time() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(SplitPlaces::game().table, 0x100)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 3 + 0x40];
    let base = cells.as_mut_ptr() as u32;
    let p = SplitPlaces {
        time: base,
        checks: base + 4,
        count: base + 8,
        table: base + 12,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x436B3D,
            len: 0x2C,
            entry: &[0xA1, 0x60, 0xA0, 0x71, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x71A060, p.time),
            (0x71A068, p.checks),
            (0x601830, p.count),
            (0x601C68, p.table),
        ],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    for (checks, count) in [
        (0u32, 5u32),
        (1, 5),
        (4, 5),
        (6, 5),
        (9, 5),
        (0x8000_0000, 3),
        (3, 0x3F),
    ] {
        let fill = || unsafe {
            for k in 0..(3 + 0x40u32) {
                ((base + 4 * k) as *mut u32).write(0xA5A5_0000 | k);
            }
            (p.checks as *mut u32).write(checks);
            (p.count as *mut u32).write(count);
        };
        fill();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        fill();
        let left = unsafe { keep_split_time_at(&p) };
        let mine = capture(&all);
        let regs_ok = left.is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
        t.check(regs_ok && theirs == mine, || {
            format!("checks {checks:#x}, count {count}: regs_ok={regs_ok}; the memory differs")
        });
    }
    unsafe {
        (p.checks as *mut u32).write(0x50);
        (p.count as *mut u32).write(0x50);
    }
    let refused = unsafe { keep_split_time_at(&p) }.is_err();
    t.check(refused, || "index 0x4F not refused".into());
    drop((copy, cells));
    t.finish()
}
