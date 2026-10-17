//! `drive::car_near_checkpoint` (0x4375F2): whether a car (esi) in the stage (+0x25C not
//! 0) is within 3 checkpoints of eax: CF when |+0x98 - eax| (`neg` of a negative) less 4
//! has its sign set.
//!
//! In:  esi a car block, eax a checkpoint
//! Out: CF; ebx the distance when +0x25C is not 0; the rest kept (ZF not read: `jb`)
//!
//! Caller: `drive::checkpoint_taken` (four cars). Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original on a test car, 40 cases.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "drive::car_near_checkpoint",
    rva(0x4375F2),
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

/// The Rust core; `r` the registers at entry.
pub unsafe fn near_at(car: u32, r: &Regs) -> FlagResult {
    unsafe {
        let get = |o: u32| ((car + o) as *const u32).read_unaligned();
        let result = |ebx: Option<u32>, carry| FlagResult {
            eax: r.eax,
            ecx: None,
            ebx,
            carry,
            zero: None,
        };
        if get(0x25C) == 0 {
            return result(None, false);
        }
        let mut d = get(0x98).wrapping_sub(r.eax);
        if (d as i32) < 0 {
            d = d.wrapping_neg();
        }
        result(Some(d), (d.wrapping_sub(4) as i32) < 0)
    }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { near_at(r.esi, r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 0x260 / 4];
    let car = cells.as_mut_ptr() as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4375F2,
            len: 0x21,
            entry: &[0x83, 0xBE, 0x5C, 0x02, 0x00, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for in_stage in [0u32, 1] {
        for (own, at) in [
            (10u32, 10u32),
            (10, 13),
            (10, 14),
            (14, 10),
            (13, 10),
            (0, 0x8000_0000),
            (0x8000_0000, 0),
            (5, 1),
            (1, 5),
            (0, 3),
            (u32::MAX, 2),
            (2, u32::MAX),
            (0x7FFF_FFFF, 0x8000_0002),
            (100, 97),
            (97, 100),
            (50, 0),
            (0, 50),
            (3, 0),
            (4, 0),
            (0, 4),
        ] {
            unsafe {
                ((car + 0x25C) as *mut u32).write(in_stage);
                ((car + 0x98) as *mut u32).write(own);
            }
            let before = Regs {
                esi: car,
                eax: at,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let mine = unsafe { near_at(car, &before) };
            t.check(mine.matches(&before, &ro), || {
                format!("in stage {in_stage}, own {own:#x}, checkpoint {at:#x}: original ebx {:#x} carry {}; rust {mine:?}", ro.ebx, ro.carry())
            });
        }
    }
    drop((copy, cells));
    t.finish()
}
