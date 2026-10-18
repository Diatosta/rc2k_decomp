//! `scene::full_view_window` (0x44570A): `scene::choose_view_window` with `word_636526` 2
//! (800 x 600 when framed), the word back to 0 after.
//!
//! In:  nothing
//! Out: as `choose_view_window` leaves them
//!
//! Callers: `sub_45A893` (0x45A906), code at 0x4435B8. Self-test: the original and the
//! Rust version on the game's variables, framed and not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::scene::choose_view_window::{MODE_RVA, choose_view_window, regions};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::full_view_window", rva(0x44570A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): what it leaves in the registers.
pub unsafe fn full_view_window() -> Left {
    unsafe {
        game_write(MODE_RVA, 2u16);
        let left = choose_view_window();
        game_write(MODE_RVA, 0u16);
        left
    }
}

fn all_regions() -> Vec<crate::service::harness::Region> {
    let mut r = regions();
    r.push(at(MODE_RVA));
    r
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { full_view_window() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &all_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = all_regions();
    let mut vars = regions.clone();
    vars.extend(crate::scene::set_view_window::inputs());
    vars.extend([rva(0x621514), rva(0x621480)].map(at));
    let saved = capture(&vars);
    for case in 0..3u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(rva(0x621514), u32::from(case != 0));
                game_write(rva(0x621480), u32::from(case == 2));
                game_write(MODE_RVA, 1u16);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
