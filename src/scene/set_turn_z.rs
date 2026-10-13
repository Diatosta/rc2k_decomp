//! `scene::set_turn_z` (0x42715D): the turning matrix about the z axis for an angle of
//! 32 bits a turn.
//!
//! In:  edx = the angle, a whole turn being 2^32
//! Out: eax = the sine, ebx = the cosine, ecx = the sine negated, edi = the matrix;
//!      everything else preserved
//!
//! The angle as a whole number (signed) over `flt_5C5D98` (65536) twice and over
//! `flt_5C5DD4` (1 / 2 pi), then `fsincos`: the instruction itself, as its result is
//! the processor's. The pair is kept in `dword_624158 and dword_62415C`, the angle in
//! `dword_62400C`, and goes to `flt_5EA00C`: +0 the cosine, +4 the sine, +0x10 the sine negated, +0x14 the cosine.
//! The matrix's other floats stay as they are.
//! Self-test: 48 angles on the game's matrix, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::matrix::test_floats;
use crate::scene::view_matrix::{self, Axis};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_turn_z", rva(0x42715D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

const AXIS: Axis = Axis::Z;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { view_matrix::set_fine_turn(AXIS, r.edx) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &AXIS.regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = AXIS.regions();
    let saved = capture(&regions);
    let mut seed = 0x2468_ACE5u32;
    for n in 0..48u32 {
        let angle = match n {
            0 => 0,
            1 => 1,
            2 => 0xFFFF_FFFF,
            3 => 0x4000_0000,
            4 => 0x8000_0000,
            5 => 0x7FFF_FFFF,
            6 => 0xC000_0000,
            7 => 0x2000_0000,
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
