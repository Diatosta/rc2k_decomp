//! `drive::duel_sound_levels` (0x45C596): two sound levels from the first car block
//! (`g_carBlockStore`): +0x900 / 4 - 4, held 0 ..= 0x40 (by the signs), into
//! `dword_5165BC`; at 0x38 or more `dword_5165A8` 1 when a random number
//! (`math::next_random`) x 500 has a high dword of 0 (a rare draw); (+0x908 squared, low
//! dword) >> 12 - 8, held, into `dword_5165C0`.
//!
//! In:  nothing
//! Out: eax the second level, edx the square's high dword, ebx 500 after a draw; the rest
//!      kept; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version on the figures (saved and put back), the
//! random state included.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::duel_sound_levels", rva(0x45C596), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const B: usize = 0x70F3EC; // g_carBlockStore

fn held(v: u32) -> u32 {
    let v = if (v as i32) < 0 { 0 } else { v };
    if (v.wrapping_sub(0x40) as i32) < 0 {
        v
    } else {
        0x40
    }
}

/// The Rust version.
pub unsafe fn duel_sound_levels(entry: &Regs) -> Left {
    let mut left = Left::nothing();
    p(0x5165BC, held((g(B + 0x900) >> 2).wrapping_sub(4)));
    if g(0x5165BC) >= 0x38 {
        let r = unsafe { crate::math::next_random::next_random() };
        let product = i64::from(r as i32) * 500;
        left = left.with(EBX, 500);
        if (product >> 32) as u32 == 0 {
            p(0x5165A8, 1);
        }
    }
    let x = g(B + 0x908);
    let square = i64::from(x as i32) * i64::from(x as i32);
    let eax = held(((square as u32) >> 12).wrapping_sub(8));
    p(0x5165C0, eax);
    let _ = entry;
    left.with(EAX, eax).with(EDX, (square >> 32) as u32)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { duel_sound_levels(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    let state = crate::framework::game_addr(crate::math::next_random::STATE_RVA) as u32;
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(a(0x5165A8), 4), (a(0x5165BC), 8), (state, 4)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![
        (a(0x70F3EC + 0x900), 12),
        (a(0x5165A8), 4),
        (a(0x5165BC), 8),
        (
            crate::framework::game_addr(crate::math::next_random::STATE_RVA) as u32,
            4,
        ),
    ];
    let saved = capture(&all);
    for case in 0..16u32 {
        let set = || {
            p(
                0x70F3EC + 0x900,
                [0u32, 0x10, 0xF0, 0x200, 0x8000_0000][(case % 5) as usize],
            );
            p(
                0x70F3EC + 0x908,
                [0u32, 0x200, 0x900, 0x1_0000, 0xFFFF_F000][(case % 5) as usize],
            );
            p(0x5165A8, 0);
            unsafe {
                crate::framework::game_write(
                    crate::math::next_random::STATE_RVA,
                    case.wrapping_mul(0x2545_F491),
                )
            };
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
