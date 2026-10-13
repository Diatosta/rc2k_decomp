//! `scene::light_dot` (0x44EC18): how much a light falls on a point, as a whole number.
//!
//! In:  nothing
//! Out: eax = the number; everything else preserved
//!
//! The three floats at `dword_62DAA8` times the three at `dword_62DAB4`, added (the
//! second and third products first), times `flt_62DAC0`, rounded as the x87 is set to
//! (`fistp`); also left in `dword_62400C`. x87, with the original's instructions.
//! Callers: the two lighting functions `sub_4511F1` and `sub_451296`, which put a
//! point's place in the first three, a light's direction in the second and its
//! strength in the factor.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::light_dot", rva(0x44EC18), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// `dword_62DAA8` .. `dword_62DAB0` the point, `dword_62DAB4` .. `dword_62DABC` the
/// light, `flt_62DAC0` the factor.
pub const INPUTS_RVA: usize = rva(0x62DAA8);

// ---- Rust version ----

pub unsafe fn light_dot() -> i32 {
    unsafe {
        let v: [u32; 7] = game_read(INPUTS_RVA);
        let mut out = 0i32;
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fmul dword ptr [{v} + 12]",
            "fld dword ptr [{v} + 4]",
            "fmul dword ptr [{v} + 16]",
            "fld dword ptr [{v} + 8]",
            "fmul dword ptr [{v} + 20]",
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fmul dword ptr [{v} + 24]",
            "fistp dword ptr [{o}]",
            v = in(reg) v.as_ptr(),
            o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        game_write(CHECKED_RVA, out);
        out
    }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [at(CHECKED_RVA)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { light_dot() } as u32))
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
    let inputs = [(crate::framework::game_addr(INPUTS_RVA) as u32, 28)];
    let saved = (capture(&regions), capture(&inputs));
    let cases: [[f32; 7]; 12] = [
        [0.0; 7],
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 1.0],
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 0.5],
        [100.5, -200.25, 9000.0, 0.6, -0.8, 0.0, 255.0],
        [-3000.0, 2500.0, 10000.0, 0.577, 0.577, 0.577, 0.0001],
        [1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0],
        [1.0, 1.0, 1.0, 0.5, 0.5, 1.5, 1.0],
        [1e20, 1e20, 1e20, 1e20, 1e20, 1e20, 1.0],
        [16777217.0, 1.0, 0.33333334, 1.0, 0.33333334, 3.0, 1.0],
        [f32::NAN, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [0.1, 0.2, 0.3, -0.7, 0.9, -0.11, 12345.678],
        [-1.0, -2.0, -3.0, 4.0, 5.0, 6.0, 0.25],
    ];
    for case in cases {
        check_state(
            &mut t,
            &format!("{case:?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(CHECKED_RVA, 0xAAAA_AAAAu32);
                game_write(INPUTS_RVA, case.map(f32::to_bits));
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
