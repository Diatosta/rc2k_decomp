//! `physics::restore_saved_car` (0x43DC25): puts a saved state back into the work copy:
//! `dword_614738` + 1; the three dwords `dword_6141E0` .. `E8` into `dword_71A10C` ..
//! `114`; the 0x40 bytes at `unk_6141EC` into `g_crashMatrix` (0x71A08C); and
//! `dword_71A238` dwords from `dword_6D2814` into `dword_71ADC8` (the copy's +0xDF0).
//! What the pieces are (a place, the car's matrix, a list) is a reading.
//!
//! In:  nothing
//! Out: eax = `dword_6141E8`, ecx = 0, esi = 0x6D2814 + 4 n, edi = 0x71ADC8 + 4 n (n the
//!      count); flags the `add`'s (no caller reads them)
//!
//! Callers (7): `sub_43BDD1`, `sub_45D737`, `sub_45D7B6` (twice), `sub_45E3BF`,
//! `sub_45EFF3`, `sub_45F703`, under `sub_4454D2` (most followed by `sub_43F1A8`).
//! Refuses a count above 0x7C (unsigned: the list would run past the work copy's end,
//! 0x71AFB8), before writing.
//! Self-test: a copy of the original whose twelve references are the test's, counts 0 ..
//! 0x7C; the refusal.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    ECX, EDI, ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::restore_saved_car",
    rva(0x43DC25),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The most dwords the list may have: from 0x71ADC8 to the copy's end.
pub const MOST_LISTED: u32 = (0x71AFB8 - 0x71ADC8) / 4;

/// The variables' addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct SavedCar {
    pub counter: u32,
    pub saved_place: u32,
    pub place: u32,
    pub saved_matrix: u32,
    pub matrix: u32,
    pub saved_list: u32,
    pub list: u32,
    pub count: u32,
}

impl SavedCar {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            counter: a(0x614738),
            saved_place: a(0x6141E0),
            place: a(0x71A10C),
            saved_matrix: a(0x6141EC),
            matrix: a(0x71A08C),
            saved_list: a(0x6D2814),
            list: a(0x71ADC8),
            count: a(0x71A238),
        }
    }

    /// What it writes, for a count of `n`.
    pub fn regions(&self, n: u32) -> Vec<Region> {
        vec![
            (self.counter, 4),
            (self.place, 12),
            (self.matrix, 0x40),
            (self.list, 4 * n.min(MOST_LISTED) as usize),
        ]
    }

    unsafe fn at(a: u32) -> u32 {
        unsafe { (a as *const u32).read_unaligned() }
    }
}

unsafe fn copy_dwords(from: u32, to: u32, n: u32) {
    for k in 0..n {
        unsafe {
            let v = ((from + 4 * k) as *const u32).read_unaligned();
            ((to + 4 * k) as *mut u32).write_unaligned(v);
        }
    }
}

/// The Rust core on the variables at `p`.
pub unsafe fn restore_saved_car_at(p: &SavedCar) -> Result<Left, Refused> {
    unsafe {
        let n = SavedCar::at(p.count);
        if n > MOST_LISTED {
            return Err(BadInput("a list of more than 0x7C dwords (past the work copy)").into());
        }
        let c = SavedCar::at(p.counter).wrapping_add(1);
        (p.counter as *mut u32).write_unaligned(c);
        copy_dwords(p.saved_place, p.place, 3);
        copy_dwords(p.saved_matrix, p.matrix, 0x10);
        copy_dwords(p.saved_list, p.list, n);
        Ok(Left::nothing()
            .with(crate::framework::EAX, SavedCar::at(p.saved_place + 8))
            .with(ECX, 0)
            .with(ESI, p.saved_list + 4 * n)
            .with(EDI, p.list + 4 * n))
    }
}

/// The Rust version.
pub unsafe fn restore_saved_car() -> Result<Left, Refused> {
    unsafe { restore_saved_car_at(&SavedCar::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { restore_saved_car() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let p = SavedCar::game();
    let n = unsafe { SavedCar::at(p.count) };
    dispatch_state(&HOOK, r, call_original, &p.regions(n), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // counter, saved place 3, place 3, saved matrix 16, matrix 16, saved list 0x80, list
    // 0x80, count.
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
            address: 0x43DC25,
            len: 0x4D,
            entry: &[0x83, 0x05, 0x38, 0x47, 0x61, 0x00, 0x01],
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
    for (k, n) in [0u32, 1, 2, 7, 0x40, 0x7B, 0x7C].into_iter().enumerate() {
        let seed = k as u32 * 0x2545_F491 + 9;
        fill(n, seed);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        fill(n, seed);
        let mine = unsafe { restore_saved_car_at(&p) };
        let state = capture(&all);
        let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
        t.check(regs_ok && theirs == state, || {
            format!("count {n:#x}: regs_ok={regs_ok} {mine:?}; the memory differs")
        });
    }
    for n in [0x7Du32, 0x8000_0000] {
        fill(n, 1);
        let before = capture(&all);
        let refused = unsafe { restore_saved_car_at(&p) }.is_err();
        t.check(refused && capture(&all) == before, || {
            format!("count {n:#x}: refused {refused}, or something written")
        });
    }
    drop((copy, cells));
    t.finish()
}
