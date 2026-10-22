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
//! afterwards; every place is a float, held bit for bit but for a NaN's bits
//! (`Tolerance::Exact`: with shadow off the original's `invert_turn` is the converted one).

use crate::errors::Refused;
use crate::framework::{GameHook, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::start_place::*;
use crate::scene::matrix::{MATRIX_LEN, test_floats};
use crate::screens::harness::Left;
use crate::service::floats::{Floats, Tolerance, check_state_floats};
use crate::service::harness::{capture, dispatch_state, restore, state_entry};
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
    let exact: Vec<_> = regions.iter().map(|&r| (r, Tolerance::Exact)).collect();
    let mut seed = 0x2026_1017u32;
    for case in 0..20 {
        let floats = test_floats(&mut seed);
        check_state_floats(
            &mut t,
            HOOK.name(),
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
            &Floats {
                memory: &exact,
                regs: &[],
            },
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
