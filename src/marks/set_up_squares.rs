//! `marks::set_up_squares` (0x4BF529): in `sub_4C0F2E`, the items cleared and the tables of
//! four-point circles (`g_marksSquareItems` .., 0x800 of 0x3C), then `marks::reset_outline`.
//!
//! In:  nothing
//! Out: see `marks/shapes.rs`
//!
//! See `marks/shapes.rs` (its once-only mark only set (nothing else is done once)). One caller, `sub_4C0F2E`. Refuses a once-only mark
//! neither 0 nor where it goes on. Self-test: the original and the Rust version with the
//! mark 0 and set, the game's radii and radii by lot; everything put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::marks::shapes::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::set_up_squares", rva(0x4BF529), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let resume = game_addr(SQUARES.resume) as u32;
    Ok(left_of(&unsafe { set_up_squares(r, resume) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &square_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = square_regions();
    let radii: [Region; 1] = [(game_addr(SQUARES.radii) as u32, 32)];
    let saved = (capture(&regions), capture(&radii));
    let resume = game_addr(SQUARES.resume) as u32;
    for done in [0u32, resume] {
        for by_lot in [false, true] {
            check_state(
                &mut t,
                &format!(
                    "mark {done:#x}, radii {}",
                    if by_lot { "by lot" } else { "the game's" }
                ),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&regions, &saved.0);
                    restore(&radii, &saved.1);
                    if by_lot {
                        let mut v = [0u32; 8];
                        crate::scene::matrix3::test_floats(done + 7, &mut v);
                        game_write(SQUARES.radii, v);
                    }
                    game_write(SQUARES.done, done);
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe { game_write(SQUARES.done, 1u32) };
    t.check(
        unsafe { set_up_squares(&Regs::SENTINEL, resume) }.is_err(),
        || "a mark of 1: the Rust version went on".into(),
    );
    restore(&regions, &saved.0);
    restore(&radii, &saved.1);
    t.finish()
}
