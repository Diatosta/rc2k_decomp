//! `physics::turn_by_heading` (0x43F136): the floats at esi (x) and esi+8 (z) turned by
//! minus the work car's heading (`g_car` +0x9C0, `unk_71A998`) with
//! `math::turn_float_point_xz` (ebx passed through as y) and stored back.
//!
//! In:  esi the point, ebx passed on
//! Out: eax x', ecx z', edx as `turn_scratch_xz` leaves it; the rest kept; no flags read
//!
//! Caller: `sub_45E800` (0x45E8A9).
//! Self-test: the original and the Rust version on the game's variables (the heading and
//! the scratch point saved and put back) and a test point, under three precisions.

use crate::errors::Refused;
use crate::framework::{
    EAX, ECX, EDX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::math::turn_float_point_xz::turn_float_point_xz;
use crate::math::turn_scratch_xz::scratch;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::turn_by_heading", rva(0x43F136), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const HEADING_RVA: usize = rva(0x71A998); // g_car + 0x9C0

/// The Rust version.
pub unsafe fn turn_by_heading(point: u32, ebx: u32) -> Left {
    unsafe {
        let p = point as *mut u32;
        let angle = game_read::<u32>(HEADING_RVA).wrapping_neg();
        let [x, _, z, entry] =
            turn_float_point_xz(p.read_unaligned(), ebx, p.add(2).read_unaligned(), angle);
        p.write_unaligned(x);
        p.add(2).write_unaligned(z);
        Left::nothing().with(EAX, x).with(ECX, z).with(EDX, entry)
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { turn_by_heading(r.esi, r.ebx) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.esi, 12), scratch()], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut point = vec![0u32; 3];
    let pt = point.as_mut_ptr() as u32;
    let all = [(pt, 12), scratch(), (game_addr(HEADING_RVA) as u32, 4)];
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=8u32 {
                let set = || {
                    let mut f = [0u32; 3];
                    test_floats(seed * 31 + 7, &mut f);
                    unsafe { (pt as *mut [u32; 3]).write(f) };
                    unsafe { game_write(HEADING_RVA, seed.wrapping_mul(0x1357_9BDF)) };
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs {
                        esi: pt,
                        ebx: (seed as f32 * 2.5).to_bits(),
                        ..Regs::SENTINEL
                    },
                    &set,
                    &all,
                    &rust,
                );
            }
        });
    }
    restore(&all, &saved);
    drop(point);
    t.finish()
}
