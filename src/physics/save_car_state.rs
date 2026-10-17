//! `physics::save_car_state` (0x43DBD5): the counterpart of `physics::restore_saved_car`:
//! `dword_614738` = 0; `dword_71A10C` .. `114` into `dword_6141E0` .. `E8`;
//! `g_crashMatrix` (0x40 bytes) into `unk_6141EC`; `dword_71A238` dwords of
//! `dword_71ADC8` (the probe points' faces) into `dword_6D2814`.
//!
//! In:  nothing
//! Out: eax = `dword_71A114`, ecx 0, esi = 0x71ADC8 + 4 n, edi = 0x6D2814 + 4 n; no flags
//!      read
//!
//! Caller: `sub_43B635` (0x43B719), every physics step.
//! Refuses a count above 0x40 (unsigned; `dword_6D2814` has 0x40 before
//! `dword_6D2914`), before writing.
//! Self-test: a copy of the original whose twelve references are the test's, counts 0 ..
//! 0x40; the refusal.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, ECX, EDI, ESI, RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::physics::restore_saved_car::SavedCar;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::save_car_state", rva(0x43DBD5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The most the saved list holds.
pub const MOST_SAVED: u32 = 0x40;

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn copy_dwords(from: u32, to: u32, n: u32) {
    for k in 0..n {
        unsafe { ((to + 4 * k) as *mut u32).write_unaligned(at(from + 4 * k)) };
    }
}

/// The Rust core on the variables at `p` (`restore_saved_car`'s, the other way).
pub unsafe fn save_car_state_at(p: &SavedCar) -> Result<Left, Refused> {
    unsafe {
        let n = at(p.count);
        if n > MOST_SAVED {
            return Err(
                BadInput("a list of more than 0x40 dwords (past dword_6D2814's room)").into(),
            );
        }
        (p.counter as *mut u32).write_unaligned(0);
        copy_dwords(p.place, p.saved_place, 3);
        copy_dwords(p.matrix, p.saved_matrix, 0x10);
        copy_dwords(p.list, p.saved_list, n);
        Ok(Left::nothing()
            .with(EAX, at(p.place + 8))
            .with(ECX, 0)
            .with(ESI, p.list + 4 * n)
            .with(EDI, p.saved_list + 4 * n))
    }
}

/// The Rust version.
pub unsafe fn save_car_state() -> Result<Left, Refused> {
    unsafe { save_car_state_at(&SavedCar::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { save_car_state() }
}

fn regions(p: &SavedCar) -> Vec<Region> {
    vec![
        (p.counter, 4),
        (p.saved_place, 12),
        (p.saved_matrix, 0x40),
        (p.saved_list, 4 * MOST_SAVED as usize),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(&SavedCar::game()), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let len = 1 + 3 + 3 + 16 + 16 + 0x80 + 0x80 + 1;
    let mut cells = vec![0u32; len];
    let base = cells.as_mut_ptr() as u32;
    let p = SavedCar {
        counter: base,
        saved_place: base + 4,
        place: base + 16,
        saved_matrix: base + 28,
        matrix: base + 92,
        saved_list: base + 156,
        list: base + 156 + 0x200,
        count: base + 156 + 0x400,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43DBD5,
            len: 0x50,
            entry: &[0xC7, 0x05, 0x38, 0x47, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x614738, p.counter),
            (0x6141E0, p.saved_place),
            (0x6141E4, p.saved_place + 4),
            (0x6141E8, p.saved_place + 8),
            (0x71A10C, p.place),
            (0x71A110, p.place + 4),
            (0x71A114, p.place + 8),
            (0x6141EC, p.saved_matrix),
            (0x71A08C, p.matrix),
            (0x6D2814, p.saved_list),
            (0x71ADC8, p.list),
            (0x71A238, p.count),
        ],
        12,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all: [Region; 1] = [(base, 4 * len)];
    let fill = |n: u32, seed: u32| {
        let cell = base as *mut u32;
        let mut s = seed;
        for k in 0..len {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            unsafe { cell.add(k).write_volatile(s) };
        }
        unsafe { (p.count as *mut u32).write_volatile(n) };
    };
    for (k, n) in [0u32, 1, 4, 7, 0x3F, 0x40].into_iter().enumerate() {
        let seed = k as u32 * 0x2545_F491 + 3;
        fill(n, seed);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        fill(n, seed);
        let mine = unsafe { save_car_state_at(&p) };
        let state = capture(&all);
        let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
        t.check(regs_ok && theirs == state, || {
            format!("count {n:#x}: regs_ok={regs_ok} {mine:?}; the memory differs")
        });
    }
    for n in [0x41u32, 0x8000_0000] {
        fill(n, 2);
        let before = capture(&all);
        let refused = unsafe { save_car_state_at(&p) }.is_err();
        t.check(refused && capture(&all) == before, || {
            format!("count {n:#x}: refused {refused}, or something written")
        });
    }
    drop((copy, cells));
    t.finish()
}
