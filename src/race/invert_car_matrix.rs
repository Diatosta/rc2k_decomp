//! `race::invert_car_matrix` (0x43F16C): the work car's matrix (`g_crashMatrix`,
//! 0x71A08C, `g_car` +0xB4) undone by `scene::invert_turn` into `g_carInverseMatrix`
//! (0x71A9A4, +0x9CC); `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/start_place.rs`. Callers: `sub_43FB47` (race init's camera start, and three
//! more) and `sub_43B635` three times (the physics, stream B's). Shadow mode: the Rust
//! version, the matrices and `invert_turn`'s work put back, the original's, compared.
//! Self-test: the game's car matrix set to matrices by lot, both versions, put back
//! afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::start_place::*;
use crate::scene::matrix::{MATRIX_LEN, test_floats};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::invert_car_matrix", rva(0x43F16C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { invert_car_matrix() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &invert_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = invert_regions();
    let matrix = (game_addr(CAR_MATRIX_RVA) as u32, MATRIX_LEN);
    let mut kept = regions.clone();
    kept.push(matrix);
    let saved = capture(&kept);
    let mut seed = 0x2026_1017u32;
    for case in 0..20 {
        let floats = test_floats(&mut seed);
        check_state(
            &mut t,
            &format!("case {case}: {floats:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&regions, &vec![0xAA; capture(&regions).len()]);
                restore(
                    &[matrix],
                    &floats
                        .iter()
                        .flat_map(|f| f.to_le_bytes())
                        .collect::<Vec<u8>>(),
                );
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
