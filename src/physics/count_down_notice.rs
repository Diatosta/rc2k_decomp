//! `physics::count_down_notice` (0x440F08): while a notice is shown (`dword_71A700` set)
//! and its time `dword_71A704` is not 0, the time less 1; at 0 the notice is cleared
//! (`dword_71A700` 0). `physics::take_notice` sets them.
//!
//! In:  nothing
//! Out: eax the time before (when `dword_71A700` is set); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B919), every physics step.
//! Self-test: a copy of the original on test variables, shown or not, times 0, 1, 2,
//! large.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::count_down_notice",
    rva(0x440F08),
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

/// The Rust core on (shown, time) at `v`; `r` as the original leaves it.
pub unsafe fn count_down_at(v: [u32; 2], r: &mut Regs) {
    unsafe {
        let (shown, time) = (v[0] as *mut u32, v[1] as *mut u32);
        if shown.read_unaligned() == 0 {
            return;
        }
        r.eax = time.read_unaligned();
        if r.eax == 0 {
            return;
        }
        let left = r.eax - 1;
        time.write_unaligned(left);
        if left == 0 {
            shown.write_unaligned(0);
        }
    }
}

fn game() -> [u32; 2] {
    [0x71A700usize, 0x71A704].map(|x| game_addr(rva(x)) as u32)
}

/// The Rust version.
pub unsafe fn count_down_notice(r: &mut Regs) {
    unsafe { count_down_at(game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { count_down_notice(&mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[0], 4), (v[1], 4)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 2];
    let base = cells.as_mut_ptr() as u32;
    let v = [base, base + 4];
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x440F08,
            len: 0x26,
            entry: &[0x83, 0x3D, 0x00, 0xA7, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x71A700, v[0]), (0x71A704, v[1])],
        4,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all: [Region; 1] = [(base, 8)];
    for shown in [0u32, 0xCF] {
        for time in [0u32, 1, 2, 0x12C, u32::MAX] {
            let fill = || unsafe {
                (v[0] as *mut u32).write_volatile(shown);
                (v[1] as *mut u32).write_volatile(time);
            };
            fill();
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill();
            let mut g = Regs::SENTINEL;
            unsafe { count_down_at(v, &mut g) };
            let mine = capture(&all);
            let regs_ok = Left::nothing()
                .with(EAX, g.eax)
                .matches(&Regs::SENTINEL, &ro);
            t.check(regs_ok && theirs == mine, || {
                format!(
                    "shown {shown:#x}, time {time:#x}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    drop((copy, cells));
    t.finish()
}
