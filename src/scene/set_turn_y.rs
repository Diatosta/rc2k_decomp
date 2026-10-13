//! `scene::set_turn_y` (0x42702A): the turning matrix about the y axis for an angle.
//!
//! In:  edx = the angle, sixteen bits a turn
//! Out: eax = the sine, ebx = the cosine, ecx = the sine negated, edi = the matrix;
//!      everything else preserved
//!
//! The sine and cosine come from the game's two tables of 0x1000 floats (the angle's
//! low four bits dropped), are kept in `dword_624150` and `dword_624154`, and go to the matrix
//! `TURN_Y_RVA`: +0 the cosine, +8 the sine negated, +0x20 the sine, +0x28 the cosine.
//! Its other floats stay as they are.
//! 19 callers.
//! Self-test: 41 angles on the game's matrix, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::scene::matrix::{self, matrix, test_floats};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_turn_y", rva(0x42702A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn regions() -> [Region; 2] {
    [
        matrix(matrix::TURN_Y_RVA),
        (game_addr(matrix::TURN_Y_WORK_RVA) as u32, 8),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { matrix::set_turn_y(r.edx) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let mut seed = 0x2468_ACE1u32;
    for n in 0..41u32 {
        let angle = match n {
            0 => 0,
            1 => 0xF,
            2 => 0x10,
            3 => 0xFFFF,
            4 => 0x1_0000,
            5 => 0xFFFF_FFF0,
            6 => 0x8000_4000,
            n => n.wrapping_mul(0x0612_3457),
        };
        let floats = test_floats(&mut seed);
        check_state(
            &mut t,
            &format!("angle {angle:#x}"),
            call_original,
            Regs {
                edx: angle,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.len()]);
                (regions[0].0 as *mut [u32; 16]).write_unaligned(floats);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
