//! `race::normalise_work_vector` (0x41FC4D): `g_workVector` made one long.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/norms.rs` (the length is the 80-bit root, unlike `scene::unit_vector`'s). Eight
//! callers (`race::face_normal` and the race's code). Self-test: the game's vector and
//! length with vectors of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::normalise_work_vector",
    rva(0x41FC4D),
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { normalise_work_vector() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &norm_regions()[..2], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = norm_regions();
    let saved = capture(&regions);
    let f = |x: f32| x.to_bits();
    let mut cases = vec![
        [0, 0, 0],
        [0x8000_0000, 0, 0],
        [f(1.0), f(2.0), f(2.0)],
        [f(-3.0), f(0.0), f(4.0)],
        [f(1e-30), f(-1e-30), f(1e-30)],
        [f(3.0e18), f(-7.0e18), f(1.0e18)],
        [f(-0.0), f(5.0), f(-0.0)],
    ];
    let mut s = 0x2468_ACE1u32;
    for _ in 0..40 {
        s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let a = (s >> 9) as f32 - 4_000_000.0;
        s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let b = (s >> 7) as f32 * 0.37 - 1.0e7;
        s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let c = (s >> 11) as f32 * 1.7e-3;
        cases.push([f(a), f(b), f(c)]);
    }
    for v in cases {
        check_state(
            &mut t,
            &format!("{v:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(WORK_VECTOR_RVA, v);
                game_write(LENGTH_RVA, 0xAAAA_AAAAu32);
            },
            &regions[..2],
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
