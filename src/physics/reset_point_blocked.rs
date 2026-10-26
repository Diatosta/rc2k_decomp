//! `physics::reset_point_blocked` (0x43F0DF): whether the checkpoint `dword_614C50` is no
//! place to put a reset car, as `sub_43EFEF` ("cant reset car anywhere - waiting a
//! second") asks it of the other cars' blocks (esi 0x7103CC, 0x7113AC, 0x71238C, ...).
//! Only when the block's +0x25C is 2 (by the look of it, a car in the stage; a reading):
//! CF when the checkpoint is within 8 of the start or the end (`g_checkpointCount` - 8),
//! or within -8 .. 7 of the block's +0x94 (its checkpoint). The comparisons take the sign
//! of the difference (`js`/`jns`), as here.
//!
//! In:  esi the block
//! Out: CF; eax and ebx as the path leaves them (see `reset_point_blocked_at`); the rest
//!      kept. ZF is not read (`jb`).
//!
//! Caller: `sub_43EFEF` (six calls), when a car is reset onto the line: not every frame.
//! Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original on test variables, kinds 1 .. 3, checkpoints across
//! the window and the stage's ends, counts small and large.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use rc2k_core::physics::line_places::{self, Blocked};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::reset_point_blocked",
    rva(0x43F0DF),
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

const CANDIDATE_RVA: usize = rva(0x614C50); // dword_614C50: the checkpoint tried
const CHECKPOINT_COUNT_RVA: usize = rva(0x623640); // g_checkpointCount

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core: the candidate and count read at `candidate`, `count`; `r` the registers
/// at entry.
// Converted: `rc2k_core::physics::line_places::reset_point_blocked`.
pub unsafe fn reset_point_blocked_at(candidate: u32, count: u32, r: &Regs) -> FlagResult {
    let b = unsafe {
        line_places::reset_point_blocked(
            at(r.esi + 0x25C),
            at(candidate),
            || at(count),
            || at(r.esi + 0x94),
        )
    };
    let (eax, ebx) = match b {
        Blocked::Away => (r.eax, None),
        Blocked::Start => (unsafe { at(candidate) }, None),
        Blocked::End { end } => (unsafe { at(candidate) }, Some(end)),
        Blocked::Gap { end, gap, .. } => (gap, Some(end)),
    };
    FlagResult {
        eax,
        ecx: None,
        ebx,
        carry: b.carry(),
        zero: None,
    }
}

/// The Rust version.
pub unsafe fn reset_point_blocked(r: &Regs) -> FlagResult {
    unsafe {
        reset_point_blocked_at(
            game_addr(CANDIDATE_RVA) as u32,
            game_addr(CHECKPOINT_COUNT_RVA) as u32,
            r,
        )
    }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { reset_point_blocked(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 2 + 0x260 / 4];
    let base = cells.as_mut_ptr();
    let (candidate, count, block) = (base, unsafe { base.add(1) }, unsafe { base.add(2) });
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F0DF,
            len: 0x39,
            entry: &[0x83, 0xBE, 0x5C, 0x02, 0x00, 0x00, 0x02],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x614C50, candidate as u32), (0x623640, count as u32)],
        3,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for kind in [1u32, 2, 3] {
        for n in [0u32, 10, 16, 40, 0x200, 0x8000_0010] {
            for c in [
                0u32,
                7,
                8,
                9,
                20,
                31,
                32,
                33,
                0x1F8,
                0x7FFF_FFFF,
                0x8000_0000,
                u32::MAX,
            ] {
                for own in [0u32, 13, 20, 27, 28, 0x1F0, 0x8000_0005] {
                    unsafe {
                        candidate.write_volatile(c);
                        count.write_volatile(n);
                        block.add(0x25C / 4).write_volatile(kind);
                        block.add(0x94 / 4).write_volatile(own);
                    }
                    let before = Regs {
                        esi: block as u32,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    unsafe { call_game(&mut ro, copy.addr()) };
                    let mine =
                        unsafe { reset_point_blocked_at(candidate as u32, count as u32, &before) };
                    t.check(mine.matches(&before, &ro), || {
                        format!(
                            "kind {kind}, count {n:#x}, checkpoint {c:#x}, its own {own:#x}: original eax {:#x} ebx {:#x} carry {}; rust {mine:?}",
                            ro.eax,
                            ro.ebx,
                            ro.carry()
                        )
                    });
                }
            }
        }
    }
    drop((copy, cells));
    t.finish()
}
