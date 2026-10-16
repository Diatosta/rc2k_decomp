//! `scene::set_turn_z_16` (0x427072): the turning matrix about the z axis for an angle of
//! sixteen bits a turn.
//!
//! In:  edx = the angle, sixteen bits a turn
//! Out: eax = the sine, ebx = the cosine, ecx = the sine negated, edi = the matrix;
//!      everything else preserved
//!
//! As `scene::set_turn_x`: the sine and cosine kept in `dword_624158` and `dword_62415C`,
//! and to `flt_5EA00C` (the matrix of `scene::set_turn_z`, whose angle is 32 bits a turn):
//! +0 the cosine, +4 the sine, +0x10 the sine negated, +0x14 the cosine. Three callers
//! (`race::get_mat_crash` and two more). Self-test: 41 angles on the game's matrix, put
//! back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::scene::matrix::{self, matrix, test_floats};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_turn_z_16", rva(0x427072), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn regions() -> [Region; 2] {
    [
        matrix(crate::scene::view_matrix::TURN_Z_RVA),
        (
            game_addr(crate::scene::view_matrix::TURN_Z_WORK_RVA) as u32,
            8,
        ),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { matrix::set_turn_z_16(r.edx) })
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
    let mut seed = 0x1357_9BDFu32;
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
